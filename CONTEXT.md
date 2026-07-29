# Domain context

## Purpose

This repository contains an unofficial Rust port of Xfce Terminal. The port
uses the existing GTK 3 and VTE libraries and measures observable behavior
against one frozen revision of the C application.

## Terms

### C reference

The upstream application at commit
`b5933b80d28ca35f873df8da2998e23be5f4e104`. Differential tests build this
revision in a detached worktree. It is the source of truth for behavior, not a
runtime dependency of the finished Rust application.

### Rust candidate

The replacement executable while parity work is under way. It is named
`xfce4-terminal-rs` until the final cutover so developers can run it beside the
C reference.

### Launch request

One invocation of the program, including its environment and command-line
arguments. A request can describe several windows and tabs.

### Window specification

The parsed settings for one window, including its tabs, display, geometry,
visibility, state, font, zoom level, and drop-down mode.

### Tab specification

The parsed settings for one terminal tab, including its command, working
directory, title behavior, colors, active state, and hold behavior.

### Spawn request

Everything a screen settles before VTE forks: the file to execute, the argument
vector that file receives, the spawn flags that describe that vector, and the
environment the child inherits. A tab specification can name the command
outright; otherwise the preferences choose between their own command line and
the user's login shell.

### Terminal server

The first process that owns `org.xfce.Terminal5` on the session bus. Later
processes send their launch requests to this server unless server mode is
disabled.

### Parity row

A behavior listed in `docs/PARITY.md` with an independent C-versus-Rust proof.
A row is complete only when its automated check passes.

