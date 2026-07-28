//! Command-line data model and parser.

use std::ffi::{OsStr, OsString};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImmediateAction {
    Help,
    Version,
    ColorTable,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImmediateOptions {
    pub action: Option<ImmediateAction>,
    pub disable_server: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Visibility {
    #[default]
    Default,
    Show,
    Hide,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DynamicTitleMode {
    #[default]
    Default,
    Replace,
    Prepend,
    Append,
    Hide,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabSpec {
    pub command: Option<Vec<OsString>>,
    pub directory: Option<OsString>,
    pub title: Option<OsString>,
    pub initial_title: Option<OsString>,
    pub color_text: Option<OsString>,
    pub color_bg: Option<OsString>,
    pub color_title: Option<OsString>,
    pub dynamic_title_mode: DynamicTitleMode,
    pub position: i32,
    pub hold: bool,
    pub active: bool,
}

impl Default for TabSpec {
    fn default() -> Self {
        Self {
            command: None,
            directory: None,
            title: None,
            initial_title: None,
            color_text: None,
            color_bg: None,
            color_title: None,
            dynamic_title_mode: DynamicTitleMode::Default,
            position: -1,
            hold: false,
            active: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowSpec {
    pub tabs: Vec<TabSpec>,
    pub display: Option<OsString>,
    pub geometry: Option<OsString>,
    pub role: Option<OsString>,
    pub workspace: i32,
    pub startup_id: Option<OsString>,
    pub sm_client_id: Option<OsString>,
    pub icon: Option<OsString>,
    pub font: Option<OsString>,
    pub drop_down: bool,
    pub fullscreen: bool,
    pub maximize: bool,
    pub minimize: bool,
    pub reuse_last_window: bool,
    pub menubar: Visibility,
    pub borders: Visibility,
    pub toolbar: Visibility,
    pub scrollbar: Visibility,
    pub zoom: i32,
}

impl Default for WindowSpec {
    fn default() -> Self {
        Self {
            tabs: vec![TabSpec::default()],
            display: None,
            geometry: None,
            role: None,
            workspace: -1,
            startup_id: None,
            sm_client_id: None,
            icon: None,
            font: None,
            drop_down: false,
            fullscreen: false,
            maximize: false,
            minimize: false,
            reuse_last_window: false,
            menubar: Visibility::Default,
            borders: Visibility::Default,
            toolbar: Visibility::Default,
            scrollbar: Visibility::Default,
            zoom: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    message: Vec<u8>,
    unknown_short: Option<(Vec<u8>, usize)>,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into().into_bytes(),
            unknown_short: None,
        }
    }

    fn unknown_argument(argument: &[u8]) -> Self {
        let mut message = b"Unknown option \"".to_vec();
        message.extend_from_slice(argument);
        message.push(b'"');
        Self {
            message,
            unknown_short: None,
        }
    }

    fn unknown_short(argument: &[u8], position: usize) -> Self {
        let mut message = b"Unknown option \"-".to_vec();
        message.push(argument[position]);
        message.push(b'"');
        Self {
            message,
            unknown_short: Some((argument.to_vec(), position)),
        }
    }

    /// Returns the error text as the original command-line bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&String::from_utf8_lossy(&self.message))
    }
}

impl std::error::Error for ParseError {}

fn write_string(output: &mut Vec<u8>, name: &str, value: Option<&OsStr>) {
    use std::io::Write;

    match value {
        Some(value) => {
            write!(output, "|{name}={}:", os_string_len(value)).unwrap();
            write_os_string(output, value);
        }
        None => write!(output, "|{name}=-").unwrap(),
    }
}

fn write_os_string(output: &mut Vec<u8>, value: &OsStr) {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        output.extend_from_slice(value.as_bytes());
    }
    #[cfg(not(unix))]
    output.extend_from_slice(value.to_string_lossy().as_bytes());
}

fn os_string_len(value: &OsStr) -> usize {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        value.as_bytes().len()
    }
    #[cfg(not(unix))]
    {
        value.to_string_lossy().len()
    }
}

fn visibility_number(value: Visibility) -> u8 {
    match value {
        Visibility::Default => 0,
        Visibility::Show => 1,
        Visibility::Hide => 2,
    }
}

fn title_mode_number(value: DynamicTitleMode) -> u8 {
    match value {
        DynamicTitleMode::Replace => 0,
        DynamicTitleMode::Prepend => 1,
        DynamicTitleMode::Append => 2,
        DynamicTitleMode::Hide => 3,
        DynamicTitleMode::Default => 4,
    }
}

/// Serializes launch specifications as bytes in the shared C/Rust probe
/// format.
pub fn format_launch_specs_bytes(windows: &[WindowSpec]) -> Vec<u8> {
    use std::io::Write;

    let mut output = Vec::new();
    for window in windows {
        output.push(b'W');
        write_string(&mut output, "display", window.display.as_deref());
        write_string(&mut output, "geometry", window.geometry.as_deref());
        write_string(&mut output, "role", window.role.as_deref());
        write!(output, "|workspace={}", window.workspace).unwrap();
        write_string(&mut output, "startup_id", window.startup_id.as_deref());
        write_string(&mut output, "sm_client_id", window.sm_client_id.as_deref());
        write_string(&mut output, "icon", window.icon.as_deref());
        write_string(&mut output, "font", window.font.as_deref());
        writeln!(
            output,
            "|drop_down={}|fullscreen={}|maximize={}|minimize={}|reuse_last_window={}|menubar={}|borders={}|toolbar={}|scrollbar={}|zoom={}",
            u8::from(window.drop_down),
            u8::from(window.fullscreen),
            u8::from(window.maximize),
            u8::from(window.minimize),
            u8::from(window.reuse_last_window),
            visibility_number(window.menubar),
            visibility_number(window.borders),
            visibility_number(window.toolbar),
            visibility_number(window.scrollbar),
            window.zoom,
        )
        .unwrap();

        for tab in &window.tabs {
            output.push(b'T');
            match &tab.command {
                Some(command) => {
                    write!(output, "|command={}", command.len()).unwrap();
                    for argument in command {
                        write!(output, ":{}:", os_string_len(argument)).unwrap();
                        write_os_string(&mut output, argument);
                    }
                }
                None => output.extend_from_slice(b"|command=-"),
            }
            write_string(&mut output, "directory", tab.directory.as_deref());
            write_string(&mut output, "title", tab.title.as_deref());
            write_string(&mut output, "initial_title", tab.initial_title.as_deref());
            write_string(&mut output, "color_text", tab.color_text.as_deref());
            write_string(&mut output, "color_bg", tab.color_bg.as_deref());
            write_string(&mut output, "color_title", tab.color_title.as_deref());
            writeln!(
                output,
                "|dynamic_title_mode={}|position={}|hold={}|active={}",
                title_mode_number(tab.dynamic_title_mode),
                tab.position,
                u8::from(tab.hold),
                u8::from(tab.active),
            )
            .unwrap();
        }
    }
    output
}

/// Serializes launch specifications as UTF-8 for ordinary textual probes.
pub fn format_launch_specs(windows: &[WindowSpec]) -> String {
    String::from_utf8_lossy(&format_launch_specs_bytes(windows)).into_owned()
}

/// Parses the options handled before window and tab launch specifications.
pub fn parse_immediate(args: &[&str]) -> ImmediateOptions {
    let mut options = ImmediateOptions::default();

    for argument in args {
        if *argument == "--" || *argument == "--execute" {
            break;
        }
        if argument.starts_with('-') && !argument.starts_with("--") {
            match argument.as_bytes().get(1) {
                Some(b'x') => break,
                Some(b'h') => {
                    options.action = Some(ImmediateAction::Help);
                    break;
                }
                Some(b'V') => {
                    options.action = Some(ImmediateAction::Version);
                    break;
                }
                _ => {}
            }
        }
        match *argument {
            "--help" => {
                options.action = Some(ImmediateAction::Help);
                break;
            }
            "--version" => {
                options.action = Some(ImmediateAction::Version);
                break;
            }
            "--color-table" => {
                options.action = Some(ImmediateAction::ColorTable);
                break;
            }
            "--disable-server" => options.disable_server = true,
            _ => {}
        }
    }

    options
}

/// Returns the English command help printed in the C locale.
pub fn help_text() -> &'static str {
    concat!(
        "Usage:\n",
        "  xfce4-terminal [OPTION...]\n",
        "\n",
        "General Options:\n",
        "  -h, --help; -V, --version; --disable-server; --color-table; --preferences;\n",
        "  --default-display=display; --default-working-directory=directory\n",
        "\n",
        "Window or Tab Separators:\n",
        "  --tab; --window\n",
        "\n",
        "Tab Options:\n",
        "  -x, --execute; -e, --command=command; -T, --title=title;\n",
        "  --dynamic-title-mode=mode ('replace', 'before', 'after', 'none');\n",
        "  --initial-title=title; --working-directory=directory; -H, --hold;\n",
        "  --active-tab; --color-text=color; --color-bg=color\n",
        "\n",
        "Window Options:\n",
        "  --display=display; --geometry=geometry; --role=role; --drop-down;\n",
        "  --startup-id=string; -I, --icon=icon; --fullscreen; --maximize; --minimize;\n",
        "  --show-menubar, --hide-menubar; --show-borders, --hide-borders;\n",
        "  --show-toolbar, --hide-toolbar; --show-scrollbar, --hide-scrollbar;\n",
        "  --font=font; --zoom=zoom; --class=class\n",
        "\n",
        "See the xfce4-terminal man page for full explanation of the options above.\n",
        "\n",
    )
}

/// Returns the version and attribution text printed in the C locale.
pub fn version_text(xfce_version: &str) -> String {
    format!(
        "xfce4-terminal {}-{} (Xfce {xfce_version})\n\n\
Copyright (c) 2003-2026\n\
\tThe Xfce development team. All rights reserved.\n\n\
Written by Benedikt Meurer <benny@xfce.org>,\n\
Nick Schermer <nick@xfce.org>,\n\
Igor Zakharov <f2404@yandex.ru>,\n\
Sergios - Anestis Kefalidis <sergioskefalidis@gmail.com>.\n\n\
Please report bugs to <https://gitlab.xfce.org/apps/xfce4-terminal/-/issues>.\n",
        crate::reference::REFERENCE_VERSION,
        &crate::reference::baseline_commit()[..8]
    )
}

/// Returns the version of the native Xfce libraries used by this process.
pub fn native_xfce_version() -> String {
    crate::ffi::xfce::version()
}

fn push_color_rows(output: &mut String, bright: &str, start: u32) {
    use std::fmt::Write;

    for number in start..=37 {
        let foreground = match number {
            28 => 0,
            29 => 1,
            value => value,
        };
        write!(output, " {bright:>2}{foreground:>2}m |").unwrap();
        write!(
            output,
            "\u{1b}[{bright}{foreground}m {bright:>2}{foreground:>2}m "
        )
        .unwrap();
        for background in 40..=47 {
            write!(
                output,
                "\u{1b}[{bright}{foreground};{background}m {bright:>2}{foreground:>2}m "
            )
            .unwrap();
        }
        output.push_str("\u{1b}[0m\n");
    }
}

/// Returns the ANSI color table printed by `--color-table`.
pub fn color_table() -> String {
    use std::fmt::Write;

    let mut output = format!("{:>7}|{:>7}", "", "");
    for background in 40..=47 {
        write!(output, "   {background}m ").unwrap();
    }
    output.push('\n');
    push_color_rows(&mut output, "", 28);
    push_color_rows(&mut output, "1;", 30);
    output
}

fn c_strtol(value: &str) -> i32 {
    crate::ffi::libc::strtol_i32(value)
}

fn parse_command(value: &str) -> Result<Vec<OsString>, ParseError> {
    glib::shell_parse_argv(value).map_err(|error| ParseError::new(error.to_string()))
}

fn require_long_value(
    name: &str,
    inline: Option<&str>,
    args: &[&str],
    index: &mut usize,
    message: &str,
) -> Result<String, ParseError> {
    if let Some(value) = inline {
        return Ok(value.to_owned());
    }
    *index += 1;
    args.get(*index)
        .map(|value| (*value).to_owned())
        .ok_or_else(|| ParseError::new(format!("Option \"{name}\" requires {message}")))
}

fn require_short_value(
    option: &str,
    argument: &str,
    position: usize,
    args: &[&str],
    index: &mut usize,
    message: &str,
) -> Result<String, ParseError> {
    if position + 1 < argument.len() {
        return Ok(argument[position + 1..].to_owned());
    }
    *index += 1;
    args.get(*index)
        .map(|value| (*value).to_owned())
        .ok_or_else(|| ParseError::new(format!("Option \"{option}\" requires {message}")))
}

fn current_tab(windows: &mut [WindowSpec]) -> &mut TabSpec {
    windows
        .last_mut()
        .expect("the parser always has a window")
        .tabs
        .last_mut()
        .expect("every window has a tab")
}

fn set_visibility(window: &mut WindowSpec, name: &str, value: Visibility) -> bool {
    let target = match name {
        "menubar" => &mut window.menubar,
        "borders" => &mut window.borders,
        "toolbar" => &mut window.toolbar,
        "scrollbar" => &mut window.scrollbar,
        _ => return false,
    };
    *target = value;
    true
}

fn short_group_is_execute(argument: &[u8]) -> bool {
    argument.len() >= 2
        && argument.last() == Some(&b'x')
        && argument[1..argument.len() - 1]
            .iter()
            .all(|option| *option == b'H')
}

/// Parses window and tab launch specifications using the C application's rules.
pub fn parse_launch(
    args: &[&str],
    mut can_reuse_window: bool,
) -> Result<Vec<WindowSpec>, ParseError> {
    let mut windows = vec![WindowSpec::default()];
    let mut default_display: Option<OsString> = None;
    let mut default_directory: Option<OsString> = None;
    let mut ignore_window_option = true;
    let mut index = 0;

    while index < args.len() {
        let argument = args[index];
        if argument == "--" {
            break;
        }
        if !argument.starts_with('-') || argument == "-" {
            return Err(ParseError::unknown_argument(argument.as_bytes()));
        }

        if let Some(long) = argument.strip_prefix("--") {
            let (name, inline) = long
                .split_once('=')
                .map_or((long, None), |(name, value)| (name, Some(value)));

            if let Some(item) = name.strip_prefix("show-") {
                if inline.is_none()
                    && set_visibility(windows.last_mut().unwrap(), item, Visibility::Show)
                {
                    index += 1;
                    continue;
                }
            }
            if let Some(item) = name.strip_prefix("hide-") {
                if inline.is_none()
                    && set_visibility(windows.last_mut().unwrap(), item, Visibility::Hide)
                {
                    index += 1;
                    continue;
                }
            }

            match name {
                "default-display" => {
                    default_display = Some(
                        require_long_value(
                            "--default-display",
                            inline,
                            args,
                            &mut index,
                            "specifying the default X display as its parameter",
                        )?
                        .into(),
                    );
                }
                "default-working-directory" => {
                    default_directory = Some(
                        require_long_value(
                            "--default-working-directory",
                            inline,
                            args,
                            &mut index,
                            "specifying the default working directory as its parameter",
                        )?
                        .into(),
                    );
                }
                "execute" if inline.is_none() => {
                    index += 1;
                    if index >= args.len() {
                        return Err(ParseError::new(
                            "Option \"--execute/-x\" requires specifying the command to run on the rest of the command line separated from \"--execute/-x\"",
                        ));
                    }
                    current_tab(&mut windows).command =
                        Some(args[index..].iter().map(OsString::from).collect());
                    break;
                }
                "command" => {
                    let value = require_long_value(
                        "--command/-e",
                        inline,
                        args,
                        &mut index,
                        "specifying the command to run as its parameter",
                    )?;
                    current_tab(&mut windows).command = Some(parse_command(&value)?);
                }
                "working-directory" => {
                    current_tab(&mut windows).directory = Some(
                        require_long_value(
                            "--working-directory",
                            inline,
                            args,
                            &mut index,
                            "specifying the working directory as its parameter",
                        )?
                        .into(),
                    );
                }
                "title" => {
                    current_tab(&mut windows).title = Some(
                        require_long_value(
                            "--title/-T",
                            inline,
                            args,
                            &mut index,
                            "specifying the title as its parameter",
                        )?
                        .into(),
                    );
                }
                "dynamic-title-mode" => {
                    let value = require_long_value(
                        "--dynamic-title-mode",
                        inline,
                        args,
                        &mut index,
                        "specifying the dynamic title mode as its parameter",
                    )?;
                    current_tab(&mut windows).dynamic_title_mode =
                        match value.to_ascii_lowercase().as_str() {
                            "replace" => DynamicTitleMode::Replace,
                            "before" => DynamicTitleMode::Prepend,
                            "after" => DynamicTitleMode::Append,
                            "none" => DynamicTitleMode::Hide,
                            _ => {
                                return Err(ParseError::new(format!(
                                    "Invalid argument for option \"--dynamic-title-mode\": {value}"
                                )));
                            }
                        };
                }
                "initial-title" => {
                    current_tab(&mut windows).initial_title = Some(
                        require_long_value(
                            "--initial-title",
                            inline,
                            args,
                            &mut index,
                            "specifying the initial title as its parameter",
                        )?
                        .into(),
                    );
                }
                "hold" if inline.is_none() => current_tab(&mut windows).hold = true,
                "active-tab" if inline.is_none() => current_tab(&mut windows).active = true,
                "color-text" | "color-bg" => {
                    let option = format!("--{name}");
                    let value = require_long_value(
                        &option,
                        inline,
                        args,
                        &mut index,
                        "specifying the color as its parameter",
                    )?;
                    if gdk::RGBA::parse(&value).is_err() {
                        return Err(ParseError::new(format!("Unable to parse color: {value}")));
                    }
                    let tab = current_tab(&mut windows);
                    if name == "color-text" {
                        tab.color_text = Some(value.into());
                    } else {
                        tab.color_bg = Some(value.into());
                    }
                }
                "display" | "geometry" | "role" | "sm-client-id" | "startup-id" | "font" => {
                    let message = match name {
                        "display" => "specifying the X display as its parameter",
                        "geometry" => "specifying the window geometry as its parameter",
                        "role" => "specifying the window role as its parameter",
                        "sm-client-id" => "specifying the unique session id as its parameter",
                        "startup-id" => "specifying the startup id as its parameter",
                        "font" => "specifying the font name as its parameter",
                        _ => unreachable!(),
                    };
                    let option = format!("--{name}");
                    let value = require_long_value(&option, inline, args, &mut index, message)?;
                    let window = windows.last_mut().unwrap();
                    match name {
                        "display" => window.display = Some(value.into()),
                        "geometry" => window.geometry = Some(value.into()),
                        "role" => window.role = Some(value.into()),
                        "sm-client-id" => window.sm_client_id = Some(value.into()),
                        "startup-id" => window.startup_id = Some(value.into()),
                        "font" => window.font = Some(value.into()),
                        _ => unreachable!(),
                    }
                }
                "workspace" => {
                    let value = require_long_value(
                        "--workspace",
                        inline,
                        args,
                        &mut index,
                        "specifying the workspace number as its parameter",
                    )?;
                    windows.last_mut().unwrap().workspace = c_strtol(&value);
                }
                "icon" => {
                    windows.last_mut().unwrap().icon = Some(
                        require_long_value(
                            "--icon/-I",
                            inline,
                            args,
                            &mut index,
                            "specifying an icon name or filename as its parameter",
                        )?
                        .into(),
                    );
                }
                "drop-down" if inline.is_none() => windows.last_mut().unwrap().drop_down = true,
                "fullscreen" if inline.is_none() => windows.last_mut().unwrap().fullscreen = true,
                "maximize" if inline.is_none() => windows.last_mut().unwrap().maximize = true,
                "minimize" if inline.is_none() => windows.last_mut().unwrap().minimize = true,
                "tab" if inline.is_none() => {
                    if can_reuse_window {
                        windows.last_mut().unwrap().reuse_last_window = true;
                        can_reuse_window = false;
                    } else {
                        windows.last_mut().unwrap().tabs.push(TabSpec::default());
                    }
                }
                "window" if inline.is_none() => {
                    if can_reuse_window && ignore_window_option {
                        ignore_window_option = false;
                        can_reuse_window = false;
                    } else {
                        can_reuse_window = false;
                        windows.push(WindowSpec::default());
                    }
                }
                "zoom" => {
                    let value = require_long_value(
                        "--zoom",
                        inline,
                        args,
                        &mut index,
                        "specifying the zoom (-7 .. 7) as its parameter",
                    )?;
                    let zoom = c_strtol(&value);
                    if !(-7..=7).contains(&zoom) {
                        return Err(ParseError::new(
                            "Option \"--zoom\" requires specifying the zoom (-7 .. 7) as its parameter",
                        ));
                    }
                    windows.last_mut().unwrap().zoom = zoom;
                }
                "class" => {
                    require_long_value(
                        "--class",
                        inline,
                        args,
                        &mut index,
                        "specifying the class name as its parameter",
                    )?;
                }
                "disable-server" | "sync" | "g-fatal-warnings" if inline.is_none() => {}
                _ => return Err(ParseError::unknown_argument(argument.as_bytes())),
            }
        } else {
            let mut position = 1;
            while position < argument.len() {
                let option = argument.as_bytes()[position] as char;
                match option {
                    'H' => current_tab(&mut windows).hold = true,
                    'x' => {
                        if !short_group_is_execute(argument.as_bytes()) || index + 1 >= args.len() {
                            return Err(ParseError::new(
                                "Option \"--execute/-x\" requires specifying the command to run on the rest of the command line separated from \"--execute/-x\"",
                            ));
                        }
                        current_tab(&mut windows).command =
                            Some(args[index + 1..].iter().map(OsString::from).collect());
                        index = args.len();
                        break;
                    }
                    'e' | 'T' | 'I' => {
                        let (name, message) = match option {
                            'e' => (
                                "--command/-e",
                                "specifying the command to run as its parameter",
                            ),
                            'T' => ("--title/-T", "specifying the title as its parameter"),
                            'I' => (
                                "--icon/-I",
                                "specifying an icon name or filename as its parameter",
                            ),
                            _ => unreachable!(),
                        };
                        let value = require_short_value(
                            name, argument, position, args, &mut index, message,
                        )?;
                        match option {
                            'e' => current_tab(&mut windows).command = Some(parse_command(&value)?),
                            'T' => current_tab(&mut windows).title = Some(value.into()),
                            'I' => windows.last_mut().unwrap().icon = Some(value.into()),
                            _ => unreachable!(),
                        }
                        break;
                    }
                    _ => {
                        return Err(if position == 1 {
                            ParseError::unknown_argument(argument.as_bytes())
                        } else {
                            ParseError::unknown_short(argument.as_bytes(), position)
                        });
                    }
                }
                position += 1;
            }
        }
        index += 1;
    }

    for window in &mut windows {
        if window.display.is_none() {
            window.display.clone_from(&default_display);
        }
        for tab in &mut window.tabs {
            if tab.directory.is_none() {
                tab.directory.clone_from(&default_directory);
            }
        }
    }

    Ok(windows)
}

#[cfg(unix)]
const RAW_PREFIX: &[u8] = b"__XFCE_RAW_";

#[cfg(unix)]
fn encode_utf8_segment(bytes: &[u8], encoded: &mut String) {
    use std::fmt::Write;

    let mut offset = 0;
    while offset < bytes.len() {
        match std::str::from_utf8(&bytes[offset..]) {
            Ok(valid) => {
                encoded.push_str(valid);
                break;
            }
            Err(error) => {
                let valid_end = offset + error.valid_up_to();
                encoded.push_str(
                    std::str::from_utf8(&bytes[offset..valid_end])
                        .expect("valid_up_to identifies UTF-8"),
                );
                let invalid_len = error.error_len().unwrap_or_else(|| bytes.len() - valid_end);
                for byte in &bytes[valid_end..valid_end + invalid_len] {
                    write!(encoded, "__XFCE_RAW_B{byte:02X}_").unwrap();
                }
                offset = valid_end + invalid_len;
            }
        }
    }
}

#[cfg(unix)]
fn encode_argument(bytes: &[u8]) -> String {
    let mut encoded = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let next_prefix = bytes[offset..]
            .windows(RAW_PREFIX.len())
            .position(|window| window == RAW_PREFIX)
            .map(|position| offset + position)
            .unwrap_or(bytes.len());
        encode_utf8_segment(&bytes[offset..next_prefix], &mut encoded);
        if next_prefix == bytes.len() {
            break;
        }
        encoded.push_str("__XFCE_RAW_P_");
        offset = next_prefix + RAW_PREFIX.len();
    }
    encoded
}

#[cfg(unix)]
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(unix)]
fn decode_markers(bytes: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset..].starts_with(b"__XFCE_RAW_P_") {
            decoded.extend_from_slice(RAW_PREFIX);
            offset += b"__XFCE_RAW_P_".len();
            continue;
        }

        let marker_end = offset + b"__XFCE_RAW_B".len() + 3;
        if marker_end <= bytes.len()
            && bytes[offset..].starts_with(b"__XFCE_RAW_B")
            && bytes[marker_end - 1] == b'_'
        {
            let digits = (
                hex_value(bytes[offset + b"__XFCE_RAW_B".len()]),
                hex_value(bytes[offset + b"__XFCE_RAW_B".len() + 1]),
            );
            if let (Some(high), Some(low)) = digits {
                decoded.push((high << 4) | low);
                offset = marker_end;
                continue;
            }
        }
        decoded.push(bytes[offset]);
        offset += 1;
    }
    decoded
}

#[cfg(unix)]
fn decode_os_string(value: &mut OsString) {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    *value = OsString::from_vec(decode_markers(value.as_bytes()));
}

#[cfg(unix)]
fn decode_option(value: &mut Option<OsString>) {
    if let Some(value) = value {
        decode_os_string(value);
    }
}

#[cfg(unix)]
fn decode_windows(windows: &mut [WindowSpec]) {
    for window in windows {
        decode_option(&mut window.display);
        decode_option(&mut window.geometry);
        decode_option(&mut window.role);
        decode_option(&mut window.startup_id);
        decode_option(&mut window.sm_client_id);
        decode_option(&mut window.icon);
        decode_option(&mut window.font);
        for tab in &mut window.tabs {
            if let Some(command) = &mut tab.command {
                for argument in command {
                    decode_os_string(argument);
                }
            }
            decode_option(&mut tab.directory);
            decode_option(&mut tab.title);
            decode_option(&mut tab.initial_title);
            decode_option(&mut tab.color_text);
            decode_option(&mut tab.color_bg);
            decode_option(&mut tab.color_title);
        }
    }
}

#[cfg(unix)]
fn decode_error(mut error: ParseError) -> ParseError {
    if let Some((argument, position)) = error.unknown_short.take() {
        let argument = decode_markers(&argument);
        return ParseError::unknown_short(&argument, position);
    }
    error.message = decode_markers(&error.message);
    error
}

/// Parses process arguments without losing non-UTF-8 bytes on Unix.
#[cfg(unix)]
pub fn parse_launch_os(
    args: &[OsString],
    can_reuse_window: bool,
) -> Result<Vec<WindowSpec>, ParseError> {
    use std::os::unix::ffi::OsStrExt;

    let encoded = args
        .iter()
        .map(|argument| encode_argument(argument.as_bytes()))
        .collect::<Vec<_>>();
    let arguments = encoded.iter().map(String::as_str).collect::<Vec<_>>();
    let mut windows = parse_launch(&arguments, can_reuse_window).map_err(decode_error)?;
    decode_windows(&mut windows);

    Ok(windows)
}

/// Parses process arguments on platforms whose native strings are Unicode.
#[cfg(not(unix))]
pub fn parse_launch_os(
    args: &[OsString],
    can_reuse_window: bool,
) -> Result<Vec<WindowSpec>, ParseError> {
    let arguments = args
        .iter()
        .map(|argument| argument.to_string_lossy())
        .collect::<Vec<_>>();
    let arguments = arguments
        .iter()
        .map(|argument| argument.as_ref())
        .collect::<Vec<_>>();
    parse_launch(&arguments, can_reuse_window)
}
