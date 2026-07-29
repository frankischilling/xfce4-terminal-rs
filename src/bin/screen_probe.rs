//! Writes the screen-model report of the Rust candidate.
//!
//! `tests/reference/screen-probe.c` writes the same report from the frozen C
//! screen, and `tests/reference/screen-model.sh` compares the two. The report
//! goes to a named file because the wrappers that give the frozen probe a
//! display and a session bus add output of their own.

use std::io::Write as _;
use std::process::ExitCode;

use gdk::RGBA;
use xfce4_terminal::screen::{self, AppliedColors, ColorInputs, TitleMode};

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(fixtures), Some(destination), None) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        eprintln!("usage: xfce4-terminal-screen-probe FIXTURE_FILE REPORT_FILE");
        return ExitCode::from(2);
    };

    let Ok(fixtures) = std::fs::read_to_string(&fixtures) else {
        eprintln!("cannot read {}", fixtures.to_string_lossy());
        return ExitCode::from(2);
    };

    let mut report = Vec::new();
    let scenarios: Vec<_> = fixtures.split('\n').collect();
    for (index, scenario) in scenarios.iter().enumerate() {
        if scenario.starts_with('#') || (index + 1 == scenarios.len() && scenario.is_empty()) {
            continue;
        }

        let _ = writeln!(report, "scenario\t{index}\t{scenario}");
        let fields: Vec<_> = scenario.split('\t').collect();
        match fields.as_slice() {
            ["title-parse", session, directory, vte_title, template] => {
                let parsed = screen::parse_title(
                    unescape(template).as_deref(),
                    session.parse().expect("session id"),
                    unescape(directory).as_deref(),
                    unescape(vte_title).as_deref(),
                );
                let _ = writeln!(report, "title-parse\t{index}\t{parsed}");
            }
            [
                "title",
                custom,
                initial,
                preference_initial,
                mode,
                session,
                directory,
                vte_title,
            ] => {
                let title = screen::screen_title(
                    unescape(custom).as_deref(),
                    unescape(initial).as_deref(),
                    unescape(preference_initial).as_deref().unwrap_or(""),
                    TitleMode::parse(mode).expect("title mode"),
                    session.parse().expect("session id"),
                    unescape(directory).as_deref(),
                    unescape(vte_title).as_deref(),
                );
                let _ = writeln!(report, "title\t{index}\t{title}");
            }
            ["paste", text] => {
                let unsafe_text = screen::is_text_unsafe(unescape(text).as_deref());
                let _ = writeln!(
                    report,
                    "paste\t{index}\t{}",
                    if unsafe_text { "unsafe" } else { "safe" }
                );
            }
            ["cwd", stored, uri] => {
                let directory = screen::resolve_working_directory(
                    unescape(stored).as_deref(),
                    unescape(uri).as_deref(),
                    None,
                )
                .unwrap_or_default();
                let _ = writeln!(report, "cwd\t{index}\t{directory}");
            }
            [
                "colors",
                palette,
                foreground,
                background,
                use_theme,
                vary,
                cursor_default,
                selection_default,
                bold_default,
                bold_is_bright,
                custom_fg,
                custom_bg,
                cursor_fg,
                cursor,
                selection,
                selection_bg,
                bold,
            ] => {
                let applied = screen::resolve_colors(&ColorInputs {
                    palette: unescape(palette),
                    foreground: unescape(foreground),
                    background: unescape(background),
                    use_theme: *use_theme == "true",
                    background_vary: *vary == "true",
                    custom_foreground: unescape(custom_fg),
                    custom_background: unescape(custom_bg),
                    stored_background: None,
                    cursor_use_default: *cursor_default == "true",
                    cursor_foreground: unescape(cursor_fg),
                    cursor: unescape(cursor),
                    selection_use_default: *selection_default == "true",
                    selection: unescape(selection),
                    selection_background: unescape(selection_bg),
                    bold_use_default: *bold_default == "true",
                    bold: unescape(bold),
                    bold_is_bright: *bold_is_bright == "true",
                    theme_foreground: RGBA::WHITE,
                    theme_background: RGBA::BLACK,
                });
                write_colors(&mut report, index, &applied);
            }
            _ => {
                eprintln!("unknown scenario on line {index}");
                return ExitCode::from(2);
            }
        }
    }

    if let Err(error) = std::fs::write(&destination, report) {
        eprintln!("cannot write {}: {error}", destination.to_string_lossy());
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

fn write_colors(report: &mut Vec<u8>, index: usize, applied: &AppliedColors) {
    let _ = writeln!(
        report,
        "colors\t{index}\t{}",
        if applied.used_default_palette {
            "default"
        } else {
            "palette"
        }
    );
    write_optional_rgba(
        report,
        "fg",
        applied
            .foreground
            .as_ref()
            .filter(|_| !applied.used_default_palette),
    );
    write_optional_rgba(
        report,
        "bg",
        applied
            .background
            .as_ref()
            .filter(|_| !applied.used_default_palette),
    );
    if let Some(palette) = &applied.palette {
        for (position, color) in palette.iter().enumerate() {
            write_rgba(report, &format!("palette-{position}"), color);
        }
    }
    if applied.cursor_configured {
        write_optional_rgba(report, "cursor-fg", applied.cursor_foreground.as_ref());
        write_optional_rgba(report, "cursor", applied.cursor.as_ref());
    }
    if applied.selection_configured {
        write_optional_rgba(
            report,
            "selection-fg",
            applied.selection_foreground.as_ref(),
        );
        write_optional_rgba(
            report,
            "selection-bg",
            applied.selection_background.as_ref(),
        );
    }
    // VTE 0.52+ always receives the bold color, including an explicit NULL when
    // the preference asks for the default.
    write_optional_rgba(report, "bold", applied.bold.as_ref());
    let _ = writeln!(
        report,
        "bold-is-bright\t{index}\t{}",
        if applied.bold_is_bright {
            "true"
        } else {
            "false"
        }
    );
}

fn write_optional_rgba(report: &mut Vec<u8>, name: &str, color: Option<&RGBA>) {
    match color {
        Some(color) => write_rgba(report, name, color),
        None => {
            let _ = writeln!(report, "color\t{name}\t-");
        }
    }
}

fn write_rgba(report: &mut Vec<u8>, name: &str, color: &RGBA) {
    let _ = writeln!(
        report,
        "color\t{name}\t{:.6}\t{:.6}\t{:.6}\t{:.6}",
        color.red(),
        color.green(),
        color.blue(),
        color.alpha()
    );
}

fn unescape(field: &str) -> Option<String> {
    if field == "-" {
        return None;
    }
    let mut result = String::new();
    let mut chars = field.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(character);
        }
    }
    Some(result)
}
