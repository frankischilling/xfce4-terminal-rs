//! The child process a terminal screen starts.
//!
//! A screen decides three things before VTE forks: which file to execute, the
//! argument vector that file receives, and the environment it inherits. None of
//! those need a widget, so they live here and the widget layer passes what it
//! knows: the command the option parser produced for the tab, the preference
//! values, and the display of a realized toplevel.
//!
//! The reference copies the working directory into `PWD` when it is built
//! against VTE older than 0.51.90, because VTE resolved symbolic links itself
//! before that release. Neither the frozen build nor this port uses such a
//! version, so `PWD` is inherited like any other variable.

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;

use crate::ffi::{glib as glib_wrapper, libc};
use crate::preferences::{PreferenceError, PreferenceValue, Preferences};

/// Shells the reference tries when neither the environment nor the password
/// database names an executable one, in the order it tries them.
pub const FALLBACK_SHELLS: [&str; 13] = [
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
];

/// Milliseconds the reference lets a spawn take before it gives up.
pub const SPAWN_TIMEOUT_MS: i32 = 30_000;

/// Pseudo-terminal flags the reference asks VTE for.
pub const PTY_FLAGS: zoha_vte::PtyFlags = zoha_vte::PtyFlags::DEFAULT;

/// The name the terminal reports itself under to its children.
const COLORTERM_VALUE: &str = "xfce4-terminal";

/// Variables a child never inherits.
///
/// The terminal either sets these itself further down or considers them stale:
/// a size, a window, and a display belong to this terminal rather than to the
/// one the parent process ran in.
const WITHHELD_VARIABLES: [&str; 8] = [
    "COLUMNS",
    "LINES",
    "WINDOWID",
    "GNOME_DESKTOP_ICON",
    "COLORTERM",
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "TERM",
];

/// The preferences a child command depends on.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandPreferences {
    /// Whether a shell is started as a login shell.
    pub login_shell: bool,
    /// Whether the custom command replaces the login shell.
    pub run_custom_command: bool,
    /// The command line the custom command setting holds.
    pub custom_command: String,
}

impl CommandPreferences {
    /// Reads the preferences a child command depends on from a channel.
    ///
    /// A channel that holds no usable string for the custom command leaves the
    /// empty default, which the reference then refuses as an empty command.
    pub fn read(preferences: &Preferences) -> Result<Self, PreferenceError> {
        Ok(Self {
            login_shell: boolean(preferences, "command-login-shell")?,
            run_custom_command: boolean(preferences, "run-custom-command")?,
            custom_command: match preferences.get("custom-command")? {
                PreferenceValue::String(value) => value.unwrap_or_default(),
                other => return Err(unexpected("custom-command", &other)),
            },
        })
    }
}

/// The file a screen executes and the argument vector it passes.
#[derive(Clone, Debug, PartialEq)]
pub struct ChildCommand {
    /// The file to execute, which may be a bare name for a path search.
    pub command: OsString,
    /// The argument vector, whose first entry is the name the child sees.
    pub argv: Vec<OsString>,
}

impl ChildCommand {
    /// Returns the vector VTE receives, which repeats the file in front of it.
    pub fn spawn_argv(&self) -> Vec<OsString> {
        let mut arguments = Vec::with_capacity(self.argv.len() + 1);
        arguments.push(self.command.clone());
        arguments.extend(self.argv.iter().cloned());
        arguments
    }

    /// Returns the flags that vector needs.
    ///
    /// The file leads the vector, so VTE has to be told that the entry after it
    /// is argument zero rather than the first real argument.
    pub fn spawn_flags(&self) -> glib::SpawnFlags {
        glib::SpawnFlags::SEARCH_PATH | glib::SpawnFlags::FILE_AND_ARGV_ZERO
    }
}

/// A reason a screen cannot decide which child to start.
#[derive(Clone, Debug, PartialEq)]
pub enum ChildCommandError {
    /// The custom command preference holds no words.
    EmptyCustomCommand,
    /// GLib could not split the custom command preference.
    UnsplittableCustomCommand(String),
    /// Nothing on this host looked like an executable login shell.
    NoLoginShell,
}

impl ChildCommandError {
    /// Returns the message the reference shows in the C locale.
    pub fn message(&self) -> String {
        match self {
            Self::EmptyCustomCommand => {
                "Empty custom command in the terminal preferences".to_owned()
            }
            Self::UnsplittableCustomCommand(message) => message.clone(),
            Self::NoLoginShell => "Unable to determine your login shell.".to_owned(),
        }
    }
}

impl std::fmt::Display for ChildCommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message())
    }
}

impl std::error::Error for ChildCommandError {}

/// The display a realized toplevel lends to the child environment.
#[derive(Clone, Debug, PartialEq)]
pub enum Toplevel {
    /// No toplevel is realized, so the child learns of no display.
    Unrealized,
    /// An X11 toplevel names both its window and its display.
    X11 {
        /// The X window identifier of the toplevel.
        window: i64,
        /// The name of the display the toplevel is on.
        display: String,
    },
    /// A Wayland toplevel names only its display.
    Wayland {
        /// The name of the display the toplevel is on.
        display: String,
    },
}

/// Decides which child a screen starts.
///
/// A tab that was asked for a command runs it unchanged. Otherwise the
/// preferences decide between their own command line and a login shell, and
/// either way the first argument becomes the base name of the file, prefixed
/// with a dash for a login shell.
pub fn child_command(
    screen_command: Option<&[OsString]>,
    preferences: &CommandPreferences,
) -> Result<ChildCommand, ChildCommandError> {
    child_command_with(screen_command, preferences, login_shell)
}

/// Decides which child a screen starts, resolving the shell on demand.
///
/// `resolve_shell` runs at most once, and never when the tab or the preferences
/// already name a command, so a host without a usable shell can still start
/// one. `child_command` resolves it with [`login_shell`]; the parity tests
/// supply a fixed shell so that the decisions which do not depend on the host
/// can be compared anywhere.
pub fn child_command_with(
    screen_command: Option<&[OsString]>,
    preferences: &CommandPreferences,
    resolve_shell: impl FnOnce() -> Result<OsString, ChildCommandError>,
) -> Result<ChildCommand, ChildCommandError> {
    // A tab whose command is an empty vector counts as having none, which is
    // how the screen stores what the option parser gave it.
    if let Some([command, arguments @ ..]) = screen_command {
        return Ok(ChildCommand {
            command: command.clone(),
            argv: std::iter::once(command).chain(arguments).cloned().collect(),
        });
    }

    let (file, mut argv) = if preferences.run_custom_command {
        let mut argv = glib::shell_parse_argv(&preferences.custom_command)
            .map_err(unsplittable_custom_command)?;
        // A successful split always yields at least one word. That word names
        // the file to run, and the words after it stay as arguments.
        let file = argv.remove(0);
        (file, argv)
    } else {
        (resolve_shell()?, Vec::new())
    };

    let mut name = OsString::new();
    if preferences.login_shell {
        name.push("-");
    }
    name.push(glib_wrapper::path_basename(&file));
    argv.insert(0, name);

    Ok(ChildCommand {
        command: file,
        argv,
    })
}

/// Returns the login shell the reference starts on this host.
///
/// A shell named by the environment is trusted only while the process runs with
/// the privileges of the user who started it, so a set-user-id terminal cannot
/// be talked into running something else. The password database comes next, and
/// then the first executable entry of [`FALLBACK_SHELLS`].
pub fn login_shell() -> Result<OsString, ChildCommandError> {
    if libc::privileges_unchanged() {
        if let Some(shell) = glib::getenv("SHELL").filter(|shell| libc::is_executable(shell)) {
            return Ok(shell);
        }
    }

    if let Some(shell) = libc::password_database_shell().filter(|shell| libc::is_executable(shell))
    {
        return Ok(shell);
    }

    FALLBACK_SHELLS
        .iter()
        .map(OsStr::new)
        .find(|shell| libc::is_executable(shell))
        .map(OsStr::to_os_string)
        .ok_or(ChildCommandError::NoLoginShell)
}

/// Returns the environment a child inherits.
///
/// The process environment is passed on with the variables this terminal owns
/// removed, then the terminal names itself, and a realized toplevel adds the
/// display the child is shown on.
pub fn child_environment(toplevel: &Toplevel) -> Vec<OsString> {
    let mut environment: Vec<OsString> = glib::listenv()
        .into_iter()
        .filter(|name| !withheld(name))
        // A name that lost its value between the listing and the lookup is
        // dropped rather than passed on empty.
        .filter_map(|name| glib::getenv(&name).map(|value| entry(&name, value.as_os_str())))
        .collect();

    environment.push(entry(OsStr::new("COLORTERM"), OsStr::new(COLORTERM_VALUE)));

    match toplevel {
        Toplevel::Unrealized => {}
        Toplevel::X11 { window, display } => {
            environment.push(entry(
                OsStr::new("WINDOWID"),
                OsStr::new(&window.to_string()),
            ));
            environment.push(entry(OsStr::new("DISPLAY"), OsStr::new(display)));
        }
        Toplevel::Wayland { display } => {
            environment.push(entry(OsStr::new("WAYLAND_DISPLAY"), OsStr::new(display)));
        }
    }

    environment
}

fn withheld(name: &OsStr) -> bool {
    WITHHELD_VARIABLES
        .iter()
        .any(|withheld| name.as_bytes() == withheld.as_bytes())
}

fn entry(name: &OsStr, value: &OsStr) -> OsString {
    let mut entry = OsString::with_capacity(name.len() + value.len() + 1);
    entry.push(name);
    entry.push("=");
    entry.push(value);
    entry
}

fn unsplittable_custom_command(error: glib::Error) -> ChildCommandError {
    if error.matches(ShellError::EmptyString) {
        ChildCommandError::EmptyCustomCommand
    } else {
        ChildCommandError::UnsplittableCustomCommand(error.message().to_owned())
    }
}

fn boolean(preferences: &Preferences, name: &str) -> Result<bool, PreferenceError> {
    match preferences.get(name)? {
        PreferenceValue::Boolean(value) => Ok(value),
        other => Err(unexpected(name, &other)),
    }
}

fn unexpected(name: &str, value: &PreferenceValue) -> PreferenceError {
    PreferenceError::new(format!(
        "preference {name:?} does not hold the type of {value:?}"
    ))
}

/// The error codes `g_shell_parse_argv` reports.
///
/// The reference replaces the message of only one of them, so the port needs to
/// tell that one apart from the rest.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ShellError {
    BadQuoting,
    EmptyString,
    Failed,
}

impl glib::error::ErrorDomain for ShellError {
    fn domain() -> glib::Quark {
        // SAFETY: the quark function takes no arguments and interns a static
        // string, so the identifier it returns stays valid for the process.
        unsafe { glib::translate::from_glib(glib::ffi::g_shell_error_quark()) }
    }

    fn code(self) -> i32 {
        match self {
            Self::BadQuoting => glib::ffi::G_SHELL_ERROR_BAD_QUOTING,
            Self::EmptyString => glib::ffi::G_SHELL_ERROR_EMPTY_STRING,
            Self::Failed => glib::ffi::G_SHELL_ERROR_FAILED,
        }
    }

    fn from(code: i32) -> Option<Self> {
        match code {
            glib::ffi::G_SHELL_ERROR_BAD_QUOTING => Some(Self::BadQuoting),
            glib::ffi::G_SHELL_ERROR_EMPTY_STRING => Some(Self::EmptyString),
            _ => Some(Self::Failed),
        }
    }
}
