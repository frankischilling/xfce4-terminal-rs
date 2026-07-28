# Command-line behavior

The Rust candidate keeps the `xfce4-terminal` command-line contract even
though its development binary is named `xfce4-terminal-rs`.

## Immediate commands

The following commands return without starting a terminal window:

- `-h` and `--help` print the option summary.
- `-V` and `--version` print the application version, native Xfce version,
  credits, and bug tracker.
- `--color-table` prints the same ANSI foreground and background table as the
  C application.

These outputs are compared byte for byte with the frozen C executable under
the C locale.

## Windows and tabs

Arguments before the first separator describe the first window and its first
tab. `--tab` adds a tab to the current window. `--window` starts another
window. When an existing window can be reused, the first `--tab` targets that
window and the first leading `--window` is ignored, matching the server-side C
parser.

`--default-display` and `--default-working-directory` are applied after all
windows and tabs have been parsed. A value set on a specific window or tab
takes precedence over its default.

## Commands and option values

`-x` or `--execute` assigns the rest of the argument list to the current tab
without shell parsing. The short form may end a group, as in `-Hx`, and must be
followed by at least one argument. Text after `x` in the same group is an
error.

`-e` or `--command` accepts one string and parses it with GLib shell syntax.
This preserves quoting and escaping behavior used by the C application.

Short boolean options may be grouped. A short option that takes a value uses
the rest of its group as that value, or consumes the next argument when no
text remains.

On Unix, argument values remain native OS strings. Invalid UTF-8 bytes are
preserved in commands, titles, paths, display names, identifiers, icons, and
fonts, including in the differential probe output.

## Parity probe

`xfce4-terminal-options-probe` is a test-only binary. It serializes parsed
window and tab specifications in the same format as
`tests/reference/options-probe.c`. CI builds the C probe against the frozen
reference source and compares both programs across successful and failing
argument sets.

The C probe calls `terminal_window_attr_parse`, which is declared in
`terminal-options.h`. That function is the parser module's public boundary;
the probe does not inspect private GTK widget state.

The probe is not installed and is not part of the user-facing command line.
