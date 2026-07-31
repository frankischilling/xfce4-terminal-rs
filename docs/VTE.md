# VTE adapter

`src/terminal.rs` owns the first live-widget boundary in the Rust port. It
creates a `VteTerminal`, keeps the five frozen link patterns registered while
URL highlighting is enabled, and removes those patterns when the preference is
disabled.

The adapter compiles every pattern with the same PCRE2 flags used by the C
widget: case-insensitive matching, UTF handling, disabled UTF validation, and
multiline matching. It requests both JIT modes that the C widget requests, but
a JIT failure does not discard an otherwise usable pattern.

`VteAdapter::copy_link` writes the normalized link text to PRIMARY first and
CLIPBOARD second. The order comes from the frozen context-menu action. A
lowercase `mailto:` prefix is removed before either selection is updated.

`tests/vte_adapter.rs` runs the candidate under Xvfb. The focused differential
check is `tests/reference/vte-adapter.sh`, which builds a probe against the
frozen C widget and compares the configured-off state, the enabled pattern
count, both selections, and the disabled state with the Rust probe. It uses a
private session bus and XDG directories so the Xfconf preference starts from a
known state.

This is a widget-boundary check, not complete terminal-screen parity. The
candidate has no screen controller yet, so that controller must subscribe to
Xfconf changes and call `sync_link_highlighting`. Pointer-driven link lookup,
the observable order of selection ownership, terminal child lifecycle, search,
scrolling, backgrounds, and window actions remain under issue #4.
