//! Writes the link contract of the Rust candidate.
//!
//! `tests/reference/link-probe.c` writes the same report from the frozen C
//! widget, and `tests/reference/link-matching.sh` compares the two. The report
//! goes to a named file because the wrappers that give the frozen probe a
//! display and a session bus add output of their own.

use std::fmt::Write as _;
use std::process::ExitCode;
use std::sync::Mutex;

use xfce4_terminal::links::{
    self, PATTERNS, classify, clipboard_text, is_clickable, kind_name, launch_uri,
};

/// The report under construction.
///
/// The messages the measured calls log have to appear in the report where they
/// happened, exactly as they do in the frozen probe, so the log writer and the
/// reporting functions append to one buffer.
static REPORT: Mutex<String> = Mutex::new(String::new());

macro_rules! report {
    ($($argument:tt)*) => {{
        let mut report = REPORT.lock().expect("the report buffer is usable");
        let _ = writeln!(report, $($argument)*);
    }};
}

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(fixtures), Some(destination), None) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        eprintln!("usage: xfce4-terminal-link-probe FIXTURE_FILE REPORT_FILE");
        return ExitCode::from(2);
    };

    let Ok(fixtures) = std::fs::read_to_string(&fixtures) else {
        eprintln!("cannot read {}", fixtures.to_string_lossy());
        return ExitCode::from(2);
    };

    // The reference warns and carries on with the patterns it has. A probe that
    // did the same would report a contract it cannot produce, so it stops here.
    let compile_errors = links::compile_errors();
    if !compile_errors.is_empty() {
        for (index, error) in compile_errors {
            eprintln!("pattern {index} failed to compile with error code {error}");
        }
        return ExitCode::from(3);
    }

    for (index, entry) in PATTERNS.iter().enumerate() {
        report!(
            "pattern\t{index}\t{}\t{}",
            kind_name(Some(entry.kind)),
            entry.pattern
        );
    }

    // Report the messages of the measured calls instead of leaving them on the
    // standard error stream, where the isolated session adds unrelated noise.
    glib::log_set_writer_func(|level, fields| {
        for field in fields {
            if field.key() == "MESSAGE" {
                let level = if level == glib::LogLevel::Warning {
                    "warning"
                } else {
                    "other"
                };
                report!("log\t{level}\t{}", field.value_str().unwrap_or_default());
            }
        }
        glib::LogWriterOutput::Handled
    });

    let candidates: Vec<_> = fixtures.split('\n').collect();
    for (index, candidate) in candidates.iter().enumerate() {
        // The trailing newline of the file does not introduce a candidate.
        if candidate.starts_with('#') || (index + 1 == candidates.len() && candidate.is_empty()) {
            continue;
        }

        report_candidate(candidate);
    }

    let report = std::mem::take(&mut *REPORT.lock().expect("the report buffer is usable"));
    if let Err(error) = std::fs::write(&destination, report) {
        eprintln!("cannot write {}: {error}", destination.to_string_lossy());
        return ExitCode::from(2);
    }

    ExitCode::SUCCESS
}

fn report_candidate(candidate: &str) {
    let kind = classify(candidate);

    report!("classify\t{candidate}\t{}", kind_name(kind));
    report!("clickable\t{candidate}\t{}", is_clickable(candidate, kind));

    match launch_uri(candidate, kind) {
        Ok(uri) => report!("launch\t{candidate}\t{uri}"),
        Err(message) => {
            // The reference logs this from the call itself. The port returns it
            // instead, so the probe logs it here to keep the report's shape.
            glib::g_warning!(xfce4_terminal::LOG_DOMAIN, "{}", message);
            report!("launch\t{candidate}\t<none>");
        }
    }

    report!("clipboard\t{candidate}\t{}", clipboard_text(candidate));
}
