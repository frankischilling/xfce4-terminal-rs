#!/bin/sh

# Compares the screen model of the frozen C screen with the Rust candidate.
#
# Both probes read the same scenario corpus and write titles, paste-safety
# decisions, working directories, and the colors handed to VTE. The frozen probe
# needs a display and a session bus because it builds a real screen. Each probe
# writes to a named file, so the messages those wrappers print cannot reach the
# compared report.

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
fixtures=$script_dir/screen-fixtures.txt
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-screen.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM

run_probe()
{
  probe=$1
  root=$2
  report=$3
  mkdir -p "$root/home" "$root/config" "$root/cache"
  env \
    HOME="$root/home" \
    XDG_CONFIG_HOME="$root/config" \
    XDG_CACHE_HOME="$root/cache" \
    LC_ALL=C \
    NO_AT_BRIDGE=1 \
    GIO_USE_VFS=local \
    xvfb-run --auto-servernum \
    dbus-run-session -- "$probe" "$fixtures" "$report"
}

run_probe "$reference_probe" "$test_root/reference" "$test_root/reference.tsv"
run_probe "$candidate_probe" "$test_root/candidate" "$test_root/candidate.tsv"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"

# Both probes decide for themselves which lines of the corpus are scenarios, so
# a report has to account for every one of them.
scenarios=$(grep -cE '^(title-parse|title|paste|cwd|colors)	' "$fixtures")
reported=$(grep -cE '^(title-parse|title|paste|cwd|colors)	' "$test_root/reference.tsv")
if [ "$reported" -ne "$scenarios" ]; then
  echo "$0: reported $reported of $scenarios scenarios" >&2
  exit 1
fi

grep -q '^title-parse	.*Untitled$' "$test_root/reference.tsv"
grep -q '^paste	.*	unsafe$' "$test_root/reference.tsv"
grep -q '^paste	.*	safe$' "$test_root/reference.tsv"
grep -q '^cwd	.*	/tmp/from-uri$' "$test_root/reference.tsv"
grep -q '^colors	.*	palette$' "$test_root/reference.tsv"
grep -q '^colors	.*	default$' "$test_root/reference.tsv"
