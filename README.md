[![License](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://gitlab.xfce.org/apps/xfce4-terminal/-/blob/master/COPYING)

# xfce4-terminal-rs

This is an unofficial Rust port of Xfce Terminal. The goal is to replace the
application code without changing how the terminal looks, starts commands,
stores settings, talks over D-Bus, or integrates with X11 and Wayland.

The port is still in progress. The C program at commit
`b5933b80d28ca35f873df8da2998e23be5f4e104` is the reference, and the installed
binary remains that implementation until the parity suite passes.

Xfce Terminal uses GTK 3 and VTE. It supports normal and drop-down windows,
tabs, search, configurable shortcuts, large scrollback buffers, Unicode,
custom fonts and colors, transparent or image backgrounds, and command-line
window construction.

## Building

Install Rust 1.85 or newer and the native dependencies listed in
`meson.build`. Then run:

```sh
meson setup build
meson compile -C build
meson test -C build --print-errorlogs
```

The build currently produces two programs:

```text
build/terminal/xfce4-terminal
build/rust/xfce4-terminal-rs
```

The first is the C reference. The second is the Rust candidate and is not
installed yet.

For the covered C-locale cases, the candidate matches the frozen command-line
help, version output, ANSI color table, and window or tab launch parser. It
does not create terminal windows yet.

Rust tests can also run directly:

```sh
cargo test --all-targets --all-features --locked
```

## Documentation

- `docs/PARITY.md` records the behavior that must match before cutover.
- `docs/ARCHITECTURE.md` describes the build, Rust, and compatibility
  boundaries.
- `docs/CLI.md` documents command-line behavior and its differential probe.
- `HACKING` explains the issue and pull request workflow.
- The existing XML manual under `doc/` remains the source for the
  `xfce4-terminal` man page.

The original Xfce documentation is available at
[docs.xfce.org](https://docs.xfce.org/apps/xfce4-terminal/start). Upstream
source and releases remain at
[gitlab.xfce.org](https://gitlab.xfce.org/apps/xfce4-terminal).

## Issues and license

Report problems with this port in the
[GitHub issue tracker](https://github.com/frankischilling/xfce4-terminal-rs/issues).
Report problems with the official C application to the Xfce project.

The source is licensed under GPL-2.0-or-later. See `COPYING`.
