#!/bin/sh

# Builds the screen-model probe against the frozen reference build.

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_BUILD OUTPUT_FILE" >&2
  exit 2
fi

reference_build=$(CDPATH= cd -- "$1" && pwd)
output_dir=$(CDPATH= cd -- "$(dirname "$2")" && pwd)
output=$output_dir/$(basename "$2")
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)

command=$(
  python3 "$script_dir/probe-command.py" \
    "$reference_build" \
    terminal-screen.c \
    "$script_dir/screen-probe.c" \
    "$output"
)

cd "$reference_build"
eval "$command"
test -x "$output"
