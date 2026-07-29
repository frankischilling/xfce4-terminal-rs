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
| Terminal links | Pattern text and table order, hyperlink classification, opened URI, clickable file hosts, and clipboard text | Complete |
| Terminal child process | The command, the argument vector, the spawn flags, and the child environment, over each answer the login shell search can reach as an ordinary user, and with and without a realized X11 toplevel | Complete |
| Terminal screen | Child lifecycle, titles, colors, search, paste, scrolling, and the highlighting the same patterns drive inside VTE | Not started |
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

Some reference behavior lives in file-private functions. A probe for those
includes the frozen source file and links the rest of the frozen objects, and
`tests/reference/probe-command.py` recovers the compile flags and link line from
the reference build so the probe matches the frozen binary's build options.

Including the frozen source also puts the private fields of a frozen object in
reach. A probe may read or write one when the behavior under test cannot be
entered from outside, provided the values it reports still come from frozen
code. The link probe reads the widget's compiled patterns for that reason: the
function that uses them takes a `GdkEvent` and needs a live terminal, so the
classification cannot be requested directly, but the patterns, their compile
options, and their order remain the reference's own. The child probe writes a
screen's own command for the same reason: a screen receives one only through a
tab attribute structure that the option parser owns.

Some behavior depends on the host rather than on either program.
`tests/reference/login-shell-shim.c` is preloaded into both, so that the login
shell search can be followed past the answer a real host would give, down to a
host that offers no shell at all. The shim answers only the two questions that
search asks, and only about the paths the test names.

The child environment carries a realized toplevel's window and display, which
name resources of a single run. Both probes therefore share one display, and the
frozen probe records the window it realized so the candidate can report the
environment of that same window. Choosing and realizing a window belongs to the
window interface row.

Two parts of the child process contract are still unproven, and neither row
above claims them. A Wayland session names its display in a variable of its own,
which no comparison reaches until the drop-down row brings a compositor. A
terminal installed set-user-id ignores the `SHELL` variable, which no comparison
reaches while both programs run as the user who started them.

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
