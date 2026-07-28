#!/bin/sh

set -eu

repository=/repository
output_dir=/output
source_dir=$output_dir/source
build_dir=$output_dir/build

if [ ! -r "$repository/tests/reference/BASELINE" ]; then
  echo "mount the repository at /repository" >&2
  exit 2
fi

if [ -e "$source_dir" ] || [ -e "$build_dir" ]; then
  echo "reference output already exists under /output" >&2
  exit 2
fi

baseline=$(sed -n '1p' "$repository/tests/reference/BASELINE")
git config --global --add safe.directory "$repository"
git config --global --add safe.directory "$repository/.git"
git clone --quiet --no-checkout "$repository" "$source_dir"
git -C "$source_dir" checkout --quiet --detach "$baseline"
meson setup "$build_dir" "$source_dir" -Ddoc=false
meson compile -C "$build_dir"
"$repository/tests/reference/build-options-probe.sh" \
  "$source_dir" \
  "$build_dir/options-probe"

printf '%s\n' "$build_dir/terminal/xfce4-terminal"
