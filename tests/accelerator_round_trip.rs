use std::process::Command;

mod support;

use support::TempDirectory;

#[test]
fn gtk_accelerator_map_saves_and_loads_at_the_legacy_path() {
    let root = TempDirectory::new("xfce4-terminal-accels");
    let status = Command::new("xvfb-run")
        .args(["--auto-servernum"])
        .arg(env!("CARGO_BIN_EXE_xfce4-terminal-accelerator-probe"))
        .arg(root.path())
        .status()
        .expect("run GTK accelerator probe");
    let accelerator_file = root.path().join("xfce4/terminal/accels.scm");
    assert!(status.success());
    assert!(accelerator_file.is_file());
}
