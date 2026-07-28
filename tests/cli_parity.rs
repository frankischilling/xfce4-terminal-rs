use std::ffi::OsStr;

use xfce4_terminal::cli::{
    DynamicTitleMode, ImmediateAction, TabSpec, Visibility, WindowSpec, color_table, help_text,
    parse_immediate, parse_launch, version_text,
};

#[test]
fn no_arguments_create_one_default_window_and_tab() {
    assert_eq!(
        parse_launch(&[], false).unwrap(),
        vec![WindowSpec::default()]
    );
}

#[test]
fn defaults_apply_only_where_a_specific_value_is_missing() {
    let windows = parse_launch(
        &[
            "--default-display=:7",
            "--default-working-directory=/tmp",
            "--title=first",
            "--tab",
            "--working-directory=/var",
            "--window",
            "--display=:8",
        ],
        false,
    )
    .unwrap();

    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].display.as_deref(), Some(OsStr::new(":7")));
    assert_eq!(
        windows[0].tabs[0].directory.as_deref(),
        Some(OsStr::new("/tmp"))
    );
    assert_eq!(
        windows[0].tabs[0].title.as_deref(),
        Some(OsStr::new("first"))
    );
    assert_eq!(
        windows[0].tabs[1].directory.as_deref(),
        Some(OsStr::new("/var"))
    );
    assert_eq!(windows[1].display.as_deref(), Some(OsStr::new(":8")));
    assert_eq!(
        windows[1].tabs[0].directory.as_deref(),
        Some(OsStr::new("/tmp"))
    );
}

#[test]
fn first_tab_reuses_an_active_window_but_window_starts_a_new_one() {
    let windows = parse_launch(
        &["--tab", "--title=reused", "--window", "--title=new"],
        true,
    )
    .unwrap();

    assert_eq!(windows.len(), 2);
    assert!(windows[0].reuse_last_window);
    assert_eq!(windows[0].tabs.len(), 1);
    assert_eq!(
        windows[0].tabs[0].title.as_deref(),
        Some(OsStr::new("reused"))
    );
    assert_eq!(windows[1].tabs[0].title.as_deref(), Some(OsStr::new("new")));
}

#[test]
fn grouped_short_options_and_execute_match_the_c_rules() {
    let windows = parse_launch(&["-HTgrouped", "-x", "printf", "%s", "hello"], false).unwrap();
    let tab = &windows[0].tabs[0];

    assert!(tab.hold);
    assert_eq!(tab.title.as_deref(), Some(OsStr::new("grouped")));
    assert_eq!(
        tab.command
            .as_ref()
            .unwrap()
            .iter()
            .map(|argument| argument.to_str().unwrap())
            .collect::<Vec<_>>(),
        ["printf", "%s", "hello"]
    );

    let grouped_execute = parse_launch(&["-Hx", "echo"], false).unwrap();
    assert!(grouped_execute[0].tabs[0].hold);
    assert_eq!(
        grouped_execute[0].tabs[0].command.as_deref(),
        Some([std::ffi::OsString::from("echo")].as_slice())
    );
    assert!(parse_launch(&["-xecho"], false).is_err());
}

#[test]
fn command_strings_use_glib_shell_syntax() {
    let windows = parse_launch(
        &[
            "--command=printf '%s %s' \"hello world\" done",
            "--dynamic-title-mode=before",
            "--active-tab",
            "--show-menubar",
        ],
        false,
    )
    .unwrap();

    assert_eq!(
        windows[0].tabs[0]
            .command
            .as_ref()
            .unwrap()
            .iter()
            .map(|argument| argument.to_str().unwrap())
            .collect::<Vec<_>>(),
        ["printf", "%s %s", "hello world", "done"]
    );
    assert_eq!(
        windows[0].tabs[0].dynamic_title_mode,
        DynamicTitleMode::Prepend
    );
    assert!(windows[0].tabs[0].active);
    assert_eq!(windows[0].menubar, Visibility::Show);
}

#[test]
fn invalid_zoom_and_unknown_options_report_reference_errors() {
    assert_eq!(
        parse_launch(&["--zoom=8"], false).unwrap_err().to_string(),
        "Option \"--zoom\" requires specifying the zoom (-7 .. 7) as its parameter"
    );
    assert_eq!(
        parse_launch(&["--wat"], false).unwrap_err().to_string(),
        "Unknown option \"--wat\""
    );
}

#[test]
fn tab_default_matches_the_reference() {
    assert_eq!(WindowSpec::default().tabs, vec![TabSpec::default()]);
}

#[test]
fn immediate_options_stop_at_execute_and_the_option_delimiter() {
    assert_eq!(
        parse_immediate(&["--disable-server", "-V"]).action,
        Some(ImmediateAction::Version)
    );
    assert_eq!(
        parse_immediate(&["-hV"]).action,
        Some(ImmediateAction::Help)
    );
    assert!(!parse_immediate(&["-x", "--help"]).disable_server);
    assert_eq!(parse_immediate(&["-xecho", "--help"]).action, None);
    assert_eq!(parse_immediate(&["--", "--help"]).action, None);
}

#[test]
fn unknown_short_options_preserve_the_reference_argument_boundary() {
    assert_eq!(
        parse_launch(&["-é"], false).unwrap_err().as_bytes(),
        b"Unknown option \"-\xc3\xa9\""
    );
    assert_eq!(
        parse_launch(&["-Hé"], false).unwrap_err().as_bytes(),
        b"Unknown option \"-\xc3\""
    );
}

#[test]
fn reference_text_outputs_keep_the_public_command_name() {
    assert!(help_text().starts_with("Usage:\n  xfce4-terminal [OPTION...]\n\n"));
    assert!(help_text().ends_with(
        "See the xfce4-terminal man page for full explanation of the options above.\n\n"
    ));
    assert!(
        version_text("4.20").starts_with(
            "xfce4-terminal 1.2.0-dev-b5933b80 (Xfce 4.20)\n\nCopyright (c) 2003-2026\n"
        )
    );
    assert!(color_table().starts_with("       |          40m    41m    42m "));
    assert!(color_table().ends_with("\u{1b}[0m\n"));
}
