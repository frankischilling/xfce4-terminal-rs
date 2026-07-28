use xfce4_terminal::reference::{
    DBUS_INTERFACE, DBUS_METHOD, DBUS_PATH, DBUS_SERVICE, REFERENCE_VERSION, baseline_commit,
};

#[test]
fn reference_identity_is_loaded_from_the_baseline_file() {
    assert_eq!(baseline_commit().len(), 40);
    assert!(
        baseline_commit()
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    );

    let meson = include_str!("../meson.build");
    assert!(meson.contains(&format!("version : '{REFERENCE_VERSION}'")));
}

#[test]
fn dbus_identity_matches_the_c_reference_source() {
    let config =
        include_str!("../terminal/terminal-config.h.in").replace("@TERMINAL_VERSION_DBUS@", "5");

    for (name, value) in [
        ("TERMINAL_DBUS_SERVICE", DBUS_SERVICE),
        ("TERMINAL_DBUS_INTERFACE", DBUS_INTERFACE),
        ("TERMINAL_DBUS_PATH", DBUS_PATH),
        ("TERMINAL_DBUS_METHOD_LAUNCH", DBUS_METHOD),
    ] {
        assert!(
            config.contains(&format!("#define {name} \"{value}\"")),
            "{name} differs from the C reference"
        );
    }
}
