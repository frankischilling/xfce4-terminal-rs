//! Observable identity of the C application used for differential tests.

/// Full commit ID of the frozen C reference.
pub const BASELINE_COMMIT: &str = "b5933b80d28ca35f873df8da2998e23be5f4e104";

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
