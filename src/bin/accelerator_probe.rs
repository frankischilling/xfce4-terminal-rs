use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_home = std::env::args_os()
        .nth(1)
        .expect("usage: xfce4-terminal-accelerator-probe CONFIG_HOME");
    xfce4_terminal::localization::initialize()?;
    gtk::init()?;
    let accel_path = "<Actions>/terminal-window/new-tab";
    let default = gtk::accelerator_parse("<Primary>F11");
    let saved = gtk::accelerator_parse("<Primary>F12");
    let changed = gtk::accelerator_parse("<Primary>F10");
    xfce4_terminal::accelerators::add_entry(accel_path, default.0, default.1)?;
    assert!(xfce4_terminal::accelerators::change_entry(
        accel_path, saved.0, saved.1
    )?);
    xfce4_terminal::accelerators::save(Path::new(&config_home))?;
    assert!(xfce4_terminal::accelerators::change_entry(
        accel_path, changed.0, changed.1
    )?);
    xfce4_terminal::accelerators::load(Path::new(&config_home))?;
    assert_eq!(
        xfce4_terminal::accelerators::lookup_entry(accel_path)?,
        Some(saved)
    );
    assert!(xfce4_terminal::accelerators::path(Path::new(&config_home)).is_file());
    Ok(())
}
