# Screen model

A terminal screen decides several things before it needs a running child: the
title shown on the tab, the working directory a new tab would inherit, whether
pasted text needs a confirmation dialog, and the colors handed to VTE.

`src/screen.rs` owns that model. The widget layer supplies the values only a
realized terminal knows — the window title VTE currently reports, a directory
URI, a process cwd, and any per-tab color overrides — and the functions here
turn those values into the same answers the frozen C helpers produce.

## Titles

`parse_title` walks a template from percent sign to percent sign:

| Sequence | Expansion |
| --- | --- |
| `%#` | the screen's session id |
| `%d` | the basename of the working directory |
| `%D` | the full working directory |
| `%w` | the VTE window title, or `Untitled` under `LC_ALL=C` |
| anything else after `%` | the percent is kept and the next character is read again |

A missing template becomes an empty string. `screen_title` then applies the
dynamic title mode: a custom title wins outright; otherwise replace, prepend,
append, or hide combine the parsed initial title with the window title the same
way `terminal_screen_get_title` does.

## Working directory

`resolve_working_directory` mirrors the frozen getter once the widget layer has
asked VTE and the process table. A directory URI that `g_filename_from_uri`
accepts replaces the stored value. A process cwd is used only when no URI is
present. When both are missing, the stored directory is kept.

## Paste safety

`is_text_unsafe` is true when the text contains a newline or a carriage return.
The confirmation dialog itself belongs to the window layer; this predicate is
the gate that dialog uses.

## Colors

`resolve_colors` computes the arguments `terminal_screen_update_colors` would
pass to VTE for one preference and tab color set. Fixtures keep
`color-background-vary` and `color-use-theme` off so the result stays
deterministic. An unparseable palette falls back to VTE's default colors. Cursor
and selection colors are emitted only when their "use default" preferences are
off. On VTE 0.52 and newer, the bold color is always set, including an explicit
null when the preference asks for the default.

Random hue variation and theme-derived colors remain for a later slice that can
feed a controlled style context into the comparison.

## Test boundary

`tests/screen_model.rs` keeps the interesting cases readable without a
reference build. `tests/reference/screen-probe.c` includes the frozen screen
implementation and writes the same report the Rust probe produces.
`tests/reference/screen-model.sh` compares the two under a private bus and a
display.

The frozen probe replaces the VTE getters for the window title and the
directory URI, and replaces the VTE color setters, so the report records the
values the frozen helpers would pass without depending on VTE to echo them
back. Modern VTE no longer accepts the OSC sequences that used to set those
properties, which is why the getters are interposed rather than fed.
