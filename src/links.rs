//! Link patterns and URI handling for text printed in a terminal.
//!
//! The patterns come from the fragments of the C reference's regex header. Each
//! fragment is a macro that expands to a string literal, so the assembled
//! patterns are compile-time constants with the same text as the C build
//! produces. Keeping the fragments apart is what makes the long patterns
//! reviewable; `tests/reference/link-matching.sh` proves the assembled text
//! matches the reference byte for byte.
//!
//! The functions here decide only what a link is and which URI it stands for.
//! Showing a menu, launching a handler, and writing to a clipboard belong to
//! the widget layer, so an unusable candidate is reported as an error instead
//! of being logged here.

use std::sync::LazyLock;

use crate::ffi::pcre2;

macro_rules! apos_start_def {
    () => {
        "(?<APOS_START>(?<='))?"
    };
}

macro_rules! scheme {
    () => {
        "(?ix: news | telnet | nntp | https? | ftps? | sftp | webcal )"
    };
}

macro_rules! userchars {
    () => {
        "-+.[:alnum:]"
    };
}

macro_rules! user {
    () => {
        concat!("[", userchars!(), "]+")
    };
}

macro_rules! passchars_class {
    () => {
        "[-[:alnum:]\\Q,?;.:/!%$^*&~\"#'\\E]"
    };
}

macro_rules! pass {
    () => {
        concat!("(?x: :", passchars_class!(), "* )?")
    };
}

macro_rules! userpass {
    () => {
        concat!("(?:", user!(), pass!(), "@)?")
    };
}

macro_rules! s4_def {
    () => {
        "(?(DEFINE)(?<S4>(?x: (?: [0-9] | [1-9][0-9] | 1[0-9]{2} | 2[0-4][0-9] | 25[0-5] ) (?! [0-9] ) )))"
    };
}

macro_rules! ipv4_def {
    () => {
        concat!(
            s4_def!(),
            "(?(DEFINE)(?<IPV4>(?x: (?: (?&S4) \\. ){3} (?&S4) )))"
        )
    };
}

macro_rules! s6_def {
    () => {
        "(?(DEFINE)(?<S6>[[:xdigit:]]{1,4})(?<CS6>:(?&S6))(?<S6C>(?&S6):))"
    };
}

macro_rules! ipv6_full {
    () => {
        "(?x: (?&S6C){7} (?&S6) )"
    };
}

macro_rules! ipv6_left {
    () => {
        "(?x: : (?&CS6){1,7} )"
    };
}

macro_rules! ipv6_mid {
    () => {
        "(?x: (?! (?: [[:xdigit:]]*: ){8} ) (?&S6C){1,6} (?&CS6){1,6} )"
    };
}

macro_rules! ipv6_right {
    () => {
        "(?x: (?&S6C){1,7} : )"
    };
}

macro_rules! ipv6_null {
    () => {
        "(?x: :: )"
    };
}

macro_rules! ipv6v4_full {
    () => {
        "(?x: (?&S6C){6} )"
    };
}

macro_rules! ipv6v4_left {
    () => {
        "(?x: :: (?&S6C){0,5} )"
    };
}

macro_rules! ipv6v4_mid {
    () => {
        "(?x: (?! (?: [[:xdigit:]]*: ){7} ) (?&S6C){1,4} (?&CS6){1,4} ) :"
    };
}

macro_rules! ipv6v4_right {
    () => {
        "(?x: (?&S6C){1,5} : )"
    };
}

macro_rules! ip_def {
    () => {
        concat!(
            ipv4_def!(),
            s6_def!(),
            "(?(DEFINE)(?<IPV6>(?x: (?: ",
            ipv6_null!(),
            " | ",
            ipv6_left!(),
            " | ",
            ipv6_mid!(),
            " | ",
            ipv6_right!(),
            " | ",
            ipv6_full!(),
            " | (?: ",
            ipv6v4_full!(),
            " | ",
            ipv6v4_left!(),
            " | ",
            ipv6v4_mid!(),
            " | ",
            ipv6v4_right!(),
            " ) (?&IPV4) ) (?! [.:[:xdigit:]] ) )))"
        )
    };
}

macro_rules! hostnamesegmentchars_class {
    () => {
        "(?x: [-[:alnum:]] | (?! [[:ascii:]] ) [[:graph:]] )"
    };
}

macro_rules! hostname1 {
    () => {
        concat!(
            "(?x: (?: ",
            hostnamesegmentchars_class!(),
            "+ \\. )* ",
            hostnamesegmentchars_class!(),
            "* (?! [0-9] ) ",
            hostnamesegmentchars_class!(),
            "+ )"
        )
    };
}

macro_rules! hostname2 {
    () => {
        concat!(
            "(?x: (?: ",
            hostnamesegmentchars_class!(),
            "+ \\.)+ ",
            hostname1!(),
            " )"
        )
    };
}

macro_rules! url_host {
    () => {
        concat!("(?x: ", hostname1!(), " | (?&IPV4) | \\[ (?&IPV6) \\] )")
    };
}

macro_rules! email_host {
    () => {
        concat!(
            "(?x: ",
            hostname2!(),
            " | \\[ (?: (?&IPV4) | (?&IPV6) ) \\] )"
        )
    };
}

macro_rules! n_1_65535 {
    () => {
        "(?x: (?: [1-9][0-9]{0,3} | [1-5][0-9]{4} | 6[0-4][0-9]{3} | 65[0-4][0-9]{2} | 655[0-2][0-9] | 6553[0-5] ) (?! [0-9] ) )"
    };
}

macro_rules! port {
    () => {
        concat!("(?x: \\:", n_1_65535!(), " )?")
    };
}

macro_rules! pathchars_class {
    () => {
        "[-[:alnum:]\\Q_$.+!*,:;@&=?/~#|%'\\E]"
    };
}

macro_rules! pathterm_class {
    () => {
        "[-[:alnum:]\\Q_$+*:@&=/~#|%'\\E]"
    };
}

macro_rules! pathterm_noapos_class {
    () => {
        "[-[:alnum:]\\Q_$+*:@&=/~#|%\\E]"
    };
}

macro_rules! path_inner_def {
    () => {
        concat!(
            "(?(DEFINE)(?<PATH_INNER>(?x: (?: ",
            pathchars_class!(),
            "* (?: \\( (?&PATH_INNER) \\) | \\[ (?&PATH_INNER) \\] ) )* ",
            pathchars_class!(),
            "* )))"
        )
    };
}

macro_rules! path_def {
    () => {
        concat!(
            "(?(DEFINE)(?<PATH>(?x: (?: ",
            pathchars_class!(),
            "* (?: \\( (?&PATH_INNER) \\) | \\[ (?&PATH_INNER) \\] ) )* (?: ",
            pathchars_class!(),
            "* (?(<APOS_START>)",
            pathterm_noapos_class!(),
            "|",
            pathterm_class!(),
            ") )? )))"
        )
    };
}

macro_rules! urlpath {
    () => {
        "(?x: /(?&PATH) )?"
    };
}

macro_rules! defs {
    () => {
        concat!(apos_start_def!(), ip_def!(), path_inner_def!(), path_def!())
    };
}

/// Matches a URL that already carries a scheme the terminal recognizes.
pub const URL_AS_IS: &str = concat!(
    defs!(),
    scheme!(),
    "://",
    userpass!(),
    url_host!(),
    port!(),
    urlpath!()
);

/// Matches a local or remote `file:` URI.
pub const URL_FILE: &str = concat!(
    defs!(),
    "(?ix: file:/ (?: / (?: ",
    hostname1!(),
    " )? / )? (?! / ) )(?&PATH)"
);

/// Matches a host that begins with a `www` or `ftp` label and has no scheme.
pub const URL_HTTP: &str = concat!(
    defs!(),
    "(?<!(?:",
    hostnamesegmentchars_class!(),
    "|[.]))(?=(?i:www|ftp))",
    hostname1!(),
    port!(),
    urlpath!()
);

/// Matches an electronic mail address with an optional `mailto:` scheme.
pub const EMAIL: &str = concat!(defs!(), "(?i:mailto:)?", user!(), "@", email_host!());

/// Matches the schemes that name documentation or content rather than a host.
pub const NEWS_MAN: &str =
    "(?i:news:|man:|info:|magnet:)[-[:alnum:]\\Q^_{|}~!\"#$%&'()*+,./;:=?`\\E]+";

/// The scheme a matched link belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkKind {
    /// The candidate is already a complete URI.
    FullHttp,
    /// The candidate is a host name that needs an `http` scheme.
    Http,
    /// The candidate is a mail address.
    Email,
    /// The candidate is a `file:` URI.
    File,
}

/// One entry of the pattern table, in the order the reference registers them.
#[derive(Clone, Copy, Debug)]
pub struct LinkPattern {
    /// The expanded pattern text.
    pub pattern: &'static str,
    /// The scheme handling that applies to a match.
    pub kind: LinkKind,
}

/// The registered patterns. Earlier entries win when several would match.
pub const PATTERNS: [LinkPattern; 5] = [
    LinkPattern {
        pattern: URL_AS_IS,
        kind: LinkKind::FullHttp,
    },
    LinkPattern {
        pattern: URL_HTTP,
        kind: LinkKind::Http,
    },
    LinkPattern {
        pattern: URL_FILE,
        kind: LinkKind::File,
    },
    LinkPattern {
        pattern: EMAIL,
        kind: LinkKind::Email,
    },
    LinkPattern {
        pattern: NEWS_MAN,
        kind: LinkKind::FullHttp,
    },
];

/// The scheme prefix the reference prepends to a mail address.
const MAILTO: &str = "mailto:";

static COMPILED: LazyLock<Vec<Result<pcre2::Pattern, i32>>> = LazyLock::new(|| {
    PATTERNS
        .iter()
        .map(|entry| pcre2::Pattern::compile(entry.pattern, 0))
        .collect()
});

/// Returns the name the parity comparison prints for a classification.
pub fn kind_name(kind: Option<LinkKind>) -> &'static str {
    match kind {
        None => "none",
        Some(LinkKind::FullHttp) => "full-http",
        Some(LinkKind::Http) => "http",
        Some(LinkKind::Email) => "email",
        Some(LinkKind::File) => "file",
    }
}

/// Returns the PCRE2 error number of every pattern that failed to compile.
///
/// The reference warns once per unusable pattern and then skips it, so a
/// classification stays possible even then. An empty result means all patterns
/// are available.
pub fn compile_errors() -> Vec<(usize, i32)> {
    COMPILED
        .iter()
        .enumerate()
        .filter_map(|(index, compiled)| compiled.as_ref().err().map(|error| (index, *error)))
        .collect()
}

/// Classifies a candidate by the first pattern that matches it.
///
/// A match may start anywhere in the candidate, which is how the reference
/// classifies the target of an escape-sequence hyperlink. A pattern that
/// cannot be compiled or that fails to run is skipped, so classification falls
/// through to the following pattern.
pub fn classify(candidate: &str) -> Option<LinkKind> {
    for (entry, compiled) in PATTERNS.iter().zip(COMPILED.iter()) {
        if compiled
            .as_ref()
            .is_ok_and(|pattern| pattern.matches(candidate) == Ok(true))
        {
            return Some(entry.kind);
        }
    }
    None
}

/// Returns the URI a candidate opens with, adding the scheme its kind implies.
///
/// A candidate without a classification cannot be opened. The error text is
/// the message the reference logs in that case.
pub fn launch_uri(candidate: &str, kind: Option<LinkKind>) -> Result<String, String> {
    match kind {
        Some(LinkKind::FullHttp) | Some(LinkKind::File) => Ok(candidate.to_owned()),
        Some(LinkKind::Http) => Ok(format!("http://{candidate}")),
        Some(LinkKind::Email) if candidate.starts_with(MAILTO) => Ok(candidate.to_owned()),
        Some(LinkKind::Email) => Ok(format!("{MAILTO}{candidate}")),
        None => Err(format!(
            "Invalid tag specified while trying to open link \"{candidate}\"."
        )),
    }
}

/// Reports whether a link may be opened.
///
/// Only a `file:` URI is restricted: it has to name this host so that a path
/// from another machine is not opened locally. A candidate that is not a valid
/// file URI carries no host name and counts as local.
pub fn is_clickable(candidate: &str, kind: Option<LinkKind>) -> bool {
    if kind != Some(LinkKind::File) {
        return true;
    }

    match glib::filename_from_uri(candidate) {
        Ok((_, Some(host))) => {
            host.eq_ignore_ascii_case("localhost")
                || host.eq_ignore_ascii_case(glib::host_name().as_str())
        }
        _ => true,
    }
}

/// Returns the text that copying a link puts on the clipboard.
///
/// The mail scheme is dropped so that an address can be pasted into a mail
/// client, but the comparison is case sensitive in the reference and stays so
/// here.
pub fn clipboard_text(candidate: &str) -> &str {
    candidate.strip_prefix(MAILTO).unwrap_or(candidate)
}

#[cfg(test)]
mod tests {
    use super::{LinkKind, PATTERNS, compile_errors, kind_name};

    #[test]
    fn every_pattern_compiles_with_the_reference_options() {
        assert_eq!(compile_errors(), []);
    }

    #[test]
    fn the_shared_definitions_appear_once_per_pattern() {
        for (index, entry) in PATTERNS.iter().enumerate() {
            assert_eq!(
                entry.pattern.matches("(?(DEFINE)").count(),
                if index == 4 { 0 } else { 6 },
                "pattern {index} defines an unexpected number of subroutines"
            );
        }
    }

    #[test]
    fn classification_names_match_the_reference_spelling() {
        assert_eq!(kind_name(None), "none");
        assert_eq!(kind_name(Some(LinkKind::FullHttp)), "full-http");
        assert_eq!(kind_name(Some(LinkKind::Http)), "http");
        assert_eq!(kind_name(Some(LinkKind::Email)), "email");
        assert_eq!(kind_name(Some(LinkKind::File)), "file");
    }
}
