use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use xfce4_terminal::{accelerators, colors, localization};

mod support;

use support::TempDirectory;

#[test]
fn candidate_exposes_the_application_resource_constants() {
    assert_eq!(
        accelerators::path(Path::new("/tmp/config")),
        PathBuf::from("/tmp/config/xfce4/terminal/accels.scm")
    );
    assert_eq!(accelerators::RELATIVE_PATH, "xfce4/terminal/accels.scm");
    assert_eq!(localization::GETTEXT_DOMAIN, "xfce4-terminal");
    assert_eq!(localization::CHARSET, "UTF-8");
}

#[test]
fn candidate_exposes_every_frozen_accelerator_definition() {
    assert_eq!(
        accelerators::definition_contract(),
        include_str!("reference/accelerator-contract.tsv")
    );
    assert_eq!(accelerators::definitions().len(), 65);
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
    let root = TempDirectory::new("xfce4-terminal-colors");
    let data = root.path().join("data/xfce4/terminal/colorschemes");
    let fallback_data = root
        .path()
        .join("fallback-data/xfce4/terminal/colorschemes");
    let config = root.path().join("config/xfce4/terminal/colorschemes");
    std::fs::create_dir_all(&data).expect("create data schemes");
    std::fs::create_dir_all(&fallback_data).expect("create fallback data schemes");
    std::fs::create_dir_all(&config).expect("create user schemes");
    std::fs::write(data.join("global.theme"), "[Scheme]\nName=Global\n")
        .expect("write global scheme");
    std::fs::write(
        fallback_data.join("global.theme"),
        "[Scheme]\nName=Shadowed global\n",
    )
    .expect("write shadowed global scheme");
    std::fs::write(
        fallback_data.join("fallback.theme"),
        "[Scheme]\nName=Fallback\n",
    )
    .expect("write fallback scheme");
    std::fs::create_dir(data.join("directory.theme")).expect("create directory match");
    std::fs::write(
        fallback_data.join("directory.theme"),
        "[Scheme]\nName=Regular fallback\n",
    )
    .expect("write regular fallback scheme");
    let symlink_source = root.path().join("symlink-source.theme");
    std::fs::write(&symlink_source, "[Scheme]\nName=Symlink primary\n")
        .expect("write symlink source");
    symlink(&symlink_source, data.join("symlink.theme")).expect("create scheme symlink");
    std::fs::write(
        fallback_data.join("symlink.theme"),
        "[Scheme]\nName=Shadowed symlink\n",
    )
    .expect("write shadowed symlink scheme");
    std::fs::write(config.join("user.theme"), "[Scheme]\nName=Custom\n")
        .expect("write user scheme");
    std::fs::write(
        config.join("extensionless"),
        "[Scheme]\nName=Any filename\n",
    )
    .expect("write extensionless user scheme");
    std::fs::write(
        data.join("missing-name.theme"),
        "[Scheme]\nColorForeground=#fff\n",
    )
    .expect("write titleless scheme");
    std::fs::write(config.join("invalid.theme"), "not a key file")
        .expect("write unreadable scheme");

    let schemes = colors::discover(
        &[root.path().join("data"), root.path().join("fallback-data")],
        &[root.path().join("config")],
    )
    .expect("discover schemes");

    assert_eq!(
        schemes
            .iter()
            .map(|scheme| scheme.name.as_str())
            .collect::<Vec<_>>(),
        [
            "Any filename",
            "Custom",
            "Fallback",
            "Global",
            "Regular fallback",
            "Symlink primary",
        ]
    );
}

#[test]
fn system_configuration_color_schemes_follow_user_and_system_precedence() {
    let root = TempDirectory::new("xfce4-terminal-config-colors");
    let user = root.path().join("user/xfce4/terminal/colorschemes");
    let first_system = root.path().join("system-first/xfce4/terminal/colorschemes");
    let later_system = root.path().join("system-later/xfce4/terminal/colorschemes");
    std::fs::create_dir_all(&user).expect("create user schemes");
    std::fs::create_dir_all(&first_system).expect("create first system schemes");
    std::fs::create_dir_all(&later_system).expect("create later system schemes");
    std::fs::write(user.join("shared.theme"), "[Scheme]\nName=User\n").expect("write user scheme");
    std::fs::write(
        first_system.join("shared.theme"),
        "[Scheme]\nName=Shadowed user\n",
    )
    .expect("write first shadowed user scheme");
    std::fs::write(
        later_system.join("shared.theme"),
        "[Scheme]\nName=Later shadowed user\n",
    )
    .expect("write later shadowed user scheme");
    std::fs::write(
        first_system.join("system-shared.theme"),
        "[Scheme]\nName=First system\n",
    )
    .expect("write first system scheme");
    std::fs::write(
        later_system.join("system-shared.theme"),
        "[Scheme]\nName=Shadowed later system\n",
    )
    .expect("write shadowed later system scheme");
    std::fs::write(
        later_system.join("system-later.theme"),
        "[Scheme]\nName=Later system only\n",
    )
    .expect("write later system scheme");

    let schemes = colors::discover(
        &[],
        &[
            root.path().join("user"),
            root.path().join("system-first"),
            root.path().join("system-later"),
        ],
    )
    .expect("discover configuration schemes");

    assert_eq!(
        schemes
            .iter()
            .map(|scheme| scheme.name.as_str())
            .collect::<Vec<_>>(),
        ["First system", "Later system only", "User"]
    );
}
