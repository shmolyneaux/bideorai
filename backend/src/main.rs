#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket::fairing::AdHoc;
use rocket::State;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
struct Config {
    asset_dir: String,
}

impl Config {
    fn new() -> Self {
        Config {
            asset_dir: "/opt/medusa/content".to_string()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

struct AssetsDir (PathBuf);

/// Content needed when looking at an overview of many titles
#[derive(PartialEq, Debug, Serialize)]
struct TitleShort {
    title: String,
    poster_url: String,
}

/// Content needed when looking at a single title
#[derive(PartialEq, Debug, Serialize)]
struct TitleLong {
    title: String,
    poster_url: String,
    banner_url: String,
    // Genre, actors, year, and such?
    metadata: HashMap<String, String>,
    content: Vec<VideoContent>,
}

/// All information for a particular video
#[derive(PartialEq, Debug, Serialize)]
struct VideoContent {
    title: String,
    video_url: String,
    thumbnail_url: String,
    description: String,
    source: Option<String>,
    // Episode / season here?
    metadata: HashMap<String, String>,
}

#[get("/titles")]
fn titles_endpoint(asset_dir: State<AssetsDir>) -> Result<Json<Vec<TitleShort>>, String> {
    titles(&asset_dir.0).map(|titles_info| Json(titles_info))
}

/// Returns a list of names of _all_ titles
fn titles(asset_dir: &Path) -> Result<Vec<TitleShort>, String> {
    let asset_dir = asset_dir.canonicalize().map_err(|_| "Invalid configuration")?;
    let titles: Vec<String> = fs::read_dir(&asset_dir)
        .map_err(|_| "Could not read asset directory".to_string())?
        .map(|entry_result| {
            entry_result.map_err(|_| "Could not read entry".to_string())
                // We unwrap here since we assume all file names we have on
                // disk are utf-8
                .map(|entry| entry.file_name().into_string().unwrap())
        })
        .collect::<Result<Vec<String>, String>>()?;

    let titles_info = titles.iter().map(|title| {
        TitleShort {
            title: title.clone(),
            poster_url: format!("/assets/{}/poster.jpg", title),
        }
    })
    .collect();

    Ok(titles_info)
}

#[get("/titles/<name>")]
fn title_endpoint(asset_dir: State<AssetsDir>, name: String) -> Result<Json<TitleLong>, String> {
    title(&asset_dir.0, &name).map(|title_info| Json(title_info))
}

/// Returns the metadata for a single TV Show or movie
fn title(asset_dir: &Path, name: &str) -> Result<TitleLong, String> {
    let asset_dir = asset_dir.canonicalize().map_err(|_| "Invalid configuration")?;

    let mut title_path = PathBuf::new();
    title_path.push(&asset_dir);
    title_path.push(name);

    // Make sure there are no path traversal attacks
    title_path = title_path.canonicalize().map_err(|_| "Invalid title")?;
    if !title_path.starts_with(&asset_dir) {
        return Err("Invalid path".to_string());
    }

    let mut content_for_title: Vec<VideoContent> = Vec::new();

    for season_entry in fs::read_dir(dbg!(title_path)).map_err(|_| "Could not read asset directory".to_string())? {
        let season_entry = season_entry.map_err(|_| "Could not read season_entry".to_string())?;

        match season_entry.metadata() {
            Ok(meta) => if !meta.is_dir() { continue }
            _ => continue
        }

        let episode_entries = fs::read_dir(dbg!(season_entry.path())).map_err(|e| format!("{:?}", e))?;

        for entry in episode_entries {
            let entry = entry.map_err(|_| "Could not read season_entry".to_string())?;
            if entry.path().extension() == Some(OsStr::new("json")) {
                if let Some(video_content) = video_content_from_json_info(&asset_dir, &entry.path()) {
                    content_for_title.push(video_content)
                } else {
                    return Err("Read invalid content info on server".to_string());
                }
            }
        }
    }

    content_for_title.sort_unstable_by_key(|video| video.video_url.clone());

    Ok(TitleLong {
        title: name.to_string(),
        poster_url: format!("assets/{}/poster.jpg", &name),
        banner_url: format!("assets/{}/banner.jpg", &name),
        metadata: HashMap::new(),
        content: content_for_title,
    })
}


fn video_content_from_json_info(asset_dir: &Path, json_file: &Path) -> Option<VideoContent> {
    let base_path = json_file.file_stem()?.to_string_lossy();
    let video_path = json_file.parent()?.join((base_path.clone() + ".mpd").to_string());
    let thumbnail_url = json_file.parent()?.join((base_path.clone() + "-thumb.jpg").to_string());

    let info = json::parse(&fs::read_to_string(json_file).ok()?).ok()?;

    let mut metadata: HashMap<String, String> = HashMap::new();
    if let Some(episode_number) = info["episode"].as_u64() {
        metadata.insert("episode".to_string(), episode_number.to_string());
    }
    if let Some(season_number) = info["season"].as_u64() {
        metadata.insert("season".to_string(), season_number.to_string());
    }

    Some(VideoContent {
        title: info["title"].as_str()?.to_string(),
        video_url: path_to_asset_url(asset_dir, &video_path).ok()?,
        thumbnail_url: path_to_asset_url(asset_dir, &thumbnail_url).ok()?,
        description: info["plot"].as_str()?.to_string(),
        source: info["source"].as_str().map(|s| s.to_string()),
        metadata,
    })
}

// TODO: better name
static MY_SET: &percent_encoding::AsciiSet = &NON_ALPHANUMERIC.remove(b'.').remove(b'-');

fn path_to_asset_url(asset_dir: &Path, path: &Path) -> Result<String, ()> {
    let relative_path = path.strip_prefix(asset_dir).map_err(|_| ())?;
    let mut url: String = "/assets".to_string();

    for component in relative_path.components() {
        url.push_str("/");
        let p = percent_encode(component.as_os_str().as_bytes(), &MY_SET);
        let component: String = p.to_string();
        url.push_str(&component);
    }

    Ok(url)
}

fn get_config() -> Config {
    std::fs::read_to_string("Bideorai.toml")
        .map_err(|_| ())
        .and_then(|s| toml::from_str(&s).map_err(|_| ()))
        .unwrap_or_default()
}

fn main() {
    let asset_dir = dbg!(get_config()).asset_dir;

    rocket::ignite()
        .mount(
            "/",
            routes![
                title_endpoint,
                titles_endpoint,
            ]
        )
        .mount(
            "/assets",
            StaticFiles::from(asset_dir.clone())
        )
        .mount(
            "/",
            // Make this less specific with "rank"
            StaticFiles::from("static").rank(30)
        )
        .attach(AdHoc::on_attach("Assets Config", move |rocket| {
            Ok(rocket.manage(AssetsDir(Path::new(&asset_dir).to_path_buf())))
        }))
        .launch();
}

mod tests {
    extern crate tempdir;

    use super::*;

    use std::io::Write;
    use std::fs::File;

    use tempdir::TempDir;

    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[derive(Clone)]
    enum FileEntry {
        File {name: String, content: String},
        Directory {name: String, content: Vec<FileEntry>},
    }

    fn f(name: &str, content: &str) -> FileEntry {
        FileEntry::File { name: name.to_string(), content: content.to_string() }
    }

    fn d(name: &str, content: &[FileEntry]) -> FileEntry {
        FileEntry::Directory { name: name.to_string(), content: content.to_vec() }
    }

    fn hm(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn create_tmp_directory_structure(structure: &FileEntry) -> TempDir {
        let dir = TempDir::new("test_tmp").unwrap();
        create_tmp_directory_structure_inner(structure, &dir.path());
        dir
    }

    fn create_tmp_directory_structure_inner(structure: &FileEntry, dir: &Path) {
        match structure {
            FileEntry::File {name, content} => {
                let mut f = File::create(dir.join(name)).unwrap();
                f.write_all(content.as_bytes()).unwrap();
            },
            FileEntry::Directory {name, content} => {
                let new_dir = dir.join(name);
                fs::create_dir(&new_dir).unwrap();
                for entry in content {
                    create_tmp_directory_structure_inner(entry, &new_dir);
                }
            }
        }
    }

    #[test]
    pub fn test_convert_asset_path_to_url() {
        assert_eq!(
            path_to_asset_url(
                Path::new("/media/stephen/F870-7CFC/content"),
                Path::new("/media/stephen/F870-7CFC/content/ReZERO -Starting Life in Another World-/Season 01/ReZERO -Starting Life in Another World- - S01E11 - Rem.mpd"),
            ),
            Ok("/assets/ReZERO%20-Starting%20Life%20in%20Another%20World-/Season%2001/ReZERO%20-Starting%20Life%20in%20Another%20World-%20-%20S01E11%20-%20Rem.mpd".to_string())
        );
    }

    #[test]
    pub fn test_content_layout() {
        let dir = create_tmp_directory_structure(
            &d(
                "content",
                &[
                    d (
                        "Naruto",
                        &[
                            d(
                                "Season 1",
                                &[
                                    // Null source
                                    f(
                                        "Naruto - S01E01 - Enter: Naruto Uzumaki!.json",
                                        r#"
                                            {
                                                "title": "Enter: Naruto Uzumaki!",
                                                "plot": "Some cool story",
                                                "episode": 1,
                                                "season": 1,
                                                "source": null
                                            }
                                        "#
                                    ),
                                    // No Source
                                    f(
                                        "Naruto - S01E02 - My Name is Konohamaru.json",
                                        r#"
                                            {
                                                "title": "My Name is Konohamaru",
                                                "plot": "Another cool story",
                                                "episode": 2,
                                                "season": 1
                                            }
                                        "#
                                    ),
                                    // Specified source
                                    f(
                                        "Naruto - S01E03 - Sasuke and Sakura: Friends or Foes?.json",
                                        r#"
                                            {
                                                "title": "Sasuke and Sakura: Friends or Foes?",
                                                "plot": "A third cool story",
                                                "episode": 3,
                                                "season": 1,
                                                "source": "[ReleaseGroup].Naruto-03.[1080p]"
                                            }
                                        "#
                                    ),
                                ],
                            ),
                        ],
                    ),
                    d (
                        "Another Naruto",
                        &[
                            d(
                                "Season 1",
                                &[
                                    f(
                                        "Naruto - S01E01 - Enter: Naruto Uzumaki!.json",
                                        r#"
                                            {
                                                "title": "Enter: Naruto Uzumaki!",
                                                "plot": "Some cool story",
                                                "episode": 1,
                                                "season": 1
                                            }
                                        "#
                                    ),
                                    f(
                                        "Naruto - S01E02 - My Name is Konohamaru.json",
                                        r#"
                                            {
                                                "title": "My Name is Konohamaru",
                                                "plot": "Another cool story",
                                                "episode": 2,
                                                "season": 1
                                            }
                                        "#
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            )
        );

        assert_eq!(
            titles(&dir.path().join("content")),
            Ok::<Vec<TitleShort>, String>(vec![
                TitleShort {
                    title: "Naruto".to_string(),
                    poster_url: "/assets/Naruto/poster.jpg".to_string(),
                },
                TitleShort {
                    title: "Another Naruto".to_string(),
                    poster_url: "/assets/Another Naruto/poster.jpg".to_string(),
                },
            ])
        );

        assert_eq!(
            title(&dir.path().join("content"), "Naruto"),
            Ok::<TitleLong, String>(TitleLong {
                title: "Naruto".to_string(),
                poster_url: "assets/Naruto/poster.jpg".to_string(),
                banner_url: "assets/Naruto/banner.jpg".to_string(),
                metadata: HashMap::new(),
                content: vec![
                    VideoContent {
                        title: "Enter: Naruto Uzumaki!".to_string(),
                        video_url: "/assets/Naruto/Season%201/Naruto%20-%20S01E01%20-%20Enter%3A%20Naruto%20Uzumaki%21.mpd".to_string(),
                        thumbnail_url: "/assets/Naruto/Season%201/Naruto%20-%20S01E01%20-%20Enter%3A%20Naruto%20Uzumaki%21-thumb.jpg".to_string(),
                        description: "Some cool story".to_string(),
                        source: None,
                        metadata: hm(&[
                            ("season", "1"),
                            ("episode", "1")
                        ]),
                    },
                    VideoContent {
                        title: "My Name is Konohamaru".to_string(),
                        video_url: "/assets/Naruto/Season%201/Naruto%20-%20S01E02%20-%20My%20Name%20is%20Konohamaru.mpd".to_string(),
                        thumbnail_url: "/assets/Naruto/Season%201/Naruto%20-%20S01E02%20-%20My%20Name%20is%20Konohamaru-thumb.jpg".to_string(),
                        description: "Another cool story".to_string(),
                        source: None,
                        metadata: hm(&[
                            ("season", "1"),
                            ("episode", "2")
                        ]),
                    },
                    VideoContent {
                        title: "Sasuke and Sakura: Friends or Foes?".to_string(),
                        video_url: "/assets/Naruto/Season%201/Naruto%20-%20S01E03%20-%20Sasuke%20and%20Sakura%3A%20Friends%20or%20Foes%3F.mpd".to_string(),
                        thumbnail_url: "/assets/Naruto/Season%201/Naruto%20-%20S01E03%20-%20Sasuke%20and%20Sakura%3A%20Friends%20or%20Foes%3F-thumb.jpg".to_string(),
                        description: "A third cool story".to_string(),
                        source: Some("[ReleaseGroup].Naruto-03.[1080p]".to_string()),
                        metadata: hm(&[
                            ("season", "1"),
                            ("episode", "3")
                        ]),
                    },
                ],
            })
        )
    }
}
