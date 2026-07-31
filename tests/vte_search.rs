//! Public VTE search state that must stay aligned with the frozen screen.

use std::process::Command;

mod support;

use support::TempDirectory;

#[test]
fn terminal_widget_preserves_the_screen_search_contract() {
    let root = TempDirectory::new("xfce4-terminal-vte-search");
    let output = Command::new("dbus-run-session")
        .args(["--", "xvfb-run", "--auto-servernum"])
        .env("HOME", root.path().join("home"))
        .env("XDG_CONFIG_HOME", root.path().join("config"))
        .env("XDG_CACHE_HOME", root.path().join("cache"))
        .env("LC_ALL", "C")
        .env("NO_AT_BRIDGE", "1")
        .arg(env!("CARGO_BIN_EXE_xfce4-terminal-vte-search-probe"))
        .output()
        .expect("run the VTE search probe under a private display");

    assert!(
        output.status.success(),
        "probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("probe writes UTF-8"),
        concat!(
            "initial\tfalse\tfalse\n",
            "configured\ttrue\ttrue\n",
            "moves\tcalled\n",
            "reset-keeps\ttrue\ttrue\n",
            "reset-clears\tfalse\ttrue\n",
            "explicit-clear\tfalse\tfalse\n",
        )
    );
}
