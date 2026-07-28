//! Gettext configuration retained from the C application.

use crate::ffi;

pub const GETTEXT_DOMAIN: &str = "xfce4-terminal";
pub const CHARSET: &str = "UTF-8";
pub const DEFAULT_LOCALE_DIR: &str = "/usr/local/share/locale";

/// Returns the Meson-provided locale path or Cargo's default prefix.
pub fn locale_dir() -> &'static str {
    option_env!("XFCE4_TERMINAL_LOCALE_DIR").unwrap_or(DEFAULT_LOCALE_DIR)
}

/// Initializes the process gettext domain through libxfce4util.
pub fn initialize() -> Result<(), String> {
    ffi::xfce::textdomain(GETTEXT_DOMAIN, locale_dir(), CHARSET)
}
