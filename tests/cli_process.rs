use std::process::Command;

use xfce4_terminal::cli::{color_table, format_launch_specs_bytes, help_text, parse_launch_os};

fn candidate(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xfce4-terminal-rs"))
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .expect("run Rust candidate")
}

#[test]
fn help_and_color_table_are_written_to_stdout() {
    let help = candidate(&["--help"]);
    assert!(help.status.success());
    assert_eq!(help.stdout, help_text().as_bytes());
    assert!(help.stderr.is_empty());

    let colors = candidate(&["--color-table"]);
    assert!(colors.status.success());
    assert_eq!(colors.stdout, color_table().as_bytes());
    assert!(colors.stderr.is_empty());
}

#[test]
fn parse_errors_use_the_reference_process_contract() {
    let output = candidate(&["--zoom=8"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "xfce4-terminal: Option \"--zoom\" requires specifying the zoom (-7 .. 7) as its parameter\n"
    );
}

#[test]
fn version_uses_the_reference_name_and_revision() {
    let output = candidate(&["--version"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.starts_with("xfce4-terminal 1.2.0-dev-b5933b80 (Xfce "));
    assert!(stdout.ends_with(
        "Please report bugs to <https://gitlab.xfce.org/apps/xfce4-terminal/-/issues>.\n"
    ));
    assert!(output.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn non_utf8_options_report_reference_bytes_without_panicking() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let output = Command::new(env!("CARGO_BIN_EXE_xfce4-terminal-rs"))
        .arg(OsString::from_vec(vec![b'-', 0xff]))
        .env("LC_ALL", "C")
        .output()
        .expect("run Rust candidate with a non-UTF-8 option");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"xfce4-terminal: Unknown option \"-\xff\"\n");

    let grouped = Command::new(env!("CARGO_BIN_EXE_xfce4-terminal-rs"))
        .arg(OsString::from_vec(vec![b'-', b'H', 0xff]))
        .env("LC_ALL", "C")
        .output()
        .expect("run Rust candidate with a grouped non-UTF-8 option");
    assert_eq!(grouped.status.code(), Some(1));
    assert_eq!(
        grouped.stderr,
        b"xfce4-terminal: Unknown option \"-\xff\"\n"
    );

    let launch = parse_launch_os(
        &[
            OsString::from("-x"),
            OsString::from("echo"),
            OsString::from_vec(vec![0xff]),
        ],
        false,
    )
    .unwrap();
    assert!(
        format_launch_specs_bytes(&launch)
            .windows(b"|command=2:4:echo:1:\xff".len())
            .any(|window| window == b"|command=2:4:echo:1:\xff")
    );

    let separate_title = parse_launch_os(
        &[OsString::from("--title"), OsString::from_vec(vec![0xff])],
        false,
    )
    .unwrap();
    assert!(
        format_launch_specs_bytes(&separate_title)
            .windows(b"|title=1:\xff".len())
            .any(|window| window == b"|title=1:\xff")
    );

    let inline_title = parse_launch_os(
        &[OsString::from_vec(vec![
            b'-', b'-', b't', b'i', b't', b'l', b'e', b'=', 0xff,
        ])],
        false,
    )
    .unwrap();
    assert!(
        format_launch_specs_bytes(&inline_title)
            .windows(b"|title=1:\xff".len())
            .any(|window| window == b"|title=1:\xff")
    );

    let delimited = parse_launch_os(
        &[OsString::from("--"), OsString::from_vec(vec![b'-', 0xff])],
        false,
    )
    .unwrap();
    assert_eq!(delimited, vec![xfce4_terminal::cli::WindowSpec::default()]);
}
