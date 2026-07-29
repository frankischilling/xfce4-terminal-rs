#!/bin/sh

set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 OUTPUT_FILE" >&2
  exit 2
fi

output=$1
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)

cc \
  -shared \
  -fPIC \
  "$script_dir/application-call-probe.c" \
  -o "$output" \
  $(pkg-config --cflags --libs gtk+-3.0) \
  -ldl
