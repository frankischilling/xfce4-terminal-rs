#!/bin/sh

set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: $0 REFERENCE_BINARY CANDIDATE_ACCELERATOR_PROBE CALL_PROBE" >&2
  exit 2
fi

reference_binary=$1
candidate_probe=$2
call_probe=$3
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-application-calls.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM

home=$test_root/home
config=$test_root/config
cache=$test_root/cache
accelerator_dir=$config/xfce4/terminal
mkdir -p "$home" "$accelerator_dir" "$cache"
printf '%s\n' \
  '(gtk_accel_path "<Actions>/terminal-window/new-tab" "<Primary><Shift>t")' \
  > "$accelerator_dir/accels.scm"

run_isolated()
{
  output=$1
  shift
  env \
    HOME="$home" \
    XDG_CONFIG_HOME="$config" \
    XDG_CACHE_HOME="$cache" \
    dbus-run-session -- \
    xvfb-run --auto-servernum \
    env LD_PRELOAD="$call_probe" \
    "$@" > "$output"
}

run_isolated "$test_root/reference.tsv" "$reference_binary" --preferences
run_isolated "$test_root/candidate.tsv" "$candidate_probe" "$config"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"
