use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn output_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "xfce4-terminal-{name}-{}-{nonce}",
        std::process::id()
    ))
}

fn harness() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/reference/differential.sh")
}

#[test]
fn identical_processes_have_no_behavioral_difference() {
    let output = output_dir("matching");
    let status = Command::new(harness())
        .args(["/usr/bin/true", "/usr/bin/true"])
        .arg(&output)
        .status()
        .expect("run differential harness");

    assert!(status.success());
    assert_eq!(
        std::fs::read_to_string(output.join("reference.status")).unwrap(),
        "0\n"
    );
    assert_eq!(
        std::fs::read_to_string(output.join("candidate.status")).unwrap(),
        "0\n"
    );
    std::fs::remove_dir_all(output).unwrap();
}

#[test]
fn different_exit_statuses_are_reported() {
    let output = output_dir("different");
    let status = Command::new(harness())
        .args(["/usr/bin/true", "/usr/bin/false"])
        .arg(&output)
        .status()
        .expect("run differential harness");

    assert_eq!(status.code(), Some(1));
    assert!(output.join("status.diff").is_file());
    std::fs::remove_dir_all(output).unwrap();
}
