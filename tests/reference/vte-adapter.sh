#!/bin/sh

# Compares frozen VTE link registration and selection writes with the Rust adapter.

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-vte-adapter.XXXXXX")
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
    xvfb-run --auto-servernum \
    dbus-run-session -- "$probe" > "$report"
}

run_probe "$reference_probe" "$test_root/reference" "$test_root/reference.tsv"
run_probe "$candidate_probe" "$test_root/candidate" "$test_root/candidate.tsv"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"

grep -qx 'initial-highlighted-patterns	0' "$test_root/reference.tsv"
grep -qx 'enabled-patterns	5' "$test_root/reference.tsv"
grep -qx 'primary	user@example.com' "$test_root/reference.tsv"
grep -qx 'clipboard	user@example.com' "$test_root/reference.tsv"
grep -qx 'highlight-disabled	0' "$test_root/reference.tsv"
