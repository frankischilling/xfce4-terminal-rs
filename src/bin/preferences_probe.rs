use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use xfce4_terminal::preferences::{PreferenceKind, PreferenceValue, Preferences, definitions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let preferences = Preferences::new("xfce4-terminal")?;

    if std::env::args().nth(1).as_deref() == Some("--values") {
        for definition in definitions() {
            println!(
                "{}\t{}",
                definition.name,
                display_value(&preferences.get(definition.name)?)
            );
        }
        return Ok(());
    }

    if let Some(scenario) = std::env::var_os("XFCE4_TERMINAL_TEST_SCENARIO") {
        check_migration_scenario(&preferences, &scenario.to_string_lossy())?;
        return Ok(());
    }

    for definition in definitions() {
        assert_eq!(
            preferences.get(definition.name)?,
            definition.default_value()
        );
        let value = round_trip_value(definition);
        preferences.set(definition.name, value.clone())?;
        assert_eq!(preferences.get(definition.name)?, value);
    }

    let status = Command::new("xfconf-query")
        .args([
            "--channel",
            "xfce4-terminal",
            "--property",
            "/title-initial",
            "--set",
            "changed outside Rust",
        ])
        .status()?;
    assert!(status.success());
    let expected = PreferenceValue::String(Some("changed outside Rust".to_owned()));
    let context = glib::MainContext::default();
    let deadline = Instant::now() + Duration::from_secs(5);
    let observed = loop {
        let _ = context.iteration(false);

        let observed = preferences.get("title-initial")?;
        if observed == expected || Instant::now() >= deadline {
            break observed;
        }
        thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(
        observed, expected,
        "Xfconf did not deliver the external property change before the timeout"
    );

    Ok(())
}

fn check_migration_scenario(
    preferences: &Preferences,
    scenario: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match scenario {
        "valid-migration" => {
            assert_eq!(
                preferences.get("misc-bell")?,
                PreferenceValue::Boolean(false)
            );
            assert_eq!(
                preferences.get("scrolling-lines")?,
                PreferenceValue::Unsigned(1234)
            );
            assert_eq!(
                preferences.get("cell-width-scale")?,
                PreferenceValue::Double(1.25)
            );
            assert_eq!(
                preferences.get("title-initial")?,
                PreferenceValue::String(Some("Legacy title".to_owned()))
            );
            assert_eq!(
                preferences.get("scrolling-bar")?,
                PreferenceValue::Enumeration("TERMINAL_SCROLLBAR_LEFT".to_owned())
            );
            assert_eq!(
                preferences.get("color-palette")?,
                PreferenceValue::String(Some(
                    (1..=15)
                        .map(|index| format!("#{index:06x}"))
                        .collect::<Vec<_>>()
                        .join(";")
                        + ";"
                ))
            );
        }
        "invalid-migration" => {
            assert_eq!(
                preferences.get("scrolling-lines")?,
                PreferenceValue::Unsigned(1000)
            );
            assert_eq!(
                preferences.get("title-initial")?,
                PreferenceValue::String(Some("After invalid value".to_owned()))
            );
        }
        "unreadable-migration" => {
            assert_eq!(
                preferences.get("scrolling-lines")?,
                PreferenceValue::Unsigned(1000)
            );
            assert_eq!(
                preferences.get("title-initial")?,
                PreferenceValue::String(Some("Terminal".to_owned()))
            );
        }
        "uint-string-conversion" => {
            let status = Command::new("xfconf-query")
                .args([
                    "--channel",
                    "xfce4-terminal",
                    "--property",
                    "/title-initial",
                    "--create",
                    "--type",
                    "uint",
                    "--set",
                    "42",
                ])
                .status()?;
            assert!(status.success());

            let expected = PreferenceValue::String(Some("42".to_owned()));
            let context = glib::MainContext::default();
            let deadline = Instant::now() + Duration::from_secs(5);
            let observed = loop {
                let _ = context.iteration(false);
                let observed = preferences.get("title-initial")?;
                if observed == expected || Instant::now() >= deadline {
                    break observed;
                }
                thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(observed, expected);
        }
        _ => return Err(format!("unknown preference probe scenario {scenario:?}").into()),
    }
    Ok(())
}

fn display_value(value: &PreferenceValue) -> String {
    match value {
        PreferenceValue::Boolean(value) => value.to_string(),
        PreferenceValue::String(Some(value)) | PreferenceValue::Enumeration(value) => value.clone(),
        PreferenceValue::String(None) => "<null>".to_owned(),
        PreferenceValue::Unsigned(value) => value.to_string(),
        PreferenceValue::Double(value) => value.to_string(),
    }
}

fn round_trip_value(
    definition: &xfce4_terminal::preferences::PreferenceDefinition,
) -> PreferenceValue {
    match &definition.kind {
        PreferenceKind::Boolean => {
            let PreferenceValue::Boolean(default) = definition.default_value() else {
                unreachable!()
            };
            PreferenceValue::Boolean(!default)
        }
        PreferenceKind::String => {
            PreferenceValue::String(Some(format!("parity:{}", definition.name)))
        }
        PreferenceKind::Unsigned { minimum, maximum } => {
            let PreferenceValue::Unsigned(default) = definition.default_value() else {
                unreachable!()
            };
            PreferenceValue::Unsigned(if default == *minimum {
                *maximum
            } else {
                *minimum
            })
        }
        PreferenceKind::Double { minimum, maximum } => {
            PreferenceValue::Double((minimum + maximum) / 2.0)
        }
        PreferenceKind::Enumeration { values, .. } => {
            PreferenceValue::Enumeration(values.last().expect("enum value").to_string())
        }
    }
}
