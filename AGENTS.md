# Repository instructions

## Current documentation

Use Context7 MCP whenever work depends on a library, framework, SDK, API,
command-line tool, or cloud service. Resolve the library ID first, then query
the selected documentation with the full question. Prefer Context7 to web
searches for library documentation.

Context7 is not needed for business logic, code review, general programming
concepts, or scripts that do not depend on an external API.

## Source tree

The C application at commit `b5933b80d28ca35f873df8da2998e23be5f4e104`
is the parity reference. During the migration, Meson builds that application as
`build/terminal/xfce4-terminal` and the Rust candidate as
`build/rust/xfce4-terminal-rs`.

Keep each parity slice buildable. Do not remove a C behavior until the Rust
candidate has a differential test for its public boundary.

Read `README.md`, `HACKING`, `CONTEXT.md`, relevant ADRs, and `docs/PARITY.md`
before changing behavior.

## Verification

Run the narrow Rust test for the behavior being changed, then run:

```sh
cargo test --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
meson setup build
meson compile -C build
meson test -C build --print-errorlogs
```

Use a fresh Meson build directory when build definitions change.

## Agent skills

### Issue tracker

Work is tracked in GitHub Issues. External pull requests are not a triage
request surface. See `docs/agents/issue-tracker.md`.

### Triage labels

The repository uses the five standard triage labels. See
`docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository. Read `CONTEXT.md` and relevant files under
`docs/adr/`. See `docs/agents/domain.md`.

