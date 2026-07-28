#!/bin/sh

set -eu

if [ "$#" -lt 3 ]; then
  echo "usage: $0 REFERENCE_BINARY CANDIDATE_BINARY OUTPUT_DIRECTORY [-- ARGUMENT...]" >&2
  exit 2
fi

reference_binary=$1
candidate_binary=$2
output_dir=$3
shift 3

if [ "${1-}" = "--" ]; then
  shift
fi

if [ ! -x "$reference_binary" ] || [ ! -x "$candidate_binary" ]; then
  echo "reference and candidate must be executable files" >&2
  exit 2
fi

if [ -e "$output_dir" ]; then
  echo "output directory already exists: $output_dir" >&2
  exit 2
fi

mkdir -p "$output_dir"

set +e
LC_ALL=C "$reference_binary" "$@" \
  >"$output_dir/reference.stdout" \
  2>"$output_dir/reference.stderr"
reference_status=$?
LC_ALL=C "$candidate_binary" "$@" \
  >"$output_dir/candidate.stdout" \
  2>"$output_dir/candidate.stderr"
candidate_status=$?
set -e

printf '%s\n' "$reference_status" >"$output_dir/reference.status"
printf '%s\n' "$candidate_status" >"$output_dir/candidate.status"

matched=true
diff -u "$output_dir/reference.status" "$output_dir/candidate.status" \
  >"$output_dir/status.diff" || matched=false
diff -u "$output_dir/reference.stdout" "$output_dir/candidate.stdout" \
  >"$output_dir/stdout.diff" || matched=false
diff -u "$output_dir/reference.stderr" "$output_dir/candidate.stderr" \
  >"$output_dir/stderr.diff" || matched=false

if [ "$matched" = false ]; then
  exit 1
fi
