# Parity contract

The Rust application is compared with Xfce Terminal at commit
`b5933b80d28ca35f873df8da2998e23be5f4e104`. A checked row needs an automated
C-versus-Rust proof. Building successfully is necessary, but it does not prove
parity on its own.

| Area | Proof boundary | Status |
| --- | --- | --- |
| Build foundation | Cargo and Meson build the Rust candidate; CI can build the frozen C revision | Complete |
| Command line | Exit status, stdout, stderr, and parsed launch specifications | Complete |
| Preferences | Property names, types, defaults, Xfconf updates, legacy import, and accelerators | Complete |
| Terminal screen | PTY environment, child lifecycle, titles, colors, search, links, paste, and scrolling | Not started |
| Window interface | Windows, tabs, menus, toolbar, shortcuts, focus, drag and drop, and confirmations | Not started |
| Application service | Multi-window reuse, session state, and the `org.xfce.Terminal5` wire contract | Not started |
| Drop-down mode | Placement, animation, monitor changes, focus, X11, and Wayland layer shell | Not started |
| System integration | Login records and FreeBSD or DragonFlyBSD foreground-process checks | Not started |
| Packaging and pixels | Installed files and exact client-area screenshots in the frozen test environment | Not started |

`tests/reference/BASELINE` records the exact C commit. Local builds use
`tests/reference/build-reference.sh`. CI uses
`tests/reference/Containerfile` to pin the native build environment.
`tests/reference/differential.sh` captures exit status, standard output, and
standard error from both programs.

## Rules

- Expected values must come from the frozen C executable or a documented public
  interface.
- Each test observes a public boundary. Tests do not reach into private widget
  state merely to make a comparison easier.
- Dynamic values such as process IDs are normalized only when the normalization
  is recorded beside the test.
- Visual comparisons use fixed libraries, theme, fonts, locale, DPI, window
  size, terminal contents, cursor state, and compositor.
- The final cutover requires every row above to be complete.
