#!/bin/sh

set -eu

cargo_bin=$1
source_root=$2
target_dir=$3
output=$4
profile=$5
features=$6
debug_symbols=$7
locale_dir=$8

feature_args=
if [ -n "$features" ]; then
  feature_args="--features=$features"
fi

if [ "$profile" = "release" ]; then
  if [ "$debug_symbols" = "true" ]; then
    CARGO_PROFILE_RELEASE_DEBUG=true \
    CARGO_PROFILE_RELEASE_STRIP=false \
    XFCE4_TERMINAL_LOCALE_DIR="$locale_dir" \
      "$cargo_bin" build \
        --manifest-path "$source_root/Cargo.toml" \
        --target-dir "$target_dir" \
        --no-default-features \
        $feature_args \
        --release
  else
    XFCE4_TERMINAL_LOCALE_DIR="$locale_dir" \
      "$cargo_bin" build \
      --manifest-path "$source_root/Cargo.toml" \
      --target-dir "$target_dir" \
      --no-default-features \
      $feature_args \
      --release
  fi
else
  XFCE4_TERMINAL_LOCALE_DIR="$locale_dir" \
    "$cargo_bin" build \
    --manifest-path "$source_root/Cargo.toml" \
    --target-dir "$target_dir" \
    --no-default-features \
    $feature_args
fi

cp "$target_dir/$profile/xfce4-terminal-rs" "$output"
