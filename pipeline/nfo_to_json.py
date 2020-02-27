#!/usr/bin/env python3
"""
Create a JSON file that's roughly equivalent to the Kodi .nfo file format.
Currently, only a subset of the "tvshow" format is supported.

Dealing with xml sucks in most languages, so let's get rid of it! The inputs are
expected to be well-formed, so little-to-no error-checking is done.
"""
import argparse
import json
import sys
import xml.etree.ElementTree as ET

def get_node_text(node):
    return node.text


tvshow_spec = {
    "title": {"default": "Untitled", "multiple": False, "transform": get_node_text},
    "plot": {"default": "No plot information", "multiple": False, "transform": get_node_text},
    "year": {"default": 0, "multiple": False, "transform": lambda node: int(node.text)},
    "genre": {"default": [], "multiple": True, "transform": get_node_text},
}

def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="Input .nfo file")
    parser.add_argument("-o", "--output", help="Output file (defaults to stdout)")

    return parser.parse_args()


# Ubuntu 18.04 doesn't use Python 3.7 by default. For now we'll return a dict
# rather than using a dataclass.
def parse_tvshow(xml: ET.Element):
    tvshow = default_tvshow()
    for child in xml:
        tag_spec = tvshow_spec.get(child.tag)
        if tag_spec:
            tag_value = tag_spec["transform"](child)
            if tag_spec["multiple"]:
                tvshow.setdefault(child.tag, []).append(tag_value)
            else:
                tvshow[child.tag] = tag_value
        else:
            # Print on stderr so that we don't interfere with normal shell redirects
            print(f"Ignoring tag {child.tag}", file=sys.stderr)

    return tvshow


# This is a function since we always want a copy of the default
def default_tvshow():
    tvshow = {
        tag_name: tag_spec["default"]
        for tag_name, tag_spec in tvshow_spec.items()
    }

    return tvshow


if __name__ == "__main__":
    args = parse_args()

    tree = ET.parse(args.input)
    root = tree.getroot()

    assert root.tag == 'tvshow'

    tvshow = parse_tvshow(root)
    tvshow_json = json.dumps(tvshow, indent=4)

    if args.output:
        output_file = open(args.output, 'w')
    else:
        output_file = sys.stdout

    output_file.write(tvshow_json)
    output_file.write("\n")
    output_file.close()
