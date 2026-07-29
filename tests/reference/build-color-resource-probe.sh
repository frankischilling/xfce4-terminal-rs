#!/bin/sh

set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 OUTPUT_FILE" >&2
  exit 2
fi

output=$1
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)

cc \
  "$script_dir/color-resource-probe.c" \
  -o "$output" \
  $(pkg-config --cflags --libs libxfce4util-1.0)
