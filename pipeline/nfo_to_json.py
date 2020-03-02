#!/usr/bin/env python3
"""
Create a JSON file that's roughly equivalent to the Kodi .nfo file format.
Currently, only a subset of the "tvshow" format is supported.

Dealing with xml sucks in most languages, so let's get rid of it! The inputs are
expected to be well-formed, so little-to-no error-checking is done.
"""
import argparse
import json
import os
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

def get_node_text(node):
    return node.text

def get_node_int(node):
    return int(node.text)


specs = {
    "tvshow": {
        "title": {"default": "Untitled", "multiple": False, "transform": get_node_text},
        "plot": {"default": "No plot information", "multiple": False, "transform": get_node_text},
        "year": {"default": 0, "multiple": False, "transform": get_node_int},
        "genre": {"default": [], "multiple": True, "transform": get_node_text},
    },
    "episodedetails": {
        "title": {"default": "Untitled", "multiple": False, "transform": get_node_text},
        "showtitle": {"default": "Untitled", "multiple": False, "transform": get_node_text},
        "season": {"default": 0, "multiple": False, "transform": get_node_int},
        "episode": {"default": 0, "multiple": False, "transform": get_node_int},
        "plot": {"default": "No plot information", "multiple": False, "transform": get_node_text},
    },
}

def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="Input .nfo file or directory containing .nfo files")
    parser.add_argument("-o", "--output", help="Output file (defaults to stdout)")

    return parser.parse_args()


# Ubuntu 18.04 doesn't use Python 3.7 by default. For now we'll return a dict
# rather than using a dataclass.
def parse_nfo(xml: ET.Element, spec):
    info = default_info(spec)
    for child in xml:
        tag_spec = spec.get(child.tag)
        if tag_spec:
            tag_value = tag_spec["transform"](child)
            if tag_spec["multiple"]:
                info.setdefault(child.tag, []).append(tag_value)
            else:
                info[child.tag] = tag_value
        else:
            # Print on stderr so that we don't interfere with normal shell redirects
            print(f"Ignoring tag {child.tag}", file=sys.stderr)

    return info


# This is a function since we always want a copy of the default
def default_info(spec):
    info = {
        tag_name: tag_spec["default"]
        for tag_name, tag_spec in spec.items()
    }

    return info


def nfo_to_json(input_file, output_file):
    root = ET.fromstring(input_file.read())

    info = parse_nfo(root, specs[root.tag])
    info_json = json.dumps(info, indent=4)

    output_file.write(info_json)
    output_file.write("\n")


if __name__ == "__main__":
    args = parse_args()

    path = Path(args.input)

    assert path.exists(), f"Path {path} does not exist"

    if path.is_file():
        with open(path) as input_file:
            if args.output:
                output_file = open(args.output, "w")
                nfo_to_json(input_file, output_file)
                output_file.close()
            else:
                output_file = sys.stdout
                nfo_to_json(input_file, output_file)

    else:
        for root, dirs, files in os.walk(path):
            for name in files:
                if Path(name).suffix != ".nfo":
                    continue
                nfo_path = Path(root) / name
                json_path = nfo_path.with_suffix(".json")

                with open(nfo_path) as input_file:
                    with open(json_path, "w") as output_file:
                        nfo_to_json(input_file, output_file)
