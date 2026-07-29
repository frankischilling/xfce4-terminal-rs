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

Patterns are compiled through the system PCRE2 library, the same one the C
reference and VTE use. `classify` compiles them once on first use with no
options and looks for a match anywhere in the candidate, which is how the
reference decides what kind of target an escape-sequence hyperlink names. A
pattern that fails to compile is skipped rather than fatal, so classification
falls through to the next pattern; `compile_errors` reports which ones are
unavailable.

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
`localhost`, when its host is this machine, or when the text is not a valid file
URI and therefore names no host at all. A path from another machine stays
copyable but not openable.

Copying a link drops a leading `mailto:` so that an address can be pasted into a
mail client. As with the launch prefix, the comparison is case sensitive.

## Test boundary

`tests/link_matching.rs` covers the interesting candidates without needing a
reference build: the order and kinds of the table, the length of each expanded
pattern, classification, clickable file hosts, launch prefixes, and copied text.

The proof lives in `tests/reference/link-matching.sh`. Both probes read
`tests/reference/link-fixtures.txt` and print the registered patterns and, for
every candidate, its classification, whether it may be opened, the URI it opens
with, and the text copying it produces. The script then checks that the corpus
actually reached every kind, so an empty comparison cannot pass.

`tests/reference/link-probe.c` includes the frozen `terminal-widget.c` and links
the remaining frozen objects, so the file-private helpers report on the reference
rather than on a copy of it. It builds a real widget, reads a real clipboard, and
replaces `gtk_show_uri_on_window` to record the URI that would reach the desktop
launcher. `tests/reference/probe-command.py` reads the compile flags and link
line back out of the reference build so the probe and the frozen binary share one
set of build options.

The harness adds one candidate that names the local host, which is the only way
to reach the "this machine" branch of the clickable test.

Highlighting the matches inside a running terminal is not covered here. That
needs the VTE widget, so it belongs with the screen work.
