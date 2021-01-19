#!/usr/bin/env python3.6
"""
This script is run by Medusa (https://github.com/pymedusa/Medusa) as a
post-process script (https://github.com/pymedusa/Medusa/wiki/Post-Processing).

This script just records the information provided by medusa into a
configurable directory:
    argv[0]: File-path to Script
    argv[1]: Final full path to the episode file
    argv[2]: Original full path of the episode file
    argv[3]: Show indexer ID
    argv[4]: Season number
    argv[5]: Episode number
    argv[6]: Episode Air Date

Previously, I (SHM) configured this to do much more (transcoding, backup,
and metadata conversion), but that had problems. The biggest problem was
that the post processing had bugs. In particular, certain directories
were being removed unexpectedly. I believe it's not the post processing
script that's doing this, but making the script more simple will show
that more obviously.

Medusa also has terrible, terrible feedback when post processing scripts fail.
Like, none. Super duper sucks. We need to do something _really_ simple here to
have any hope of feedback. We _can_ scrape the logs and re-run this if something
goes wrong, but it's nice to not have to!

Another benefit of structuring the pipeline this way is that it's easier
to debug. Rather than reproducing the command line arguments passed by
medusa to this script, I can use the output file from this script to
feed the next script in the pipeline. The JSON output from this script
is easier to understand at a glance.

I typically try to make things configurable. For this pipeline, I think
that has hurt more than it helped. It meant that _this_ script would
decide on output locations for files. These output locations don't need
to be configurable, and making the configurable (and required!) made
these scripts more annoying to develop. So, against what I typically do,
the invocations of these scripts will be simplified, and these scripts
will become less configurable.

There is a downside to this approach. It means I need to schedule
something to search through the output directory of this script to kick
off the next steps of the pipeline. Previously, I could rely on medusa
to kick things off.
"""
import json
import logging
import sys
import subprocess
import os
from pathlib import Path

from common import Config, VideoInfo

script_dir = Path(__file__).resolve().parent

config_file = script_dir.joinpath("post_process_config.json")
config = Config.from_json(config_file.read_text())

log_location = (
    Path(config.base_log_location) / Path(__file__).with_suffix(".log").parts[-1]
)

print(f"Logging to {log_location}", file=sys.stderr)
log_location.parent.mkdir(parents=True, exist_ok=True)

logging.basicConfig(filename=log_location, level=logging.DEBUG)

_, video_path, source_path, tvdbid, season, episode, air_date = sys.argv

video_info = VideoInfo(
    video_path=video_path,
    source_path=source_path,
    tvdbid=int(tvdbid),
    season=int(season),
    episode=int(episode),
    air_date=air_date,
)

post_process_queue_dir = Path(config.post_process_queue)
post_process_queue_dir.mkdir(parents=True, exist_ok=True)

info_path = post_process_queue_dir.joinpath(
    f"{video_info.tvdbid}_S{video_info.season}E{video_info.episode}"
)

logging.info(f"Writing {video_info} to {info_path}")

info_path.write_text(video_info.to_json() + "\n")
