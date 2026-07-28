use std::path::{Path, PathBuf};

use xfce4_terminal::{accelerators, colors, localization};

#[test]
fn accelerator_and_gettext_paths_match_the_c_application() {
    assert_eq!(
        accelerators::path(Path::new("/tmp/config")),
        PathBuf::from("/tmp/config/xfce4/terminal/accels.scm")
    );
    assert_eq!(accelerators::RELATIVE_PATH, "xfce4/terminal/accels.scm");
    assert_eq!(localization::GETTEXT_DOMAIN, "xfce4-terminal");
    assert_eq!(localization::CHARSET, "UTF-8");

    let app = include_str!("../terminal/terminal-app.c");
    assert!(app.contains("#define ACCEL_MAP_PATH \"xfce4/terminal/accels.scm\""));
    let main = include_str!("../terminal/main.c");
    assert!(main.contains("xfce_textdomain (GETTEXT_PACKAGE, PACKAGE_LOCALE_DIR, \"UTF-8\")"));
}

#[test]
fn all_builtin_color_schemes_load_through_the_public_reader() {
    let schemes = colors::load_directory(Path::new("colorschemes"), "desktop.in")
        .expect("load source color schemes");
    let names = schemes
        .iter()
        .map(|scheme| scheme.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "Black on White",
            "Dark Pastels",
            "Green on Black",
            "Solarized (dark)",
            "Solarized (light)",
            "Tango",
            "White on Black",
            "XTerm",
        ]
    );
    assert_eq!(
        schemes[3].values.get("ColorPalette").map(String::as_str),
        Some(
            "#073642;#dc322f;#859900;#b58900;#268bd2;#d33682;#2aa198;\
             #eee8d5;#002b36;#cb4b16;#586e75;#657b83;#839496;#6c71c4;\
             #93a1a1;#fdf6e3"
        )
    );
}

#[test]
fn meson_installs_color_schemes_and_gettext_catalogs_at_the_reference_paths() {
    let colors_meson = include_str!("../colorschemes/meson.build");
    assert!(colors_meson.contains("'xfce4' / 'terminal' / 'colorschemes'"));
    assert_eq!(colors_meson.matches(".desktop.in'").count(), 8);

    let po_meson = include_str!("../po/meson.build");
    assert!(po_meson.contains("i18n.gettext(meson.project_name(), preset: 'glib')"));
    let potfiles = include_str!("../po/POTFILES");
    assert!(potfiles.contains("terminal/terminal-preferences.c"));
    for scheme in colors::BUILTIN_FILES {
        assert!(potfiles.contains(scheme));
    }
}
