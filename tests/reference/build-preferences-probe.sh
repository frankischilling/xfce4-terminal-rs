#!/bin/sh

set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: $0 REFERENCE_SOURCE REFERENCE_BUILD OUTPUT_FILE" >&2
  exit 2
fi

reference_source=$1
reference_build=$2
output=$3
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)

cc \
  -I"$reference_build/terminal" \
  -I"$reference_source/terminal" \
  -I"$reference_source" \
  -DHAVE_LIMITS_H=1 \
  -DHAVE_MEMORY_H=1 \
  -DHAVE_STRING_H=1 \
  -DPACKAGE_NAME=\"xfce4-terminal\" \
  -DGETTEXT_PACKAGE=\"xfce4-terminal\" \
  "$repo_root/tests/reference/preferences-probe.c" \
  "$reference_build/terminal/terminal-enum-types.c" \
  "$reference_source/terminal/terminal-preferences.c" \
  -o "$output" \
  $(pkg-config --cflags --libs \
    gtk+-3.0 \
    libpcre2-8 \
    libxfce4ui-2 \
    libxfconf-0 \
    vte-2.91)
