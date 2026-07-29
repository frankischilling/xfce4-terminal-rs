//! Screen state that does not need a running child.
//!
//! Titles, the working directory, paste safety, and the colors handed to VTE
//! are decided from preferences and a few values the widget layer already
//! knows. The functions here take those values explicitly so the same logic
//! can be compared with the frozen C helpers without realizing a full window
//! tree in every unit test.

use gdk::RGBA;

use crate::ffi::glib as glib_wrapper;

/// The C-locale spelling of the fallback window title.
///
/// The frozen screen asks gettext for this string. Under `LC_ALL=C`, and in the
/// unit tests that mirror that locale, the translated text is unchanged.
pub const UNTITLED: &str = "Untitled";

/// How a dynamic window title combines with the initial title.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleMode {
    Replace,
    Prepend,
    Append,
    Hide,
}

impl TitleMode {
    /// Parses the C enumeration nick used in preferences and fixtures.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "TERMINAL_TITLE_REPLACE" => Some(Self::Replace),
            "TERMINAL_TITLE_PREPEND" => Some(Self::Prepend),
            "TERMINAL_TITLE_APPEND" => Some(Self::Append),
            "TERMINAL_TITLE_HIDE" => Some(Self::Hide),
            _ => None,
        }
    }

    /// Returns the C enumeration nick.
    pub fn nick(self) -> &'static str {
        match self {
            Self::Replace => "TERMINAL_TITLE_REPLACE",
            Self::Prepend => "TERMINAL_TITLE_PREPEND",
            Self::Append => "TERMINAL_TITLE_APPEND",
            Self::Hide => "TERMINAL_TITLE_HIDE",
        }
    }
}

/// Expands `%#`, `%d`, `%D`, and `%w` the way `terminal_screen_parse_title` does.
///
/// A missing template becomes an empty string. A missing window title becomes
/// [`UNTITLED`]. Unknown percent sequences keep the percent and leave the next
/// character for the normal walk, which is how a doubled percent escapes.
pub fn parse_title(
    title: Option<&str>,
    session_id: u32,
    working_directory: Option<&str>,
    vte_title: Option<&str>,
) -> String {
    let Some(title) = title else {
        return String::new();
    };

    let mut result = String::new();
    let bytes = title.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let Some(relative) = title[index..].find('%') else {
            result.push_str(&title[index..]);
            break;
        };
        let percent = index + relative;
        result.push_str(&title[index..percent]);
        let next = percent + 1;
        if next >= bytes.len() {
            result.push('%');
            break;
        }
        match bytes[next] as char {
            '#' => {
                result.push_str(&session_id.to_string());
                index = next + 1;
            }
            'd' | 'D' => {
                if let Some(path) = working_directory {
                    if bytes[next] == b'D' {
                        result.push_str(path);
                    } else {
                        result.push_str(
                            &glib_wrapper::path_basename(std::ffi::OsStr::new(path))
                                .to_string_lossy(),
                        );
                    }
                }
                index = next + 1;
            }
            'w' => {
                result.push_str(vte_title.unwrap_or(UNTITLED));
                index = next + 1;
            }
            _ => {
                result.push('%');
                index = next;
            }
        }
    }

    result
}

/// Builds the title `terminal_screen_get_title` would return.
///
/// A custom title wins over every dynamic mode. Otherwise the initial title is
/// the screen's own override, or the preference when that override is missing.
pub fn screen_title(
    custom_title: Option<&str>,
    initial_title: Option<&str>,
    preference_initial: &str,
    mode: TitleMode,
    session_id: u32,
    working_directory: Option<&str>,
    vte_title: Option<&str>,
) -> String {
    if let Some(custom_title) = custom_title {
        return parse_title(Some(custom_title), session_id, working_directory, vte_title);
    }

    let initial_source = initial_title.unwrap_or(preference_initial);
    let initial = parse_title(
        Some(initial_source),
        session_id,
        working_directory,
        vte_title,
    );

    match mode {
        TitleMode::Replace => match vte_title {
            Some(title) => title.to_owned(),
            // parse_title always yields a string, so the reference's Untitled
            // branch for a null initial is unreachable from this path.
            None => initial,
        },
        TitleMode::Prepend => match vte_title {
            Some(title) => format!("{title} - {initial}"),
            None => initial,
        },
        TitleMode::Append => match vte_title {
            Some(title) => format!("{initial} - {title}"),
            None => initial,
        },
        TitleMode::Hide => initial,
    }
}

/// Returns whether paste text needs the unsafe-paste confirmation dialog.
pub fn is_text_unsafe(text: Option<&str>) -> bool {
    text.is_some_and(|text| text.contains('\n') || text.contains('\r'))
}

/// Resolves the working directory the way `terminal_screen_get_working_directory`
/// does once the widget layer has asked VTE and the process table.
///
/// A directory URI wins over a process cwd. Either one replaces the stored
/// value; when both are missing the stored value is kept.
pub fn resolve_working_directory(
    stored: Option<&str>,
    directory_uri: Option<&str>,
    process_cwd: Option<&str>,
) -> Option<String> {
    if let Some(uri) = directory_uri {
        if let Some(path) = glib_wrapper::filename_from_uri(uri) {
            return Some(path);
        }
    } else if let Some(cwd) = process_cwd {
        return Some(cwd.to_owned());
    }
    stored.map(str::to_owned)
}

/// Preference and tab values that decide the colors handed to VTE.
#[derive(Clone, Debug)]
pub struct ColorInputs {
    pub palette: Option<String>,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub use_theme: bool,
    pub background_vary: bool,
    pub custom_foreground: Option<String>,
    pub custom_background: Option<String>,
    /// RGB remembered by a previous vary-background pass, if any.
    pub stored_background: Option<RGBA>,
    pub cursor_use_default: bool,
    pub cursor_foreground: Option<String>,
    pub cursor: Option<String>,
    pub selection_use_default: bool,
    pub selection: Option<String>,
    pub selection_background: Option<String>,
    pub bold_use_default: bool,
    pub bold: Option<String>,
    pub bold_is_bright: bool,
    pub theme_foreground: RGBA,
    pub theme_background: RGBA,
}

impl ColorInputs {
    /// Defaults matching an untouched preference channel with theme colors
    /// supplied by the caller as opaque stand-ins.
    pub fn defaults() -> Self {
        Self {
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
            theme_foreground: RGBA::WHITE,
            theme_background: RGBA::BLACK,
        }
    }
}

/// The colors `terminal_screen_update_colors` would pass to VTE.
#[derive(Clone, Debug, PartialEq)]
pub struct AppliedColors {
    pub foreground: Option<RGBA>,
    pub background: Option<RGBA>,
    pub palette: Option<[RGBA; 16]>,
    pub used_default_palette: bool,
    pub cursor_configured: bool,
    pub cursor_foreground: Option<RGBA>,
    pub cursor: Option<RGBA>,
    pub selection_configured: bool,
    pub selection_foreground: Option<RGBA>,
    pub selection_background: Option<RGBA>,
    pub bold: Option<RGBA>,
    pub bold_is_bright: bool,
}

/// Computes the VTE color arguments for one preference and tab color set.
///
/// Random background variation is intentionally out of scope: fixtures keep
/// `color-background-vary` off, and a stored background is only reused when
/// that preference is already on from an earlier pass.
pub fn resolve_colors(inputs: &ColorInputs) -> AppliedColors {
    let mut palette = [RGBA::BLACK; 16];
    let valid_palette = match inputs.palette.as_deref() {
        Some(palette_str) => parse_palette(palette_str, &mut palette),
        None => false,
    };

    let (has_fg, fg) = match inputs.custom_foreground.as_deref() {
        Some(spec) => match RGBA::parse(spec) {
            Ok(color) => (true, color),
            Err(_) => (false, inputs.theme_foreground),
        },
        None => {
            let parsed = inputs
                .foreground
                .as_deref()
                .and_then(|spec| RGBA::parse(spec).ok());
            match (inputs.use_theme, parsed) {
                (true, _) | (false, None) => (true, inputs.theme_foreground),
                (false, Some(color)) => (true, color),
            }
        }
    };

    let (has_bg, background) = match inputs.custom_background.as_deref() {
        Some(spec) => match RGBA::parse(spec) {
            Ok(mut color) => {
                color.set_alpha(1.0);
                (true, Some(color))
            }
            Err(_) => (false, None),
        },
        None => {
            let mut bg = match (
                inputs.use_theme,
                inputs
                    .background
                    .as_deref()
                    .and_then(|spec| RGBA::parse(spec).ok()),
            ) {
                (true, _) | (false, None) => inputs.theme_background,
                (false, Some(color)) => color,
            };
            bg.set_alpha(1.0);

            if inputs.background_vary {
                if let Some(stored) = inputs.stored_background {
                    (true, Some(with_alpha(stored, 1.0)))
                } else {
                    // A fresh random hue is not compared here. Callers that need
                    // the vary path supply a stored background from an earlier
                    // pass instead of asking this function to invent one.
                    (true, Some(bg))
                }
            } else {
                (true, Some(bg))
            }
        }
    };

    let mut cursor_foreground = None;
    let mut cursor = None;
    if !inputs.cursor_use_default {
        cursor_foreground = inputs
            .cursor_foreground
            .as_deref()
            .and_then(|spec| RGBA::parse(spec).ok());
        cursor = inputs
            .cursor
            .as_deref()
            .and_then(|spec| RGBA::parse(spec).ok());
    }

    let mut selection_foreground = None;
    let mut selection_background = None;
    if !inputs.selection_use_default {
        selection_foreground = inputs
            .selection
            .as_deref()
            .and_then(|spec| RGBA::parse(spec).ok());
        selection_background = inputs
            .selection_background
            .as_deref()
            .and_then(|spec| RGBA::parse(spec).ok());
    }

    let bold = if inputs.bold_use_default {
        None
    } else {
        inputs
            .bold
            .as_deref()
            .and_then(|spec| RGBA::parse(spec).ok())
    };

    AppliedColors {
        foreground: has_fg.then_some(fg),
        background: has_bg.then_some(background).flatten(),
        palette: valid_palette.then_some(palette),
        used_default_palette: !valid_palette,
        cursor_configured: !inputs.cursor_use_default,
        cursor_foreground,
        cursor,
        selection_configured: !inputs.selection_use_default,
        selection_foreground,
        selection_background,
        bold,
        bold_is_bright: inputs.bold_is_bright,
    }
}

fn parse_palette(palette_str: &str, palette: &mut [RGBA; 16]) -> bool {
    let mut count = 0;
    for (index, color) in palette_str.split(';').enumerate() {
        if index >= 16 {
            break;
        }
        match RGBA::parse(color) {
            Ok(parsed) => {
                palette[index] = parsed;
                count += 1;
            }
            Err(_) => return false,
        }
    }
    count == 16
}

fn with_alpha(color: RGBA, alpha: f64) -> RGBA {
    let mut color = color;
    color.set_alpha(alpha);
    color
}
