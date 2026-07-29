#!/bin/sh

# Compares the link contract of the frozen C widget with the Rust candidate.
#
# Both probes read the same candidate corpus and print the registered patterns,
# the classification of every candidate, whether it may be opened, the URI it
# opens with, and the text copying it produces. The frozen probe needs a display
# and a session bus because it builds a real widget and reads a real clipboard.

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-links.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM

# The clickable test for a file URI compares against the local host name, which
# only this machine knows.
fixtures=$test_root/fixtures.txt
cp "$script_dir/link-fixtures.txt" "$fixtures"
printf 'file://%s/tmp/example\n' "$(uname -n)" >> "$fixtures"

run_probe()
{
  probe=$1
  root=$2
  output=$3
  mkdir -p "$root/home" "$root/config" "$root/cache"
  env \
    HOME="$root/home" \
    XDG_CONFIG_HOME="$root/config" \
    XDG_CACHE_HOME="$root/cache" \
    LC_ALL=C \
    NO_AT_BRIDGE=1 \
    xvfb-run --auto-servernum \
    dbus-run-session -- "$probe" "$fixtures" > "$output"
}

run_probe "$reference_probe" "$test_root/reference" "$test_root/reference.tsv"
run_probe "$candidate_probe" "$test_root/candidate" "$test_root/candidate.tsv"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"

# A corpus that classified nothing would let both sides agree by accident.
grep -q '	full-http$' "$test_root/reference.tsv"
grep -q '	http$' "$test_root/reference.tsv"
grep -q '	email$' "$test_root/reference.tsv"
grep -q '	file$' "$test_root/reference.tsv"
grep -q '	none$' "$test_root/reference.tsv"
grep -q '^clickable	file://.*	false$' "$test_root/reference.tsv"
