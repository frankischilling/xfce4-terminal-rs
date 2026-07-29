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
  raw_output=$output.raw
  shift
  env \
    HOME="$home" \
    XDG_CONFIG_HOME="$config" \
    XDG_CACHE_HOME="$cache" \
    NO_AT_BRIDGE=1 \
    dbus-run-session -- \
    xvfb-run --auto-servernum \
    env LD_PRELOAD="$call_probe" \
    "$@" > "$raw_output"

  awk -F '\t' \
    '$1 == "accelerator" ||
     $1 == "gettext-domain" ||
     $1 == "locale-directory" ||
     $1 == "gettext-charset" ||
     $1 == "accelerator-file"' \
    "$raw_output" | sort > "$output"

  for record in gettext-domain locale-directory gettext-charset accelerator-file; do
    count=$(awk -F '\t' -v record="$record" '$1 == record { count++ } END { print count + 0 }' "$output")
    if [ "$count" -ne 1 ]; then
      echo "expected exactly one $record record, found $count" >&2
      cat "$raw_output" >&2
      exit 1
    fi
  done

  count=$(awk -F '\t' '$1 == "accelerator" { count++ } END { print count + 0 }' "$output")
  if [ "$count" -ne 65 ]; then
    echo "expected 65 accelerator records, found $count" >&2
    cat "$raw_output" >&2
    exit 1
  fi
}

run_isolated "$test_root/reference.tsv" "$reference_binary" --preferences
run_isolated "$test_root/candidate.tsv" \
  env XFCE4_TERMINAL_ACCELERATOR_CONTRACT_ONLY=1 \
  "$candidate_probe" "$config"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"
