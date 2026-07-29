#!/bin/sh

# Compares the spawn request of the frozen C screen with the Rust candidate.
#
# Both probes read the same scenario corpus and write the command, the argument
# vector, the spawn flags, and the child environment that each scenario
# produces. The corpus runs once per login shell arrangement, because which
# shell a screen starts depends on the environment, on the password database,
# and on which files are executable.
#
# The frozen probe needs a display, since it builds a real screen and realizes a
# real window, and it needs a session bus for its preference channel. Both
# probes have to see the same display, or the environment they report would
# differ over the display alone, so the caller provides one display for the
# whole comparison. Each probe still gets its own session bus, and the address
# of that bus is the one value the reports are allowed to disagree about.

set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: $0 REFERENCE_PROBE CANDIDATE_PROBE SHIM_LIBRARY" >&2
  exit 2
fi

reference_probe=$1
candidate_probe=$2
shim=$3
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
fixtures=$script_dir/child-fixtures.txt

if [ -z "${DISPLAY:-}" ] || [ -z "${XAUTHORITY:-}" ]; then
  echo "$0: run this under a display, for instance with xvfb-run" >&2
  exit 2
fi

test_root=$(mktemp -d "${TMPDIR:-/tmp}/xfce4-terminal-child.XXXXXX")
trap 'rm -rf "$test_root"' EXIT HUP INT TERM
home=$test_root/home
config=$test_root/config
cache=$test_root/cache
toplevel=$test_root/toplevel.tsv
mkdir -p "$home" "$cache"

# A probe that stops making progress would otherwise hold the whole job until
# its own limit ran out, with nothing in the log to say which run stalled. A run
# takes well under a second once the caches a terminal widget needs are warm.
limit=120

# Both probes are started from the same directory and through the same wrapper,
# so the variables a shell adds for them, such as PWD and SHLVL, agree.
run_probe()
{
  probe=$1
  report=$2
  shift 2
  # A fresh configuration directory, together with the session bus each probe
  # gets of its own, gives every run an empty preference channel. The cache
  # directory is kept, because nothing under test is stored there and a cold one
  # makes every run rebuild the font caches that realizing a terminal needs.
  rm -rf "$config"
  mkdir -p "$config"
  timeout "$limit" \
    env -i \
    PATH="$PATH" \
    HOME="$home" \
    XDG_CONFIG_HOME="$config" \
    XDG_CACHE_HOME="$cache" \
    DISPLAY="$DISPLAY" \
    XAUTHORITY="$XAUTHORITY" \
    LC_ALL=C \
    NO_AT_BRIDGE=1 \
    GIO_USE_VFS=local \
    LD_PRELOAD="$shim" \
    "$@" \
    dbus-run-session -- "$probe" "$fixtures" "$report" "$toplevel"
}

# A session bus address names one run of one probe, so the reports record that
# the variable was passed on without comparing which bus it pointed at.
normalize()
{
  sed 's/DBUS_SESSION_BUS_ADDRESS=.*$/DBUS_SESSION_BUS_ADDRESS=<address>/' "$1"
}

compare()
{
  name=$1
  shift
  # Naming the arrangement before it runs is what tells a reader of the log
  # which of them a failure or a timeout belongs to.
  echo "comparing the $name arrangement"
  if ! run_probe "$reference_probe" "$test_root/$name-reference.raw" "$@"; then
    echo "$0: the frozen probe failed or ran past $limit seconds ($name)" >&2
    exit 1
  fi
  if ! run_probe "$candidate_probe" "$test_root/$name-candidate.raw" "$@"; then
    echo "$0: the candidate failed or ran past $limit seconds ($name)" >&2
    exit 1
  fi
  normalize "$test_root/$name-reference.raw" > "$test_root/$name-reference.tsv"
  normalize "$test_root/$name-candidate.raw" > "$test_root/$name-candidate.tsv"
  diff -u "$test_root/$name-reference.tsv" "$test_root/$name-candidate.tsv"
}

# Every path of the shell search, in the order the reference walks it: a shell
# named by the environment, one recorded in the password database, the built-in
# list, and a host that offers nothing at all.
compare environment-shell SHELL=/bin/sh
compare unset-shell
compare empty-shell SHELL=
compare unusable-shell SHELL=/nonexistent/shell
compare password-shell XFCE4_TERMINAL_PROBE_PW_SHELL=/bin/sh
compare no-password-shell \
  XFCE4_TERMINAL_PROBE_PW_SHELL=
compare fallback-shell \
  XFCE4_TERMINAL_PROBE_PW_SHELL=/nonexistent/shell
compare later-fallback-shell \
  XFCE4_TERMINAL_PROBE_PW_SHELL=/nonexistent/shell \
  XFCE4_TERMINAL_PROBE_DENY_EXEC=/bin/sh
compare no-shell-at-all \
  XFCE4_TERMINAL_PROBE_PW_SHELL=/nonexistent/shell \
  XFCE4_TERMINAL_PROBE_DENY_EXEC=/bin/sh:/bin/bash:/usr/bin/bash:/bin/dash:/usr/bin/dash:/bin/zsh:/usr/bin/zsh:/bin/tcsh:/usr/bin/tcsh:/bin/csh:/usr/bin/csh:/bin/ksh:/usr/bin/ksh

# Both probes decide for themselves which lines of the corpus are scenarios, so
# a report has to account for every one of them. Dropping the same lines on both
# sides would otherwise leave the comparison agreeing about less than it claims.
scenarios=$(grep -cv '^#' "$fixtures")
reported=$(grep -c '^scenario	' "$test_root/environment-shell-reference.tsv")
if [ "$reported" -ne "$scenarios" ]; then
  echo "$0: reported $reported of $scenarios scenarios" >&2
  exit 1
fi

# A corpus whose scenarios all failed, or all succeeded, would let both sides
# agree without deciding anything.
base=$test_root/environment-shell-reference.tsv
grep -q '^error	.*	Empty custom command in the terminal preferences$' "$base"
grep -q '^error	.*	Text ended before matching quote' "$base"
grep -q '^error	.*	Text ended just after a ' "$base"
grep -q '^spawn-flags	.*	68$' "$base"
grep -q '^constant	pty-flags	0$' "$base"
grep -q '^constant	spawn-timeout	30000$' "$base"
grep -q '^variable	.*	COLORTERM=xfce4-terminal$' "$base"
grep -q '^toplevel	.*	x11	[0-9]' "$base"
grep -q '^variable	.*	WINDOWID=[0-9]' "$base"
grep -q '^variable	.*	DISPLAY=' "$base"
# A screen with no realized toplevel learns of no window and no display.
plain=$(sed -n 's/^scenario	\([0-9]*\)	environment	plain$/\1/p' "$base")
if grep -q "^variable	$plain	.*	WINDOWID=" "$base"; then
  echo "$0: an unrealized screen reported a window" >&2
  exit 1
fi

# The first scenario asks for a plain login shell, so its command is whatever
# the search settled on. Each run has to settle on the answer its arrangement
# leaves, or the run below it would prove nothing new.
shell_scenario=$(
  sed -n 's/^scenario	\([0-9]*\)	command	false	false	$/\1/p' "$base" | sed -n 1p
)
test -n "$shell_scenario"

chosen()
{
  sed -n "s/^argument	$shell_scenario	0	//p" "$test_root/$1-reference.tsv"
}
test "$(chosen environment-shell)" = /bin/sh
test "$(chosen unusable-shell)" = "$(chosen unset-shell)"
test "$(chosen empty-shell)" = "$(chosen unset-shell)"
test "$(chosen password-shell)" = /bin/sh
test "$(chosen fallback-shell)" = /bin/sh
test "$(chosen later-fallback-shell)" != /bin/sh
test -n "$(chosen later-fallback-shell)"
grep -q "^error	$shell_scenario	Unable to determine your login shell.\$" \
  "$test_root/no-shell-at-all-reference.tsv"
