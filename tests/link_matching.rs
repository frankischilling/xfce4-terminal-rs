//! Link behavior expected from the frozen C terminal widget.
//!
//! Every expectation here was read from `tests/reference/link-probe.c` running
//! against the frozen reference. `tests/reference/link-matching.sh` repeats the
//! comparison over the whole fixture corpus; these tests keep the interesting
//! cases readable and available without a reference build.

use xfce4_terminal::links::{self, LinkKind};

#[test]
fn the_pattern_table_keeps_the_frozen_order_and_kinds() {
    let kinds: Vec<_> = links::PATTERNS.iter().map(|pattern| pattern.kind).collect();

    assert_eq!(
        kinds,
        [
            LinkKind::FullHttp,
            LinkKind::Http,
            LinkKind::File,
            LinkKind::Email,
            LinkKind::FullHttp,
        ]
    );
}

#[test]
fn the_expanded_patterns_keep_the_frozen_lengths() {
    let lengths: Vec<_> = links::PATTERNS
        .iter()
        .map(|pattern| pattern.pattern.len())
        .collect();

    assert_eq!(lengths, [1490, 1408, 1231, 1321, 71]);
}

#[test]
fn only_the_news_and_man_pattern_omits_the_shared_definitions() {
    for (index, pattern) in links::PATTERNS.iter().enumerate() {
        assert_eq!(
            pattern
                .pattern
                .starts_with("(?<APOS_START>(?<='))?(?(DEFINE)"),
            index != 4,
            "pattern {index} has the wrong prefix"
        );
    }
}

#[test]
fn candidates_are_classified_by_the_first_matching_pattern() {
    let expected = [
        ("https://example.com/path", Some(LinkKind::FullHttp)),
        ("HTTP://EXAMPLE.COM", Some(LinkKind::FullHttp)),
        ("http://[dead:beef::1.2.3.4]/", Some(LinkKind::FullHttp)),
        (
            "news://news.example.com/comp.lang.c",
            Some(LinkKind::FullHttp),
        ),
        ("news:comp.lang.c", Some(LinkKind::FullHttp)),
        ("man:ls(1)", Some(LinkKind::FullHttp)),
        (
            "magnet:?xt=urn:btih:0123456789abcdef",
            Some(LinkKind::FullHttp),
        ),
        ("www.example.com:81/path", Some(LinkKind::Http)),
        ("ftp.example.com/pub", Some(LinkKind::Http)),
        ("file:///tmp/example", Some(LinkKind::File)),
        ("file:/tmp/example", Some(LinkKind::File)),
        ("user@example.com", Some(LinkKind::Email)),
        ("mailto:user@example.com", Some(LinkKind::Email)),
        ("user@[::1]", Some(LinkKind::Email)),
        // The unregistered voice-over-IP pattern leaves these to the address
        // pattern, which matches the part after the scheme.
        ("h323:user@example.com", Some(LinkKind::Email)),
        ("sips:user@example.com", Some(LinkKind::Email)),
        // A port number above 65535, a bare host, a host below a "www" label,
        // a single-label address host, and an unsupported scheme all fail.
        ("http://1.2.3.4567/", None),
        ("example.com", None),
        ("abc.www.example.com", None),
        ("user@localhost", None),
        ("gopher://example.com", None),
        ("file://", None),
        ("localhost:8080", None),
        ("not a link", None),
        ("", None),
    ];

    for (candidate, kind) in expected {
        assert_eq!(links::classify(candidate), kind, "candidate {candidate:?}");
    }
}

#[test]
fn only_a_remote_file_uri_is_unclickable() {
    let expected = [
        ("file:///tmp/example", Some(LinkKind::File), true),
        ("file://localhost/tmp/example", Some(LinkKind::File), true),
        ("file://LocalHost/tmp/example", Some(LinkKind::File), true),
        (
            "file://elsewhere.invalid/tmp/example",
            Some(LinkKind::File),
            false,
        ),
        // A single slash is not a valid file URI, so no host name is found.
        ("file:/tmp/example", Some(LinkKind::File), true),
        ("https://elsewhere.invalid/", Some(LinkKind::FullHttp), true),
        ("not a link", None, true),
    ];

    for (candidate, kind, clickable) in expected {
        assert_eq!(
            links::is_clickable(candidate, kind),
            clickable,
            "candidate {candidate:?}"
        );
    }
}

#[test]
fn the_launched_uri_gains_the_scheme_its_pattern_implies() {
    let expected = [
        (
            "https://example.com/path",
            Some(LinkKind::FullHttp),
            "https://example.com/path",
        ),
        ("man:ls", Some(LinkKind::FullHttp), "man:ls"),
        (
            "file:///tmp/example",
            Some(LinkKind::File),
            "file:///tmp/example",
        ),
        (
            "www.example.com",
            Some(LinkKind::Http),
            "http://www.example.com",
        ),
        (
            "user@example.com",
            Some(LinkKind::Email),
            "mailto:user@example.com",
        ),
        (
            "mailto:user@example.com",
            Some(LinkKind::Email),
            "mailto:user@example.com",
        ),
        // The prefix test is case sensitive, so an upper-case scheme is kept
        // and prefixed again.
        (
            "MAILTO:user@example.com",
            Some(LinkKind::Email),
            "mailto:MAILTO:user@example.com",
        ),
    ];

    for (candidate, kind, uri) in expected {
        assert_eq!(
            links::launch_uri(candidate, kind).as_deref(),
            Ok(uri),
            "candidate {candidate:?}"
        );
    }
}

#[test]
fn an_unmatched_candidate_reports_the_frozen_warning() {
    assert_eq!(
        links::launch_uri("not a link", None),
        Err("Invalid tag specified while trying to open link \"not a link\".".to_owned())
    );
}

#[test]
fn copying_a_link_drops_only_a_lower_case_mail_scheme() {
    assert_eq!(
        links::clipboard_text("mailto:user@example.com"),
        "user@example.com"
    );
    assert_eq!(
        links::clipboard_text("MAILTO:user@example.com"),
        "MAILTO:user@example.com"
    );
    assert_eq!(
        links::clipboard_text("https://example.com/path"),
        "https://example.com/path"
    );
    assert_eq!(links::clipboard_text(""), "");
}

#[test]
fn the_fixture_corpus_covers_every_pattern_kind() {
    let fixtures = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/reference/link-fixtures.txt"
    ))
    .expect("read the link fixtures");
    let candidates: Vec<_> = fixtures
        .split('\n')
        .filter(|line| !line.starts_with('#'))
        .collect();
    let kinds: Vec<_> = candidates
        .iter()
        .map(|candidate| links::classify(candidate))
        .collect();

    for kind in [
        Some(LinkKind::FullHttp),
        Some(LinkKind::Http),
        Some(LinkKind::Email),
        Some(LinkKind::File),
        None,
    ] {
        assert!(kinds.contains(&kind), "no fixture candidate is {kind:?}");
    }
}
