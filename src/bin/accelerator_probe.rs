use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_home = std::env::args_os()
        .nth(1)
        .expect("usage: xfce4-terminal-accelerator-probe CONFIG_HOME");
    gtk::init()?;
    xfce4_terminal::accelerators::save(Path::new(&config_home))?;
    xfce4_terminal::accelerators::load(Path::new(&config_home))?;
    assert!(xfce4_terminal::accelerators::path(Path::new(&config_home)).is_file());
    Ok(())
}
