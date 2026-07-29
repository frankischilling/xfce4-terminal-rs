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
  "$script_dir/login-shell-shim.c" \
  -o "$output" \
  -ldl
