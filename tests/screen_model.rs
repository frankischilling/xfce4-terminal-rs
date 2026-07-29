//! Screen model behavior expected from the frozen C terminal screen.
//!
//! Every expectation here was read from `tests/reference/screen-probe.c`
//! running against the frozen reference. `tests/reference/screen-model.sh`
//! repeats the comparison over the whole scenario corpus; these tests keep the
//! interesting cases readable without a reference build.

use gdk::RGBA;
use xfce4_terminal::screen::{self, AppliedColors, ColorInputs, TitleMode, UNTITLED};

#[test]
fn a_null_title_template_becomes_an_empty_string() {
    assert_eq!(screen::parse_title(None, 1, Some("/tmp"), None), "");
}

#[test]
fn session_id_directory_and_window_title_expand() {
    assert_eq!(
        screen::parse_title(
            Some("tab %# in %d (%D) — %w"),
            7,
            Some("/home/frank/xfce-rust"),
            Some("vim"),
        ),
        "tab 7 in xfce-rust (/home/frank/xfce-rust) — vim"
    );
}

#[test]
fn a_missing_window_title_uses_the_untitled_fallback() {
    assert_eq!(screen::parse_title(Some("%w"), 1, None, None), UNTITLED);
}

#[test]
fn an_unknown_percent_sequence_keeps_the_percent_and_the_next_character() {
    assert_eq!(
        screen::parse_title(Some("a%x%%d"), 1, Some("/tmp/dir"), None),
        "a%x%dir"
    );
}

#[test]
fn a_trailing_percent_is_kept() {
    assert_eq!(screen::parse_title(Some("done%"), 1, None, None), "done%");
}

#[test]
fn a_custom_title_ignores_the_dynamic_mode() {
    assert_eq!(
        screen::screen_title(
            Some("Custom %d"),
            None,
            "Terminal",
            TitleMode::Replace,
            3,
            Some("/var/tmp"),
            Some("ignored"),
        ),
        "Custom tmp"
    );
}

#[test]
fn replace_mode_prefers_the_window_title() {
    assert_eq!(
        screen::screen_title(
            None,
            None,
            "Terminal",
            TitleMode::Replace,
            1,
            Some("/tmp"),
            Some("shell"),
        ),
        "shell"
    );
}

#[test]
fn replace_mode_falls_back_to_the_parsed_initial_title() {
    assert_eq!(
        screen::screen_title(
            None,
            Some("Init %d"),
            "Terminal",
            TitleMode::Replace,
            1,
            Some("/tmp"),
            None,
        ),
        "Init tmp"
    );
}

#[test]
fn append_mode_joins_with_a_spaced_dash() {
    assert_eq!(
        screen::screen_title(
            None,
            None,
            "Terminal",
            TitleMode::Append,
            1,
            Some("/tmp"),
            Some("shell"),
        ),
        "Terminal - shell"
    );
}

#[test]
fn prepend_mode_puts_the_window_title_first() {
    assert_eq!(
        screen::screen_title(
            None,
            None,
            "Terminal",
            TitleMode::Prepend,
            1,
            Some("/tmp"),
            Some("shell"),
        ),
        "shell - Terminal"
    );
}

#[test]
fn hide_mode_keeps_only_the_initial_title() {
    assert_eq!(
        screen::screen_title(
            None,
            None,
            "Terminal %#",
            TitleMode::Hide,
            9,
            Some("/tmp"),
            Some("shell"),
        ),
        "Terminal 9"
    );
}

#[test]
fn append_mode_without_a_window_title_returns_only_the_initial_title() {
    assert_eq!(
        screen::screen_title(
            None,
            None,
            "Terminal",
            TitleMode::Append,
            1,
            Some("/tmp"),
            None,
        ),
        "Terminal"
    );
}

#[test]
fn text_is_unsafe_when_it_contains_a_newline_or_carriage_return() {
    assert!(!screen::is_text_unsafe(None));
    assert!(!screen::is_text_unsafe(Some("safe")));
    assert!(screen::is_text_unsafe(Some("line\n")));
    assert!(screen::is_text_unsafe(Some("line\r")));
    assert!(screen::is_text_unsafe(Some("a\nb\rc")));
}

#[test]
fn a_stored_working_directory_is_kept_when_nothing_else_answers() {
    assert_eq!(
        screen::resolve_working_directory(Some("/stored"), None, None).as_deref(),
        Some("/stored")
    );
}

#[test]
fn a_directory_uri_replaces_the_stored_working_directory() {
    assert_eq!(
        screen::resolve_working_directory(Some("/stored"), Some("file:///tmp/from-uri"), None,)
            .as_deref(),
        Some("/tmp/from-uri")
    );
}

#[test]
fn a_directory_uri_that_cannot_be_converted_leaves_the_stored_directory() {
    // An invalid percent escape defeats g_filename_from_uri the same way a
    // remote host would: the conversion fails and the stored directory stays.
    assert_eq!(
        screen::resolve_working_directory(Some("/stored"), Some("file:///tmp/%zz"), None)
            .as_deref(),
        Some("/stored")
    );
}

#[test]
fn a_process_cwd_is_used_only_when_no_directory_uri_is_present() {
    assert_eq!(
        screen::resolve_working_directory(Some("/stored"), None, Some("/proc/cwd")).as_deref(),
        Some("/proc/cwd")
    );
    assert_eq!(
        screen::resolve_working_directory(
            Some("/stored"),
            Some("file:///tmp/from-uri"),
            Some("/proc/cwd"),
        )
        .as_deref(),
        Some("/tmp/from-uri")
    );
}

#[test]
fn the_default_palette_and_colors_are_applied() {
    let applied = screen::resolve_colors(&ColorInputs {
        palette: Some(
            "#000000;#aa0000;#00aa00;#aa5500;#0000aa;#aa00aa;#00aaaa;#aaaaaa;#555555;#ff5555;#55ff55;#ffff55;#5555ff;#ff55ff;#55ffff;#ffffff"
                .to_owned(),
        ),
        foreground: Some("#ffffff".to_owned()),
        background: Some("#000000".to_owned()),
        use_theme: false,
        background_vary: false,
        custom_foreground: None,
        custom_background: None,
        stored_background: None,
        cursor_use_default: true,
        cursor_foreground: None,
        cursor: None,
        selection_use_default: true,
        selection: None,
        selection_background: None,
        bold_use_default: true,
        bold: None,
        bold_is_bright: true,
        theme_foreground: RGBA::new(0.1, 0.2, 0.3, 1.0),
        theme_background: RGBA::new(0.4, 0.5, 0.6, 1.0),
    });

    assert!(!applied.used_default_palette);
    assert_eq!(applied.foreground, Some(RGBA::parse("#ffffff").unwrap()));
    assert_eq!(
        applied
            .background
            .map(|color| (color.red(), color.green(), color.blue(), color.alpha())),
        Some((0.0, 0.0, 0.0, 1.0))
    );
    assert_eq!(
        applied.palette.as_ref().map(|palette| palette.len()),
        Some(16)
    );
    assert!(applied.cursor_foreground.is_none());
    assert!(applied.cursor.is_none());
    assert!(applied.selection_foreground.is_none());
    assert!(applied.selection_background.is_none());
    assert!(applied.bold.is_none());
    assert!(applied.bold_is_bright);
}

#[test]
fn an_unparseable_palette_falls_back_to_the_default_palette() {
    let applied = screen::resolve_colors(&ColorInputs {
        palette: Some("#000000;#bad".to_owned()),
        foreground: Some("#ffffff".to_owned()),
        background: Some("#000000".to_owned()),
        use_theme: false,
        background_vary: false,
        custom_foreground: None,
        custom_background: None,
        stored_background: None,
        cursor_use_default: true,
        cursor_foreground: None,
        cursor: None,
        selection_use_default: true,
        selection: None,
        selection_background: None,
        bold_use_default: true,
        bold: None,
        bold_is_bright: false,
        theme_foreground: RGBA::WHITE,
        theme_background: RGBA::BLACK,
    });

    assert!(applied.used_default_palette);
    assert!(applied.palette.is_none());
    assert!(!applied.bold_is_bright);
}

#[test]
fn custom_tab_colors_override_the_preference_colors() {
    let applied = screen::resolve_colors(&ColorInputs {
        palette: Some(
            "#000000;#aa0000;#00aa00;#aa5500;#0000aa;#aa00aa;#00aaaa;#aaaaaa;#555555;#ff5555;#55ff55;#ffff55;#5555ff;#ff55ff;#55ffff;#ffffff"
                .to_owned(),
        ),
        foreground: Some("#ffffff".to_owned()),
        background: Some("#000000".to_owned()),
        use_theme: false,
        background_vary: false,
        custom_foreground: Some("#112233".to_owned()),
        custom_background: Some("#445566".to_owned()),
        stored_background: None,
        cursor_use_default: true,
        cursor_foreground: None,
        cursor: None,
        selection_use_default: true,
        selection: None,
        selection_background: None,
        bold_use_default: true,
        bold: None,
        bold_is_bright: true,
        theme_foreground: RGBA::WHITE,
        theme_background: RGBA::BLACK,
    });

    assert_eq!(applied.foreground, Some(RGBA::parse("#112233").unwrap()));
    assert_eq!(
        applied.background.map(|color| (
            (color.red() * 255.0).round() as u8,
            (color.green() * 255.0).round() as u8,
            (color.blue() * 255.0).round() as u8,
        )),
        Some((0x44, 0x55, 0x66))
    );
}

#[test]
fn explicit_cursor_selection_and_bold_colors_are_applied() {
    let applied = screen::resolve_colors(&ColorInputs {
        palette: Some(
            "#000000;#aa0000;#00aa00;#aa5500;#0000aa;#aa00aa;#00aaaa;#aaaaaa;#555555;#ff5555;#55ff55;#ffff55;#5555ff;#ff55ff;#55ffff;#ffffff"
                .to_owned(),
        ),
        foreground: Some("#ffffff".to_owned()),
        background: Some("#000000".to_owned()),
        use_theme: false,
        background_vary: false,
        custom_foreground: None,
        custom_background: None,
        stored_background: None,
        cursor_use_default: false,
        cursor_foreground: Some("#010203".to_owned()),
        cursor: Some("#040506".to_owned()),
        selection_use_default: false,
        selection: Some("#070809".to_owned()),
        selection_background: Some("#0a0b0c".to_owned()),
        bold_use_default: false,
        bold: Some("#0d0e0f".to_owned()),
        bold_is_bright: true,
        theme_foreground: RGBA::WHITE,
        theme_background: RGBA::BLACK,
    });

    assert_eq!(
        applied.cursor_foreground,
        Some(RGBA::parse("#010203").unwrap())
    );
    assert_eq!(applied.cursor, Some(RGBA::parse("#040506").unwrap()));
    assert_eq!(
        applied.selection_foreground,
        Some(RGBA::parse("#070809").unwrap())
    );
    assert_eq!(
        applied.selection_background,
        Some(RGBA::parse("#0a0b0c").unwrap())
    );
    assert_eq!(applied.bold, Some(RGBA::parse("#0d0e0f").unwrap()));
}

#[test]
fn applied_colors_debug_shape_stays_public() {
    // Keeps the report fields the differential probe relies on from drifting.
    let _: AppliedColors = screen::resolve_colors(&ColorInputs::defaults());
}
