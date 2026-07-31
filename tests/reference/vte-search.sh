#!/bin/sh

# Compares frozen VTE screen-search state with the Rust adapter.

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-vte-search.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM

run_probe()
{
  probe=$1
  root=$2
  report=$3
  raw_report=$report.raw
  mkdir -p "$root/home" "$root/config" "$root/cache"
  env \
    HOME="$root/home" \
    XDG_CONFIG_HOME="$root/config" \
    XDG_CACHE_HOME="$root/cache" \
    LC_ALL=C \
    NO_AT_BRIDGE=1 \
    xvfb-run --auto-servernum \
    dbus-run-session -- "$probe" > "$raw_report"

  awk -F '\t' '
    $1 == "initial" ||
    $1 == "configured" ||
    $1 == "moves" ||
    $1 == "reset-keeps" ||
    $1 == "reset-clears" ||
    $1 == "explicit-clear" { print }
  ' "$raw_report" > "$report"
  test "$(wc -l < "$report")" -eq 6
}

run_probe "$reference_probe" "$test_root/reference" "$test_root/reference.tsv"
run_probe "$candidate_probe" "$test_root/candidate" "$test_root/candidate.tsv"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"

grep -qx 'initial	false	false' "$test_root/reference.tsv"
grep -qx 'configured	true	true' "$test_root/reference.tsv"
grep -qx 'moves	called' "$test_root/reference.tsv"
grep -qx 'reset-keeps	true	true' "$test_root/reference.tsv"
grep -qx 'reset-clears	false	true' "$test_root/reference.tsv"
grep -qx 'explicit-clear	false	false' "$test_root/reference.tsv"
