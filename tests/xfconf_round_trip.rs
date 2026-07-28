use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn typed_values_round_trip_through_an_isolated_xfconf_service() {
    run_isolated_probe(false);
}

#[test]
fn old_terminalrc_values_migrate_into_an_isolated_xfconf_service() {
    run_isolated_probe(true);
}

fn run_isolated_probe(with_migration: bool) {
    let root = std::env::temp_dir().join(format!(
        "xfce4-terminal-xfconf-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let home = root.join("home");
    let config = root.join("config");
    let cache = root.join("cache");
    fs::create_dir_all(&home).expect("create isolated home");
    fs::create_dir_all(&config).expect("create isolated config");
    fs::create_dir_all(&cache).expect("create isolated cache");
    if with_migration {
        let legacy_dir = config.join("Terminal");
        fs::create_dir_all(&legacy_dir).expect("create legacy config directory");
        let mut terminalrc = String::from(
            "[Configuration]\n\
             MiscBell=FALSE\n\
             ScrollingLines=1234\n\
             CellWidthScale=1.25\n\
             TitleInitial=Legacy title\n\
             ScrollingBar=TERMINAL_SCROLLBAR_LEFT\n",
        );
        for index in 1..=16 {
            terminalrc.push_str(&format!("ColorPalette{index}=#{index:06x}\n"));
        }
        fs::write(legacy_dir.join("terminalrc"), terminalrc).expect("write old terminalrc");
    }

    let mut command = Command::new("dbus-run-session");
    command
        .arg("--")
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &config)
        .env("XDG_CACHE_HOME", &cache)
        .arg(env!("CARGO_BIN_EXE_xfce4-terminal-preferences-probe"));
    if with_migration {
        command.env("XFCE4_TERMINAL_TEST_MIGRATION", "1");
    }
    let status = command
        .status()
        .expect("run preference probe on a private session bus");

    let _ = fs::remove_dir_all(&root);
    assert!(status.success());
}
