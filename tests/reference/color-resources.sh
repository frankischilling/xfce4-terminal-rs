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
config_system_first=$test_root/config-system-first
config_system_later=$test_root/config-system-later
relative=xfce4/terminal/colorschemes

mkdir -p \
  "$data_home/$relative/directory.theme" \
  "$data_fallback/$relative" \
  "$config_home/$relative" \
  "$config_system_first/$relative" \
  "$config_system_later/$relative"
printf '%s\n' '[Scheme]' 'Name=Primary' > "$data_home/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Shadowed' > "$data_fallback/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Fallback' > "$data_fallback/$relative/fallback.theme"
printf '%s\n' '[Scheme]' 'Name=Regular fallback' > "$data_fallback/$relative/directory.theme"
printf '%s\n' '[Scheme]' 'Name=Symlink primary' > "$test_root/symlink-source.theme"
ln -s "$test_root/symlink-source.theme" "$data_home/$relative/symlink.theme"
printf '%s\n' '[Scheme]' 'Name=Shadowed symlink' > "$data_fallback/$relative/symlink.theme"
printf '%s\n' '[Scheme]' 'Name=User copy' > "$config_home/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Shadowed user copy' > "$config_system_first/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=Later shadowed user copy' > "$config_system_later/$relative/shared.theme"
printf '%s\n' '[Scheme]' 'Name=First system copy' > "$config_system_first/$relative/system-shared.theme"
printf '%s\n' '[Scheme]' 'Name=Later shadowed system copy' > "$config_system_later/$relative/system-shared.theme"
printf '%s\n' '[Scheme]' 'Name=Later system config' > "$config_system_later/$relative/system-later.theme"

run_probe()
{
  output=$1
  probe=$2
  probe_data_home=$3
  probe_data_dirs=$4
  probe_config_home=$5
  probe_config_dirs=$6
  filter_root=$7
  env \
    XDG_DATA_HOME="$probe_data_home" \
    XDG_DATA_DIRS="$probe_data_dirs" \
    XDG_CONFIG_HOME="$probe_config_home" \
    XDG_CONFIG_DIRS="$probe_config_dirs" \
    "$probe" \
    | awk -F '\t' -v root="$filter_root/" 'index($2, root) == 1' \
    | sort > "$output"
}

run_probe "$test_root/reference.tsv" "$reference_probe" \
  "$data_home" "$data_fallback" "$config_home" "$config_system_first:$config_system_later" "$test_root"
run_probe "$test_root/candidate.tsv" "$candidate_probe" \
  "$data_home" "$data_fallback" "$config_home" "$config_system_first:$config_system_later" "$test_root"
diff -u "$test_root/reference.tsv" "$test_root/candidate.tsv"

shared_root=$test_root/shared-root
mkdir -p "$shared_root/$relative"
printf '%s\n' '[Scheme]' 'Name=Shared root' > "$shared_root/$relative/shared.theme"

run_probe "$test_root/reference-shared.tsv" "$reference_probe" \
  "$shared_root" "$data_fallback" "$shared_root" "$config_system_first:$config_system_later" "$shared_root"
run_probe "$test_root/candidate-shared.tsv" "$candidate_probe" \
  "$shared_root" "$data_fallback" "$shared_root" "$config_system_first:$config_system_later" "$shared_root"
diff -u "$test_root/reference-shared.tsv" "$test_root/candidate-shared.tsv"
