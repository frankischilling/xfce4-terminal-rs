//! Observable identity of the C application used for differential tests.

/// Returns the full commit ID of the frozen C reference.
pub fn baseline_commit() -> &'static str {
    include_str!("../tests/reference/BASELINE").trim()
}

/// Application version at the reference commit.
pub const REFERENCE_VERSION: &str = "1.2.0-dev";

/// Session bus service claimed by the terminal server.
pub const DBUS_SERVICE: &str = "org.xfce.Terminal5";

/// D-Bus interface exported by the terminal server.
pub const DBUS_INTERFACE: &str = "org.xfce.Terminal5";

/// Object path exported by the terminal server.
pub const DBUS_PATH: &str = "/org/xfce/Terminal";

/// Method used by later invocations to forward their arguments.
pub const DBUS_METHOD: &str = "Launch";
