#!/bin/sh

set -eu

if [ "$#" -ne 1 ] || [ -z "$1" ] || [ "$1" = "/" ]; then
  echo "usage: $0 OUTPUT_DIRECTORY" >&2
  exit 2
fi

output_dir=$1
repo_root=$(git rev-parse --show-toplevel)
baseline=$(sed -n '1p' "$repo_root/tests/reference/BASELINE")
source_dir=$output_dir/source
build_dir=$output_dir/build

if [ -e "$output_dir" ]; then
  echo "output directory already exists: $output_dir" >&2
  exit 2
fi

mkdir -p "$output_dir"
git worktree add --detach "$source_dir" "$baseline"
meson setup "$build_dir" "$source_dir" -Ddoc=false
meson compile -C "$build_dir"

echo "$build_dir/terminal/xfce4-terminal"

