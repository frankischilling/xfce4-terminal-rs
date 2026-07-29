# Preferences

Xfce Terminal stores settings in the `xfce4-terminal` Xfconf channel. Each
property uses its GObject name with a leading slash. For example,
`misc-bell` is stored as `/misc-bell`.

## Preference definitions

`src/preferences.rs` exposes the 94 properties registered by the frozen C
`TerminalPreferences` class. A definition contains:

- the Xfconf name
- the storage type and any numeric bounds
- the default value
- the old `terminalrc` key
- the GType name and accepted constants for enum properties

Enums use their full C constant names in Xfconf, such as
`TERMINAL_SCROLLBAR_RIGHT`. A missing string default remains distinct from an
empty string.

The checked contract lives in
`tests/reference/preferences-contract.tsv`. The frozen-reference build
compiles `tests/reference/preferences-probe.c`, which reads the GObject class
metadata instead of parsing the C source. Protected CI compares that output
with the Rust contract.

## Xfconf behavior

`Preferences` initializes libxfconf and keeps its channel pointer inside a
small FFI wrapper. Reads return the C default when Xfconf has no stored value.
Writes reject the wrong type, out-of-range numbers, non-finite doubles, and
unknown enum constants before calling libxfconf.

Libxfconf caches channel values. An update from another process becomes
visible after the GLib main context dispatches the channel's
`property-changed` signal. GTK controllers must therefore use `Preferences`
on the main thread and continue running the normal GTK main loop.

## Legacy migration

Opening `Preferences` runs migration when the Xfconf channel is new. It checks
these files in order:

1. `$XDG_CONFIG_HOME/xfce4/terminal/terminalrc`
2. `$XDG_CONFIG_HOME/Terminal/terminalrc`

If `XDG_CONFIG_HOME` is unset or relative, the base directory is
`$HOME/.config`.

Values come from the `[Configuration]` group. The old key is the blurb from
the matching C property definition. Strings are copied, `FALSE` is the only
false boolean spelling, unsigned integers use decimal `strtoul`, and doubles
use the C locale with an ASCII fallback. An unrecognized enum becomes the
first value in that enum, matching the C transform.

Migration is best effort. An unreadable file leaves the defaults in place. An
invalid value is skipped without preventing later keys from being imported.
The normal `Preferences::set` API still rejects invalid types and values.

The older `Terminal/terminalrc` format may contain `ColorPalette1` through
`ColorPalette16`. The frozen C loop also accepts keys 1 through 15 followed by
a missing key 16 and leaves a trailing semicolon. The Rust migration preserves
that edge case.

## Accelerators, colors, and gettext

GTK accelerator maps use
`$XDG_CONFIG_HOME/xfce4/terminal/accels.scm`. The Rust wrapper loads and saves
GTK's native accelerator-map format on the initialized GTK main thread.

Meson installs the eight built-in color schemes under
`share/xfce4/terminal/colorschemes`. The public color reader combines installed
and user scheme directories, accepts any preset filename, and skips files that
cannot provide a name. It uses localized names from the `[Scheme]` key-file
group and sorts the result by name.

The gettext domain is `xfce4-terminal`, the character set is `UTF-8`, and
Meson passes its configured locale directory into the Rust build. A direct
Cargo build uses `/usr/local/share/locale`, which matches Meson's default
prefix.

## Test boundary

`tests/preferences_parity.rs` compares definitions with the frozen C probe.
`tests/reference/preferences-behavior.sh` runs the C and Rust probes on
separate private buses and compares their live defaults, valid and invalid
migration cases, and string-typed compatibility values. The isolated
environment is applied before each private bus starts, so the activated
Xfconf daemon uses the test home and configuration directories.
`tests/xfconf_round_trip.rs` checks every Rust default, writes and reads all 94
properties, observes a write from a separate Xfconf client, and exercises old
file migration without touching the user's channel.

`tests/accelerator_round_trip.rs` saves and loads GTK's accelerator map under
Xvfb and proves that loading restores a changed shortcut.
`tests/reference/application-calls.sh` observes the frozen application and the
Rust probe as separate processes. It compares the gettext domain, locale
directory, character set, and accelerator filename passed to their native
libraries. `tests/preference_assets.rs` reads all built-in schemes and combines
installed and user schemes. Protected CI also compares the installed color
schemes and gettext catalogs from the frozen and candidate Meson builds.

The installed program remains the C application. These tests cover the
preference contract, but they do not change the final cutover rule.
