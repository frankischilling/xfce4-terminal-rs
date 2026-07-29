# Links

A terminal turns some of the text it prints into links. Xfce Terminal finds them
with five regular expressions, and the pattern that matched decides how the text
is treated afterwards.

## Patterns

`src/links.rs` assembles the patterns from the same fragments as the C reference
header. A fragment is a macro that expands to a string literal, so each pattern
is a compile-time constant rather than a value built at startup. Splitting the
fragments up is what keeps a pattern of over a thousand characters reviewable.

The table is ordered, and the first pattern that matches wins:

| Order | Pattern | Kind | Matches |
| --- | --- | --- | --- |
| 1 | `URL_AS_IS` | `FullHttp` | a URL that already names a scheme the terminal knows |
| 2 | `URL_HTTP` | `Http` | a host starting with a `www` or `ftp` label |
| 3 | `URL_FILE` | `File` | a `file:` URI |
| 4 | `EMAIL` | `Email` | a mail address, with or without `mailto:` |
| 5 | `NEWS_MAN` | `FullHttp` | the `news:`, `man:`, `info:`, and `magnet:` schemes |

The header also defines a pattern for voice-over-IP schemes, but the reference
never registers it. The port omits it too, so `sips:someone@example.com` is
still classified as a mail address.

The reference compiles this table twice, for two different jobs. It compiles each
pattern with no options to classify the target of an escape-sequence hyperlink,
and it compiles the same text again with
`PCRE2_CASELESS | PCRE2_UTF | PCRE2_NO_UTF_CHECK | PCRE2_MULTILINE`, JIT-compiles
it, and hands it to VTE to highlight matches on screen. Only the first is ported
here. The options are not interchangeable: `PCRE2_UTF` changes what the host
fragment's `(?! [[:ascii:]] ) [[:graph:]]` accepts, so the highlighting path needs
its own proof and belongs with the screen work.

Patterns are compiled through the system PCRE2 library, the same one the C
reference and VTE use. `classify` compiles them once on first use with no
options and looks for a match anywhere in the candidate, which is how the
reference decides what kind of target an escape-sequence hyperlink names. A
pattern that fails to compile is skipped rather than fatal, so classification
falls through to the next pattern; `compile_errors` reports which ones are
unavailable.

A pattern can also fail while running, most plausibly by exhausting a match
limit inside the recursive path subroutine. The reference tells that apart from
a plain absence of a match and warns about it, and so does the port, because
otherwise a limit reached on a long path would be indistinguishable from text
that simply is not a link.

## What a match means

Classification alone does not produce a URI. Each kind adds the scheme its
pattern left out:

- `FullHttp` and `File` candidates are already URIs and are used unchanged.
- an `Http` candidate gains an `http://` prefix.
- an `Email` candidate gains a `mailto:` prefix unless it already starts with
  one. That test is case sensitive, so `MAILTO:someone@example.com` is prefixed
  again, exactly as in the reference.

A candidate with no classification cannot be opened. `launch_uri` reports the
message the reference logs instead of logging it, because deciding what a link
is and reporting a failure to the user belong to different layers.

Only a `file:` URI is restricted. It may be opened locally when its host is
`localhost`, when its host is this machine, or when it names no host at all. A
path from another machine stays copyable but not openable.

Whether the URI names a host is a question for GLib, and the answer does not
follow from whether GLib could turn the URI into a path. GLib reports the host
as soon as it has validated the authority, and only afterwards unescapes the
path; it rejects an escaped separator such as `%2F` along with genuinely
malformed escapes. So `file://elsewhere.invalid/pub/doc%2Fa` yields no path while
still naming a host, and the reference refuses it. Reading the host through a
conversion that reports success or failure would wrongly treat it as local,
which is why `src/ffi/glib.rs` calls `g_filename_from_uri` directly: the safe
binding discards the host whenever it reports an error.

The local form `file:///path` carries an empty host, which GLib reports as no
host, so it stays openable.

Copying a link drops a leading `mailto:` so that an address can be pasted into a
mail client. As with the launch prefix, the comparison is case sensitive.

## Test boundary

`tests/reference/link-contract.tsv` holds the pattern lines the frozen probe
wrote, so the expanded pattern text and the table order are checked against the
reference rather than against a transcribed length.

`tests/link_matching.rs` covers the interesting candidates without needing a
reference build: that contract, classification, clickable file hosts, launch
prefixes, and copied text.

The proof lives in `tests/reference/link-matching.sh`. Both probes read
`tests/reference/link-fixtures.txt` and write the registered patterns and, for
every candidate, its classification, whether it may be opened, the URI it opens
with, and the text copying it produces.

The script then guards individual branches, because a corpus can agree on every
candidate it holds while never reaching a branch at all, and a comparison that
reaches nothing proves nothing about it. It checks that the report accounts for
every candidate, since both probes decide for themselves which lines are
candidates; that every kind was reached; that a candidate outside ASCII kept its
own bytes; that a remote host is refused whether or not its path can be
unescaped, while this machine's own host is accepted; and that the two candidates
decided by table order rather than by the scheme they name keep the kind the
order gives them.

Each probe writes to a named file rather than to standard output. The frozen
probe runs under a display and a session bus, and those wrappers print messages
of their own that would otherwise land in the compared report. For the same
reason the frozen probe uses stdio instead of GLib's printing functions, which
would convert a candidate to the current locale encoding.

`tests/reference/link-probe.c` includes the frozen `terminal-widget.c` and links
the remaining frozen objects, so the file-private helpers report on the reference
rather than on a copy of it. It builds a real widget inside a real window, reads a
real clipboard, and replaces `gtk_show_uri_on_window` to record the URI that would
reach the desktop launcher. The window matters because the frozen code passes the
widget's toplevel to the launcher: without one, GLib's type check reports an
invalid cast on every candidate as soon as the reference is built without
optimization. `tests/reference/probe-command.py` reads the compiler, its flags,
and the link line back out of the reference build so the probe and the frozen
binary share one set of build options.

The harness appends two candidates naming the local host, which only this machine
knows, because that is the only way to reach the "this machine" branch of the
clickable test.

The probe asks for the URI of every candidate, while the reference asks only for
the URI of a candidate it would open. So the report carries a `launch` line for a
remote `file:` URI that the reference would never follow. That is deliberate, and
it means the report describes what each helper answers rather than which helpers a
click reaches.

Four things stay outside this boundary. Highlighting the matches inside a running
terminal needs the VTE widget and a second set of compile options, so it belongs
with the screen work. Nothing compares the extent of a match, only whether there
was one, so the fragments that trim a trailing quote or balance a bracket rest on
the pattern text being identical and the library being shared; comparing extent
belongs with highlighting, which is what extent is for. Copying sets only the
clipboard here, while the reference sets the primary selection as well, in an order
it fixes deliberately; the port has no clipboard writer yet. And the corpus is read
as text, so it cannot carry a candidate that is not valid UTF-8, even though these
patterns are compiled without PCRE2's UTF mode and the reference matches raw bytes.
VTE hands out UTF-8, so nothing reachable is lost today.
