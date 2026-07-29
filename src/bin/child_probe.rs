//! Writes the spawn request of the Rust candidate.
//!
//! `tests/reference/child-probe.c` writes the same report from the frozen C
//! screen, and `tests/reference/child-process.sh` compares the two. The report
//! goes to a named file because the wrappers that give the frozen probe a
//! display and a session bus add output of their own.
//!
//! Realizing a toplevel belongs to a widget layer the port has not reached, so
//! the frozen probe records the window it realized and this one reports the
//! environment of that same window.

use std::ffi::{OsStr, OsString};
use std::io::Write as _;
use std::os::unix::ffi::OsStrExt;
use std::process::ExitCode;

use xfce4_terminal::child::{self, ChildCommandError, CommandPreferences, Toplevel};
use xfce4_terminal::preferences::{PreferenceValue, Preferences};

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(fixtures), Some(destination), Some(toplevel), None) = (
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
    ) else {
        eprintln!("usage: xfce4-terminal-child-probe FIXTURE_FILE REPORT_FILE TOPLEVEL_FILE");
        return ExitCode::from(2);
    };

    let Ok(fixtures) = std::fs::read_to_string(&fixtures) else {
        eprintln!("cannot read {}", fixtures.to_string_lossy());
        return ExitCode::from(2);
    };

    let preferences = match Preferences::new("xfce4-terminal") {
        Ok(preferences) => preferences,
        Err(error) => {
            eprintln!("cannot open the preference channel: {error}");
            return ExitCode::from(3);
        }
    };

    let mut report = Vec::new();
    let _ = writeln!(report, "constant\tpty-flags\t{}", child::PTY_FLAGS.bits());
    let _ = writeln!(
        report,
        "constant\tspawn-timeout\t{}",
        child::SPAWN_TIMEOUT_MS
    );

    let scenarios: Vec<_> = fixtures.split('\n').collect();
    for (index, scenario) in scenarios.iter().enumerate() {
        // The trailing newline of the file does not introduce a scenario.
        if scenario.starts_with('#') || (index + 1 == scenarios.len() && scenario.is_empty()) {
            continue;
        }

        let _ = writeln!(report, "scenario\t{index}\t{scenario}");
        let fields: Vec<_> = scenario.split('\t').collect();

        match fields.as_slice() {
            [
                "command",
                login_shell,
                run_custom_command,
                custom_command,
                screen_command @ ..,
            ] => {
                let settings = CommandPreferences {
                    login_shell: *login_shell == "true",
                    run_custom_command: *run_custom_command == "true",
                    custom_command: (*custom_command).to_owned(),
                };
                if let Err(error) = store(&preferences, &settings) {
                    eprintln!("cannot store the preferences of scenario {index}: {error}");
                    return ExitCode::from(3);
                }
                report_command(&mut report, index, &preferences, screen_command);
            }
            ["environment", "plain"] => {
                report_environment(&mut report, index, &Toplevel::Unrealized);
            }
            ["environment", "realized"] => {
                let Some(realized) = read_toplevel(&toplevel) else {
                    eprintln!("cannot read a realized toplevel from the frozen probe");
                    return ExitCode::from(3);
                };
                let _ = writeln!(report, "toplevel\t{index}\t{}", realized.0);
                report_environment(&mut report, index, &realized.1);
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

/// Writes the three preferences through the channel the reference shares.
///
/// The frozen screen reads them from Xfconf, so the candidate stores them there
/// as well instead of handing the values straight to the command.
fn store(
    preferences: &Preferences,
    settings: &CommandPreferences,
) -> Result<(), Box<dyn std::error::Error>> {
    preferences.set(
        "command-login-shell",
        PreferenceValue::Boolean(settings.login_shell),
    )?;
    preferences.set(
        "run-custom-command",
        PreferenceValue::Boolean(settings.run_custom_command),
    )?;
    preferences.set(
        "custom-command",
        PreferenceValue::String(Some(settings.custom_command.clone())),
    )?;
    Ok(())
}

fn report_command(
    report: &mut Vec<u8>,
    index: usize,
    preferences: &Preferences,
    screen_command: &[&str],
) {
    let settings = match CommandPreferences::read(preferences) {
        Ok(settings) => settings,
        Err(error) => {
            let _ = writeln!(report, "error\t{index}\t{error}");
            return;
        }
    };
    let screen_command: Vec<OsString> = screen_command.iter().map(OsString::from).collect();

    match child::child_command(Some(&screen_command), &settings) {
        Ok(command) => {
            for (position, argument) in command.spawn_argv().iter().enumerate() {
                let _ = write!(report, "argument\t{index}\t{position}\t");
                let _ = report.write_all(argument.as_bytes());
                let _ = report.write_all(b"\n");
            }
            let _ = writeln!(
                report,
                "spawn-flags\t{index}\t{}",
                command.spawn_flags().bits()
            );
        }
        Err(error) => report_error(report, index, &error),
    }
}

/// Writes the message the reference puts in its error dialog.
///
/// The reference cannot separate the failure from its wording, because it hands
/// the `GError` it received straight to the dialog. The port keeps the reason
/// apart from the message, so the probe writes the message the reason produces.
fn report_error(report: &mut Vec<u8>, index: usize, error: &ChildCommandError) {
    let _ = writeln!(report, "error\t{index}\t{}", error.message());
}

fn report_environment(report: &mut Vec<u8>, index: usize, toplevel: &Toplevel) {
    for (position, variable) in child::child_environment(toplevel).iter().enumerate() {
        let _ = write!(report, "variable\t{index}\t{position}\t");
        let _ = report.write_all(variable.as_bytes());
        let _ = report.write_all(b"\n");
    }
}

/// Reads the toplevel the frozen probe realized, with the line it wrote.
fn read_toplevel(path: &OsStr) -> Option<(String, Toplevel)> {
    let description = std::fs::read_to_string(path).ok()?;
    let description = description.trim_end_matches('\n').to_owned();
    let mut fields = description.split('\t');

    let toplevel = match (fields.next()?, fields.next()?, fields.next()) {
        ("x11", window, Some(display)) => Toplevel::X11 {
            window: window.parse().ok()?,
            display: display.to_owned(),
        },
        _ => return None,
    };

    Some((description, toplevel))
}
