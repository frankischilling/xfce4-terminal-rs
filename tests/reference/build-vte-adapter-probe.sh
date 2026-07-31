#!/bin/sh

# Builds the VTE adapter probe against the frozen reference build.

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
    terminal-widget.c \
    "$script_dir/vte-adapter-probe.c" \
    "$output"
)

cd "$reference_build"
eval "$command"
test -x "$output"
