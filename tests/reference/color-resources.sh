#!/bin/sh

set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-colors.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM

data_home=$test_root/data-home
data_fallback=$test_root/data-fallback
config_home=$test_root/config-home
config_fallback=$test_root/config-fallback
relative=xfce4/terminal/colorschemes

mkdir -p \
  "$data_home/$relative/directory.theme" \
  "$data_fallback/$relative" \
  "$config_home/$relative" \
  "$config_fallback/$relative"
printf '%s\n' '[Scheme]' 'Name=Primary' > "$data_home/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Shadowed' > "$data_fallback/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Fallback' > "$data_fallback/$relative/fallback.theme"
printf '%s\n' '[Scheme]' 'Name=Regular fallback' > "$data_fallback/$relative/directory.theme"
printf '%s\n' '[Scheme]' 'Name=Symlink primary' > "$test_root/symlink-source.theme"
ln -s "$test_root/symlink-source.theme" "$data_home/$relative/symlink.theme"
printf '%s\n' '[Scheme]' 'Name=Shadowed symlink' > "$data_fallback/$relative/symlink.theme"
printf '%s\n' '[Scheme]' 'Name=User copy' > "$config_home/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Shadowed system copy' > "$config_fallback/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=System config' > "$config_fallback/$relative/system.theme"

run_probe()
{
  output=$1
  probe=$2
  env \
    XDG_DATA_HOME="$data_home" \
    XDG_DATA_DIRS="$data_fallback" \
    XDG_CONFIG_HOME="$config_home" \
    XDG_CONFIG_DIRS="$config_fallback" \
    "$probe" \
    | awk -F '\t' -v root="$test_root/" 'index($2, root) == 1' \
    | sort > "$output"
}

run_probe "$test_root/reference.tsv" "$reference_probe"
run_probe "$test_root/candidate.tsv" "$candidate_probe"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"

shared_root=$test_root/shared-root
mkdir -p "$shared_root/$relative"
printf '%s\n' '[Scheme]' 'Name=Shared root' > "$shared_root/$relative/shared.theme"

run_shared_probe()
{
  output=$1
  probe=$2
  env \
    XDG_DATA_HOME="$shared_root" \
    XDG_DATA_DIRS="$data_fallback" \
    XDG_CONFIG_HOME="$shared_root" \
    XDG_CONFIG_DIRS="$config_fallback" \
    "$probe" \
    | awk -F '\t' -v root="$shared_root/" 'index($2, root) == 1' \
    | sort > "$output"
}

run_shared_probe "$test_root/reference-shared.tsv" "$reference_probe"
run_shared_probe "$test_root/candidate-shared.tsv" "$candidate_probe"
diff -u "$test_root/reference-shared.tsv" "$test_root/candidate-shared.tsv"
