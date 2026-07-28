# Architecture

## Build boundary

Meson remains the package entrypoint because it already owns native dependency
checks, translations, desktop files, icons, color schemes, the man page, and
installation. Cargo builds the Rust code. Meson calls Cargo with a private
target directory and copies the candidate binary into the Meson build tree.

During the port, Meson also builds the original C executable. The reference
harness can build the frozen revision in a detached worktree, which keeps later
source deletion from weakening the differential tests.

## Rust boundary

The Rust package has a library and a small binary entrypoint. Behavior that can
be tested without a display belongs in the library. GTK controllers will own
widgets through composition and connect their signals on the GTK main thread.
The port does not need to reproduce private C GObject type names because the
application exports no widget ABI.

`src/cli.rs` owns immediate output, launch specification parsing, and the
shared C/Rust probe format. It does not create GTK objects. This keeps option
parity testable without a display server.

Unsafe calls belong in small FFI modules. Each wrapper states the ownership,
thread, lifetime, error, and version assumptions that make the safe interface
valid. Application code does not call raw VTE, Xfce, utempter, or BSD functions
directly.

## Compatibility boundary

The installed executable name, desktop identity, D-Bus service, Xfconf schema,
accelerator file, XDG paths, translations, themes, and man page remain
unchanged. The Rust candidate uses a temporary name only during development.
