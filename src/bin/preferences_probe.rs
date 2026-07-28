use std::process::Command;

use xfce4_terminal::preferences::{PreferenceKind, PreferenceValue, Preferences, definitions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let preferences = Preferences::new("xfce4-terminal")?;

    if std::env::var_os("XFCE4_TERMINAL_TEST_MIGRATION").is_some() {
        assert_eq!(preferences.migrate_legacy()?, 6);
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
                (1..=16)
                    .map(|index| format!("#{index:06x}"))
                    .collect::<Vec<_>>()
                    .join(";")
            ))
        );
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
    glib::MainContext::default().iteration(true);
    assert_eq!(
        preferences.get("title-initial")?,
        PreferenceValue::String(Some("changed outside Rust".to_owned()))
    );

    Ok(())
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
