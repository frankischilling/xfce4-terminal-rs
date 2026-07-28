use xfce4_terminal::reference::{
    BASELINE_COMMIT, DBUS_INTERFACE, DBUS_METHOD, DBUS_PATH, DBUS_SERVICE, REFERENCE_VERSION,
};

#[test]
fn reference_identity_matches_the_frozen_c_release() {
    assert_eq!(BASELINE_COMMIT, "b5933b80d28ca35f873df8da2998e23be5f4e104");
    assert_eq!(REFERENCE_VERSION, "1.2.0-dev");
}

#[test]
fn dbus_identity_matches_the_existing_wire_contract() {
    assert_eq!(DBUS_SERVICE, "org.xfce.Terminal5");
    assert_eq!(DBUS_INTERFACE, "org.xfce.Terminal5");
    assert_eq!(DBUS_PATH, "/org/xfce/Terminal");
    assert_eq!(DBUS_METHOD, "Launch");
}
