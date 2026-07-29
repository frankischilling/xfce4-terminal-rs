//! Prints the link contract of the Rust candidate.
//!
//! `tests/reference/link-probe.c` prints the same report from the frozen C
//! widget, and `tests/reference/link-matching.sh` compares the two.

use xfce4_terminal::links::{
    self, PATTERNS, classify, clipboard_text, is_clickable, kind_name, launch_uri,
};

fn main() -> std::process::ExitCode {
    let Some(fixtures) = std::env::args_os().nth(1) else {
        eprintln!("usage: xfce4-terminal-link-probe FIXTURE_FILE");
        return std::process::ExitCode::from(2);
    };

    let Ok(fixtures) = std::fs::read_to_string(&fixtures) else {
        eprintln!("cannot read {}", fixtures.to_string_lossy());
        return std::process::ExitCode::from(2);
    };

    // The reference warns and carries on with the patterns it has. A probe that
    // did the same would report a contract it cannot produce, so it stops here.
    let compile_errors = links::compile_errors();
    if !compile_errors.is_empty() {
        for (index, error) in compile_errors {
            eprintln!("pattern {index} failed to compile with error code {error}");
        }
        return std::process::ExitCode::from(3);
    }

    for (index, entry) in PATTERNS.iter().enumerate() {
        println!(
            "pattern\t{index}\t{}\t{}",
            kind_name(Some(entry.kind)),
            entry.pattern
        );
    }

    let candidates: Vec<_> = fixtures.split('\n').collect();
    for (index, candidate) in candidates.iter().enumerate() {
        // The trailing newline of the file does not introduce a candidate.
        if candidate.starts_with('#') || (index + 1 == candidates.len() && candidate.is_empty()) {
            continue;
        }

        report_candidate(candidate);
    }

    std::process::ExitCode::SUCCESS
}

fn report_candidate(candidate: &str) {
    let kind = classify(candidate);

    println!("classify\t{candidate}\t{}", kind_name(kind));
    println!("clickable\t{candidate}\t{}", is_clickable(candidate, kind));

    match launch_uri(candidate, kind) {
        Ok(uri) => println!("launch\t{candidate}\t{uri}"),
        Err(message) => {
            println!("log\twarning\t{message}");
            println!("launch\t{candidate}\t<none>");
        }
    }

    println!("clipboard\t{candidate}\t{}", clipboard_text(candidate));
}
