#!/usr/bin/env python3
import json
import sys
import subprocess
import os
from pathlib import Path

script_dir = Path(__file__).absolute().parent

# Keep secrets out of git :)
with open(script_dir.joinpath("post_process_config.json").as_posix()) as f:
    config = json.loads(f.read())
assert "b2_bucket" in config

nfo_to_json = script_dir.joinpath("nfo_to_json.py").as_posix()
video_to_mpd = script_dir.joinpath("video_to_mpd.py").as_posix()

# We don't control the CLI, so it's not useful to use argparse
_, video_path, source_path, tvdbid, season, episode, air_date = sys.argv
season = int(season)
episode = int(episode)

# Convert the episode metadata from .nfo to .json

# Input metadata
nfo_metadata = Path(video_path).with_suffix(".nfo")

# Output metadata
json_metadata = Path(video_path).with_suffix(".json")

nfo_to_json_args = [
    nfo_to_json,
    nfo_metadata,
    "--output",
    json_metadata
]
print(nfo_to_json_args)
subprocess.check_call(nfo_to_json_args)

# Convert the video and upload to b2

subprocess.check_call([
    video_to_mpd,
    "--input",
    video_path,
    "--b2-bucket",
    config["b2_bucket"],
    "--b2-dir",
    f"content/{tvdbid}/S{season:02}E{episode:02}",
])
