#!/bin/sh

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_SOURCE OUTPUT_FILE" >&2
  exit 2
fi

reference_source=$1
output=$2
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)

cc \
  -I"$reference_source/terminal" \
  -I"$reference_source" \
  "$repo_root/tests/reference/options-probe.c" \
  "$reference_source/terminal/terminal-options.c" \
  -o "$output" \
  $(pkg-config --cflags --libs \
    gtk+-3.0 \
    libpcre2-8 \
    libxfce4ui-2 \
    libxfce4util-1.0 \
    vte-2.91)
