use std::fs;
use std::process::Command;

use xfce4_terminal::preferences::{PreferenceKind, PreferenceValue, definitions};

mod support;

use support::TempDirectory;

#[test]
fn typed_values_round_trip_through_an_isolated_xfconf_service() {
    run_isolated_probe(None, None);
}

#[test]
fn old_terminalrc_values_migrate_into_an_isolated_xfconf_service() {
    let mut terminalrc = String::from(
        "[Configuration]\n\
         MiscBell=FALSE\n\
         ScrollingLines=1234\n\
         CellWidthScale=1.25\n\
         TitleInitial=Legacy title\n\
         ScrollingBar=TERMINAL_SCROLLBAR_LEFT\n",
    );
    for index in 1..=15 {
        terminalrc.push_str(&format!("ColorPalette{index}=#{index:06x}\n"));
    }
    run_isolated_probe(Some(&terminalrc), Some("valid-migration"));
}

#[test]
fn every_terminalrc_mapping_migrates_into_an_isolated_xfconf_service() {
    let mut terminalrc = String::from("[Configuration]\n");
    for definition in definitions() {
        terminalrc.push_str(definition.legacy_key);
        terminalrc.push('=');
        terminalrc.push_str(&legacy_source(definition));
        terminalrc.push('\n');
    }
    let key_file = glib::KeyFile::new();
    key_file
        .load_from_data(&terminalrc, glib::KeyFileFlags::NONE)
        .expect("parse generated terminalrc");
    run_isolated_probe(Some(&terminalrc), Some("all-mappings"));
}

#[test]
fn invalid_terminalrc_values_do_not_stop_later_migration() {
    run_isolated_probe(
        Some(
            "[Configuration]\n\
         ScrollingLines=999999999\n\
         TitleInitial=After invalid value\n",
        ),
        Some("invalid-migration"),
    );
}

#[test]
fn unreadable_terminalrc_does_not_prevent_preferences_startup() {
    run_isolated_probe(Some("this is not a key file"), Some("unreadable-migration"));
}

#[test]
fn non_string_storage_uses_the_frozen_string_conversion() {
    run_isolated_probe(None, Some("uint-string-conversion"));
}

fn run_isolated_probe(terminalrc: Option<&str>, scenario: Option<&str>) {
    let root = TempDirectory::new("xfce4-terminal-xfconf");
    let home = root.path().join("home");
    let config = root.path().join("config");
    let cache = root.path().join("cache");
    fs::create_dir_all(&home).expect("create isolated home");
    fs::create_dir_all(&config).expect("create isolated config");
    fs::create_dir_all(&cache).expect("create isolated cache");
    if let Some(terminalrc) = terminalrc {
        fs::create_dir(config.join("Terminal")).expect("create legacy config directory");
        fs::write(config.join("Terminal/terminalrc"), terminalrc).expect("write old terminalrc");
    }

    let mut command = Command::new("dbus-run-session");
    command
        .arg("--")
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &config)
        .env("XDG_CACHE_HOME", &cache)
        .arg(env!("CARGO_BIN_EXE_xfce4-terminal-preferences-probe"));
    if let Some(scenario) = scenario {
        command.env("XFCE4_TERMINAL_TEST_SCENARIO", scenario);
    }
    let status = command
        .status()
        .expect("run preference probe on a private session bus");

    assert!(status.success());
}

fn legacy_source(definition: &xfce4_terminal::preferences::PreferenceDefinition) -> String {
    match &definition.kind {
        PreferenceKind::Boolean => match definition.default_value() {
            PreferenceValue::Boolean(true) => "FALSE".to_owned(),
            PreferenceValue::Boolean(false) => "TRUE".to_owned(),
            _ => unreachable!(),
        },
        PreferenceKind::String if definition.legacy_key == "Encoding" => "UTF-8".to_owned(),
        PreferenceKind::String => format!("legacy:{}", definition.name),
        PreferenceKind::Unsigned { minimum, maximum } => {
            let PreferenceValue::Unsigned(default) = definition.default_value() else {
                unreachable!()
            };
            if default == *minimum {
                maximum
            } else {
                minimum
            }
            .to_string()
        }
        PreferenceKind::Double { minimum, maximum } => {
            let PreferenceValue::Double(default) = definition.default_value() else {
                unreachable!()
            };
            if default == *minimum {
                maximum
            } else {
                minimum
            }
            .to_string()
        }
        PreferenceKind::Enumeration { values, .. } => {
            let PreferenceValue::Enumeration(default) = definition.default_value() else {
                unreachable!()
            };
            values
                .iter()
                .copied()
                .find(|value| *value != default)
                .unwrap_or(values[0])
                .to_owned()
        }
    }
}
