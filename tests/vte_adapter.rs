//! Public VTE adapter behavior that must stay aligned with the frozen widget.

use std::process::Command;

mod support;

use support::TempDirectory;

#[test]
fn terminal_widget_highlights_links_and_copies_to_both_selections() {
    let root = TempDirectory::new("xfce4-terminal-vte-adapter");
    let output = Command::new("dbus-run-session")
        .args(["--", "xvfb-run", "--auto-servernum"])
        .env("HOME", root.path().join("home"))
        .env("XDG_CONFIG_HOME", root.path().join("config"))
        .env("XDG_CACHE_HOME", root.path().join("cache"))
        .env("LC_ALL", "C")
        .env("NO_AT_BRIDGE", "1")
        .arg(env!("CARGO_BIN_EXE_xfce4-terminal-vte-adapter-probe"))
        .output()
        .expect("run the VTE adapter probe under a private display");

    assert!(
        output.status.success(),
        "probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("probe writes UTF-8"),
        concat!(
            "initial-highlighted-patterns\t0\n",
            "enabled-patterns\t5\n",
            "primary\tuser@example.com\n",
            "clipboard\tuser@example.com\n",
            "highlight-disabled\t0\n",
        )
    );
}
