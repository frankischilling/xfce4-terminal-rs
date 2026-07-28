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

Migration runs only for a new Xfconf channel. It checks these files in order:

1. `$XDG_CONFIG_HOME/xfce4/terminal/terminalrc`
2. `$XDG_CONFIG_HOME/Terminal/terminalrc`

If `XDG_CONFIG_HOME` is unset or relative, the base directory is
`$HOME/.config`.

Values come from the `[Configuration]` group. The old key is the blurb from
the matching C property definition. Strings are copied, `FALSE` is the only
false boolean spelling, unsigned integers use decimal `strtoul`, and doubles
use the C locale with an ASCII fallback. An unrecognized enum becomes the
first value in that enum, matching the C transform.

The older `Terminal/terminalrc` format may contain `ColorPalette1` through
`ColorPalette16`. Migration joins them with semicolons only when all 16 keys
exist.

## Accelerators, colors, and gettext

GTK accelerator maps use
`$XDG_CONFIG_HOME/xfce4/terminal/accels.scm`. The Rust wrapper loads and saves
GTK's native accelerator-map format on the initialized GTK main thread.

Meson installs the eight built-in color schemes under
`share/xfce4/terminal/colorschemes`. The public color reader uses the same
`[Scheme]` key-file format for installed and user schemes. Scheme names remain
gettext inputs through `po/POTFILES`.

The gettext domain is `xfce4-terminal`, the character set is `UTF-8`, and
Meson passes its configured locale directory into the Rust build. A direct
Cargo build uses `/usr/local/share/locale`, which matches Meson's default
prefix.

## Test boundary

`tests/preferences_parity.rs` compares definitions with the frozen C probe.
`tests/xfconf_round_trip.rs` starts a private D-Bus session with an isolated
home and config directory. It checks every default, writes and reads all 94
properties, observes a write from a separate Xfconf client, and exercises old
file migration without touching the user's channel.

`tests/preference_assets.rs` checks the accelerator and gettext paths, reads
all built-in color schemes, and verifies their Meson and gettext inputs.

The installed program remains the C application. These tests cover the
preference contract, but they do not change the final cutover rule.
