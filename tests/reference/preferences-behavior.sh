#!/bin/sh

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-preferences.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM

write_legacy_file()
{
  root=$1
  legacy_dir=$root/config/Terminal
  mkdir -p "$legacy_dir"
  {
    printf '%s\n' \
      '[Configuration]' \
      'MiscBell=FALSE' \
      'ScrollingLines=1234' \
      'CellWidthScale=1.25' \
      'TitleInitial=Legacy title' \
      'ScrollingBar=TERMINAL_SCROLLBAR_LEFT'
    index=1
    while [ "$index" -le 15 ]; do
      printf 'ColorPalette%s=#%06x\n' "$index" "$index"
      index=$((index + 1))
    done
  } > "$legacy_dir/terminalrc"
}

write_invalid_legacy_file()
{
  root=$1
  legacy_dir=$root/config/Terminal
  mkdir -p "$legacy_dir"
  {
    printf '%s\n' \
      '[Configuration]' \
      'ScrollingLines=999999999' \
      'TitleInitial=After invalid value'
  } > "$legacy_dir/terminalrc"
}

write_unreadable_legacy_file()
{
  root=$1
  legacy_dir=$root/config/Terminal
  mkdir -p "$legacy_dir"
  printf '%s\n' 'this is not a key file' > "$legacy_dir/terminalrc"
}

write_all_legacy_file()
{
  root=$1
  contract=$2
  legacy_dir=$root/config/Terminal
  mkdir -p "$legacy_dir"
  awk -F '\t' '
    BEGIN {
      print "[Configuration]"
    }
    {
      if ($2 == "boolean")
        value = $3 == "true" ? "FALSE" : "TRUE"
      else if ($2 == "string")
        value = $5 == "Encoding" ? "UTF-8" : "legacy:" $1
      else if ($2 == "uint" || $2 == "double")
        {
          split($4, bounds, ":")
          value = $3 == bounds[1] ? bounds[2] : bounds[1]
        }
      else
        {
          sub(/^enum:/, "", $2)
          count = split($4, values, ",")
          value = values[1]
          for (item = 1; item <= count; item++)
            if (values[item] != $3)
              {
                value = values[item]
                break
              }
        }
      print $5 "=" value
    }
  ' "$contract" > "$legacy_dir/terminalrc"
}

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
    dbus-run-session -- "$probe" --values > "$output"
}

run_string_typed_probe()
{
  probe=$1
  root=$2
  output=$3
  mkdir -p "$root/home" "$root/config" "$root/cache"
  env \
    HOME="$root/home" \
    XDG_CONFIG_HOME="$root/config" \
    XDG_CACHE_HOME="$root/cache" \
    dbus-run-session -- sh -c '
        xfconf-query -c xfce4-terminal -p /misc-bell -n -t string -s TRUE
        xfconf-query -c xfce4-terminal -p /scrolling-lines -n -t string -s 2345
        xfconf-query -c xfce4-terminal -p /cell-width-scale -n -t string -s 1.75
        xfconf-query -c xfce4-terminal -p /scrolling-bar -n -t string -s INVALID
        exec "$1" --values
      ' sh "$probe" > "$output"
}

run_uint_typed_string_probe()
{
  probe=$1
  root=$2
  output=$3
  mkdir -p "$root/home" "$root/config" "$root/cache"
  env \
    HOME="$root/home" \
    XDG_CONFIG_HOME="$root/config" \
    XDG_CACHE_HOME="$root/cache" \
    dbus-run-session -- sh -c '
        xfconf-query -c xfce4-terminal -p /title-initial -n -t uint -s 42
        exec "$1" --values
      ' sh "$probe" > "$output"
}

run_probe "$reference_probe" "$test_root/reference-default" \
  "$test_root/reference-default.tsv"
run_probe "$candidate_probe" "$test_root/candidate-default" \
  "$test_root/candidate-default.tsv"
diff -u "$test_root/reference-default.tsv" "$test_root/candidate-default.tsv"

"$reference_probe" > "$test_root/preferences-contract.tsv"
mkdir -p "$test_root/reference-all-mappings" "$test_root/candidate-all-mappings"
write_all_legacy_file "$test_root/reference-all-mappings" \
  "$test_root/preferences-contract.tsv"
write_all_legacy_file "$test_root/candidate-all-mappings" \
  "$test_root/preferences-contract.tsv"
run_probe "$reference_probe" "$test_root/reference-all-mappings" \
  "$test_root/reference-all-mappings.tsv"
run_probe "$candidate_probe" "$test_root/candidate-all-mappings" \
  "$test_root/candidate-all-mappings.tsv"
diff -u "$test_root/reference-all-mappings.tsv" \
  "$test_root/candidate-all-mappings.tsv"

mkdir -p "$test_root/reference-migration" "$test_root/candidate-migration"
write_legacy_file "$test_root/reference-migration"
write_legacy_file "$test_root/candidate-migration"
run_probe "$reference_probe" "$test_root/reference-migration" \
  "$test_root/reference-migration.tsv"
run_probe "$candidate_probe" "$test_root/candidate-migration" \
  "$test_root/candidate-migration.tsv"
diff -u "$test_root/reference-migration.tsv" "$test_root/candidate-migration.tsv"

mkdir -p "$test_root/reference-invalid" "$test_root/candidate-invalid"
write_invalid_legacy_file "$test_root/reference-invalid"
write_invalid_legacy_file "$test_root/candidate-invalid"
run_probe "$reference_probe" "$test_root/reference-invalid" \
  "$test_root/reference-invalid.tsv"
run_probe "$candidate_probe" "$test_root/candidate-invalid" \
  "$test_root/candidate-invalid.tsv"
diff -u "$test_root/reference-invalid.tsv" "$test_root/candidate-invalid.tsv"

mkdir -p "$test_root/reference-unreadable" "$test_root/candidate-unreadable"
write_unreadable_legacy_file "$test_root/reference-unreadable"
write_unreadable_legacy_file "$test_root/candidate-unreadable"
run_probe "$reference_probe" "$test_root/reference-unreadable" \
  "$test_root/reference-unreadable.tsv"
run_probe "$candidate_probe" "$test_root/candidate-unreadable" \
  "$test_root/candidate-unreadable.tsv"
diff -u "$test_root/reference-unreadable.tsv" \
  "$test_root/candidate-unreadable.tsv"

run_string_typed_probe "$reference_probe" "$test_root/reference-strings" \
  "$test_root/reference-strings.tsv"
run_string_typed_probe "$candidate_probe" "$test_root/candidate-strings" \
  "$test_root/candidate-strings.tsv"
diff -u "$test_root/reference-strings.tsv" "$test_root/candidate-strings.tsv"

run_uint_typed_string_probe "$reference_probe" "$test_root/reference-uint-string" \
  "$test_root/reference-uint-string.tsv"
run_uint_typed_string_probe "$candidate_probe" "$test_root/candidate-uint-string" \
  "$test_root/candidate-uint-string.tsv"
diff -u "$test_root/reference-uint-string.tsv" \
  "$test_root/candidate-uint-string.tsv"
