use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

    let rust_main = include_str!("../src/main.rs");
    assert!(rust_main.contains("localization::initialize()"));
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
fn installed_and_user_color_schemes_share_one_sorted_model() {
    let root = std::env::temp_dir().join(format!(
        "xfce4-terminal-colors-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let data = root.join("data/xfce4/terminal/colorschemes");
    let config = root.join("config/xfce4/terminal/colorschemes");
    std::fs::create_dir_all(&data).expect("create data schemes");
    std::fs::create_dir_all(&config).expect("create user schemes");
    std::fs::write(data.join("global.theme"), "[Scheme]\nName=Global\n")
        .expect("write global scheme");
    std::fs::write(config.join("user.theme"), "[Scheme]\nName=Custom\n")
        .expect("write user scheme");

    let schemes =
        colors::discover(&[root.join("data")], &root.join("config")).expect("discover schemes");
    let _ = std::fs::remove_dir_all(&root);

    assert_eq!(
        schemes
            .iter()
            .map(|scheme| scheme.name.as_str())
            .collect::<Vec<_>>(),
        ["Custom", "Global"]
    );
}
