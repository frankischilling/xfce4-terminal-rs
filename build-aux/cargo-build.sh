#!/bin/sh

set -eu

cargo_bin=$1
source_root=$2
target_dir=$3
output=$4
profile=$5

if [ "$profile" = "release" ]; then
  "$cargo_bin" build \
    --manifest-path "$source_root/Cargo.toml" \
    --target-dir "$target_dir" \
    --release
else
  "$cargo_bin" build \
    --manifest-path "$source_root/Cargo.toml" \
    --target-dir "$target_dir"
fi

cp "$target_dir/$profile/xfce4-terminal-rs" "$output"

