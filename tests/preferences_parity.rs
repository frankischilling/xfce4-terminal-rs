use std::process::Command;

#[test]
fn definitions_match_the_frozen_c_preferences_class() {
    let Some(reference_probe) = std::env::var_os("XFCE4_TERMINAL_REFERENCE_PREFERENCES_PROBE")
    else {
        return;
    };

    let reference = Command::new(reference_probe)
        .output()
        .expect("run frozen C preference probe");
    assert!(reference.status.success());

    assert_eq!(
        xfce4_terminal::preferences::definition_contract().as_bytes(),
        reference.stdout
    );
}

#[test]
fn all_checked_definitions_are_available_through_the_public_model() {
    let definitions = xfce4_terminal::preferences::definitions();

    assert_eq!(definitions.len(), 94);
    assert_eq!(definitions[0].name, "background-mode");
    assert_eq!(definitions[93].name, "enable-sixel");
    assert_eq!(
        xfce4_terminal::preferences::definition("MiscTabPosition")
            .map(|definition| definition.name),
        None
    );
    assert_eq!(
        xfce4_terminal::preferences::definition("misc-tab-position")
            .map(|definition| definition.legacy_key),
        Some("MiscTabPosition")
    );
}
