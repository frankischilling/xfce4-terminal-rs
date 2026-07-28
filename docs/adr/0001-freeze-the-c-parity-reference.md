# ADR 0001: Freeze the C parity reference

## Status

Accepted.

## Context

Upstream Xfce Terminal continues to change while the Rust port is being built.
A moving comparison target would make failures hard to interpret and could
mix new upstream work with mistakes in the port.

## Decision

Parity is measured against commit
`b5933b80d28ca35f873df8da2998e23be5f4e104`, which reports version
`1.2.0-dev`. Tests build that revision from the repository history in a
detached worktree.

The C executable remains the installed program until the final parity gate.
The Rust candidate has a different development name so both programs can run
from the build tree.

## Consequences

New upstream changes are not folded into parity work automatically. They can be
ported after the frozen behavior passes.

The repository must retain the reference commit in its Git history. CI checkouts
that build the reference need full history.

