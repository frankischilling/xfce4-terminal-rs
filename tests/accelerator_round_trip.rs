use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn gtk_accelerator_map_saves_and_loads_at_the_legacy_path() {
    let root = std::env::temp_dir().join(format!(
        "xfce4-terminal-accels-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("create accelerator test directory");
    let status = Command::new("xvfb-run")
        .args(["--auto-servernum"])
        .arg(env!("CARGO_BIN_EXE_xfce4-terminal-accelerator-probe"))
        .arg(&root)
        .status()
        .expect("run GTK accelerator probe");
    let accelerator_file = root.join("xfce4/terminal/accels.scm");
    assert!(status.success());
    assert!(accelerator_file.is_file());
    let _ = fs::remove_dir_all(root);
}
