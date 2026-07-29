//! Child process behavior expected from the frozen C terminal screen.
//!
//! Every expectation here was read from `tests/reference/child-probe.c` running
//! against the frozen reference. `tests/reference/child-process.sh` repeats the
//! comparison over the whole scenario corpus and over several login shells;
//! these tests keep the interesting cases readable and available without a
//! reference build.
//!
//! The login shell a host resolves is its own, so the cases below supply a
//! fixed one. Which shell the reference picks, and in which order it looks, is
//! what the shell script compares.

use std::ffi::{OsStr, OsString};

use xfce4_terminal::child::{
    self, ChildCommandError, CommandPreferences, FALLBACK_SHELLS, Toplevel,
};

/// The shell the readable cases resolve to, so they hold on any host.
const SHELL: &str = "/bin/sh";

fn preferences(login_shell: bool, run_custom_command: bool, custom: &str) -> CommandPreferences {
    CommandPreferences {
        login_shell,
        run_custom_command,
        custom_command: custom.to_owned(),
    }
}

fn spawn_argv(
    screen_command: Option<&[OsString]>,
    preferences: &CommandPreferences,
) -> Result<Vec<String>, ChildCommandError> {
    child::child_command_with(screen_command, preferences, || Ok(OsString::from(SHELL))).map(
        |command| {
            command
                .spawn_argv()
                .iter()
                .map(|argument| argument.to_string_lossy().into_owned())
                .collect()
        },
    )
}

fn screen_command(arguments: &[&str]) -> Vec<OsString> {
    arguments.iter().map(OsString::from).collect()
}

#[test]
fn a_plain_screen_starts_the_login_shell_under_its_own_name() {
    assert_eq!(
        spawn_argv(None, &preferences(false, false, "")),
        Ok(vec!["/bin/sh".to_owned(), "sh".to_owned()])
    );
}

#[test]
fn a_login_shell_gains_a_leading_dash_in_its_first_argument() {
    assert_eq!(
        spawn_argv(None, &preferences(true, false, "")),
        Ok(vec!["/bin/sh".to_owned(), "-sh".to_owned()])
    );
}

#[test]
fn the_custom_command_preference_keeps_only_its_arguments() {
    // The command is the word the preference names, but the first argument
    // becomes that word's base name, so a child sees the same argument zero it
    // would see from a shell.
    assert_eq!(
        spawn_argv(None, &preferences(false, true, "/bin/echo hello world")),
        Ok(vec![
            "/bin/echo".to_owned(),
            "echo".to_owned(),
            "hello".to_owned(),
            "world".to_owned(),
        ])
    );
}

#[test]
fn a_custom_command_preference_also_gains_the_login_dash() {
    assert_eq!(
        spawn_argv(None, &preferences(true, true, "/bin/echo hello")),
        Ok(vec![
            "/bin/echo".to_owned(),
            "-echo".to_owned(),
            "hello".to_owned(),
        ])
    );
}

#[test]
fn a_relative_custom_command_is_left_for_the_path_search() {
    assert_eq!(
        spawn_argv(None, &preferences(false, true, "echo")),
        Ok(vec!["echo".to_owned(), "echo".to_owned()])
    );
}

#[test]
fn the_custom_command_preference_is_split_the_way_a_shell_would() {
    assert_eq!(
        spawn_argv(
            None,
            &preferences(false, true, "echo \"double\" 'single' bare")
        ),
        Ok(vec![
            "echo".to_owned(),
            "echo".to_owned(),
            "double".to_owned(),
            "single".to_owned(),
            "bare".to_owned(),
        ])
    );
    // A quoted word may hold a separator, and its base name is the whole word.
    assert_eq!(
        spawn_argv(None, &preferences(false, true, "'quoted command' argument")),
        Ok(vec![
            "quoted command".to_owned(),
            "quoted command".to_owned(),
            "argument".to_owned(),
        ])
    );
    // Everything from an unquoted number sign on is a comment.
    assert_eq!(
        spawn_argv(None, &preferences(false, true, "echo #comment")),
        Ok(vec!["echo".to_owned(), "echo".to_owned()])
    );
}

#[test]
fn the_base_name_of_a_directory_like_command_drops_its_trailing_slash() {
    assert_eq!(
        spawn_argv(None, &preferences(false, true, "/bin/")),
        Ok(vec!["/bin/".to_owned(), "bin".to_owned()])
    );
    assert_eq!(
        spawn_argv(None, &preferences(true, true, "/bin/")),
        Ok(vec!["/bin/".to_owned(), "-bin".to_owned()])
    );
    // The root directory is its own base name.
    assert_eq!(
        spawn_argv(None, &preferences(false, true, "/")),
        Ok(vec!["/".to_owned(), "/".to_owned()])
    );
    assert_eq!(
        spawn_argv(None, &preferences(true, true, "/")),
        Ok(vec!["/".to_owned(), "-/".to_owned()])
    );
}

#[test]
fn a_custom_command_preference_holding_no_words_is_refused() {
    for custom in ["", " ", "\t"] {
        assert_eq!(
            spawn_argv(None, &preferences(false, true, custom)),
            Err(ChildCommandError::EmptyCustomCommand),
            "custom command {custom:?}"
        );
    }
    assert_eq!(
        ChildCommandError::EmptyCustomCommand.message(),
        "Empty custom command in the terminal preferences"
    );
}

#[test]
fn an_unsplittable_custom_command_preference_reports_the_glib_message() {
    let error = spawn_argv(None, &preferences(false, true, "echo 'unbalanced"))
        .expect_err("an unbalanced quote cannot be split");

    assert_eq!(
        error.message(),
        "Text ended before matching quote was found for '. \
         (The text was \u{201c}echo 'unbalanced\u{201d})"
    );

    let error = spawn_argv(None, &preferences(false, true, "echo \\"))
        .expect_err("a trailing escape cannot be split");

    assert_eq!(
        error.message(),
        "Text ended just after a \u{201c}\\\u{201d} character. \
         (The text was \u{201c}echo \\\u{201d})"
    );
}

#[test]
fn the_custom_command_preference_is_ignored_unless_it_is_enabled() {
    assert_eq!(
        spawn_argv(None, &preferences(false, false, "/bin/echo ignored")),
        Ok(vec!["/bin/sh".to_owned(), "sh".to_owned()])
    );
}

#[test]
fn a_screen_with_its_own_command_runs_it_unchanged() {
    // Neither the base name nor the login dash applies here: the argument
    // vector reaches the child exactly as the option parser produced it, and
    // the command repeats its first entry.
    let command = screen_command(&["/bin/ls", "-l", "a b"]);

    assert_eq!(
        spawn_argv(Some(&command), &preferences(true, true, "/bin/echo unused")),
        Ok(vec![
            "/bin/ls".to_owned(),
            "/bin/ls".to_owned(),
            "-l".to_owned(),
            "a b".to_owned(),
        ])
    );
}

#[test]
fn an_empty_screen_command_falls_back_to_the_preferences() {
    assert_eq!(
        spawn_argv(Some(&[]), &preferences(false, false, "")),
        Ok(vec!["/bin/sh".to_owned(), "sh".to_owned()])
    );
}

#[test]
fn a_screen_command_needs_no_login_shell_at_all() {
    // The reference never looks a shell up on this path, so a host without one
    // still starts the command it was asked for.
    let command = screen_command(&["true"]);
    let resolved =
        child::child_command_with(Some(&command), &preferences(false, false, ""), || {
            Err(ChildCommandError::NoLoginShell)
        });

    assert_eq!(
        resolved.map(|command| command.spawn_argv()),
        Ok(vec![OsString::from("true"), OsString::from("true")])
    );
}

#[test]
fn the_spawn_request_names_its_first_argument_as_the_file() {
    let command = child::child_command_with(None, &preferences(false, false, ""), || {
        Ok(OsString::from(SHELL))
    })
    .expect("a fixed shell resolves");

    assert_eq!(
        command.spawn_flags(),
        glib::SpawnFlags::SEARCH_PATH | glib::SpawnFlags::FILE_AND_ARGV_ZERO
    );
    assert_eq!(child::SPAWN_TIMEOUT_MS, 30_000);
    assert_eq!(child::PTY_FLAGS, zoha_vte::PtyFlags::DEFAULT);
}

#[test]
fn the_fallback_shells_are_tried_in_the_frozen_order() {
    assert_eq!(
        FALLBACK_SHELLS,
        [
            "/bin/sh",
            "/bin/bash",
            "/usr/bin/bash",
            "/bin/dash",
            "/usr/bin/dash",
            "/bin/zsh",
            "/usr/bin/zsh",
            "/bin/tcsh",
            "/usr/bin/tcsh",
            "/bin/csh",
            "/usr/bin/csh",
            "/bin/ksh",
            "/usr/bin/ksh",
        ]
    );
}

#[test]
fn a_host_without_any_shell_reports_the_frozen_message() {
    assert_eq!(
        ChildCommandError::NoLoginShell.message(),
        "Unable to determine your login shell."
    );
}

#[test]
fn the_child_environment_hides_the_variables_the_terminal_owns() {
    let environment = child::child_environment(&Toplevel::Unrealized);

    for hidden in [
        "COLUMNS",
        "LINES",
        "WINDOWID",
        "GNOME_DESKTOP_ICON",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "TERM",
    ] {
        assert!(
            !environment.iter().any(|entry| named(entry, hidden)),
            "{hidden} reached the child"
        );
    }
}

#[test]
fn the_child_environment_names_the_terminal_last() {
    let environment = child::child_environment(&Toplevel::Unrealized);

    assert_eq!(
        environment.last().map(OsString::as_os_str),
        Some(OsStr::new("COLORTERM=xfce4-terminal"))
    );
    assert_eq!(
        environment
            .iter()
            .filter(|entry| named(entry, "COLORTERM"))
            .count(),
        1
    );
}

#[test]
fn the_child_environment_keeps_the_inherited_variables_in_order() {
    let environment = child::child_environment(&Toplevel::Unrealized);
    let inherited: Vec<_> = environment
        .iter()
        .filter(|entry| !named(entry, "COLORTERM"))
        .cloned()
        .collect();
    let expected: Vec<_> = glib::listenv()
        .into_iter()
        .filter(|name| {
            ![
                "COLUMNS",
                "LINES",
                "WINDOWID",
                "GNOME_DESKTOP_ICON",
                "COLORTERM",
                "DISPLAY",
                "WAYLAND_DISPLAY",
                "TERM",
            ]
            .contains(&name.to_string_lossy().as_ref())
        })
        .filter_map(|name| {
            glib::getenv(&name).map(|value| {
                let mut entry = name;
                entry.push("=");
                entry.push(value);
                entry
            })
        })
        .collect();

    assert_eq!(inherited, expected);
}

#[test]
fn a_realized_toplevel_adds_the_display_it_belongs_to() {
    let x11 = child::child_environment(&Toplevel::X11 {
        window: 2_097_156,
        display: ":100".to_owned(),
    });

    assert_eq!(
        x11.iter().rev().take(3).collect::<Vec<_>>(),
        [
            &OsString::from("DISPLAY=:100"),
            &OsString::from("WINDOWID=2097156"),
            &OsString::from("COLORTERM=xfce4-terminal"),
        ]
    );

    let wayland = child::child_environment(&Toplevel::Wayland {
        display: "wayland-0".to_owned(),
    });

    assert_eq!(
        wayland.iter().rev().take(2).collect::<Vec<_>>(),
        [
            &OsString::from("WAYLAND_DISPLAY=wayland-0"),
            &OsString::from("COLORTERM=xfce4-terminal"),
        ]
    );
}

fn named(entry: &OsStr, name: &str) -> bool {
    let entry = entry.as_encoded_bytes();
    entry.starts_with(name.as_bytes()) && entry.get(name.len()) == Some(&b'=')
}
