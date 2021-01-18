#!/usr/bin/env python3

import argparse
import contextlib
import os
import subprocess
from pathlib import Path


@contextlib.contextmanager
def run_in_dir(d):
    original_cwd = os.getcwd()
    try:
        os.chdir(d)
        yield
    finally:
        os.chdir(original_cwd)


parser = argparse.ArgumentParser()
parser.add_argument("dest", help="user@sub.foo.com")
parser.add_argument("--loc", default="/opt")
parser.add_argument("-n", "--dry-run", action="store_true")

args = parser.parse_args()

def print_and_run(cmd, cwd=None):
    if cwd:
        print(f"(From {cwd}) ", end="")
    print(cmd)
    if not args.dry_run:
        subprocess.check_call(cmd, cwd=cwd, shell=True)

base_dir = Path(__file__).parent.absolute()

# Compile backend
print_and_run(f"cargo build --release", cwd=(base_dir / "backend"))

# Compile frontend
print_and_run("elm make src/Main.elm --output=main.js", cwd=(base_dir / "frontend"))

COPY_EXCLUSIONS = [
    'backend/target/debug',
    'backend/target/*/deps',
    'backend/target/*/build',
    'backend/target/*/.fingerprint',
    'backend/target/*/incremental',
    'Bideorai.toml',
    'elm-stuff/*',
    '.git',
    '*.swp',
]

exclusions = " ".join(f"--exclude='{s}'" for s in COPY_EXCLUSIONS)

# Deploy everything
print_and_run(f"rsync -av {exclusions} -e ssh {base_dir} {args.dest}:{args.loc}")
