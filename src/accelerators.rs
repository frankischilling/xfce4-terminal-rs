//! Persistence for the global GTK accelerator map.

use std::path::{Path, PathBuf};

use crate::ffi;

/// Xfce resource path retained from the C application.
pub const RELATIVE_PATH: &str = "xfce4/terminal/accels.scm";

/// Resolves the accelerator file below an Xfce configuration directory.
pub fn path(config_home: &Path) -> PathBuf {
    config_home.join(RELATIVE_PATH)
}

/// Loads the accelerator map if it exists.
pub fn load(config_home: &Path) -> Result<(), String> {
    let path = path(config_home);
    if path.is_file() {
        ffi::gtk::load_accel_map(&path)?;
    }
    Ok(())
}

/// Creates the parent directory and saves GTK's accelerator map.
pub fn save(config_home: &Path) -> Result<(), String> {
    let path = path(config_home);
    let parent = path
        .parent()
        .ok_or_else(|| "accelerator path has no parent".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    ffi::gtk::save_accel_map(&path)
}
