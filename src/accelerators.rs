//! Persistence for the global GTK accelerator map.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::ffi;

/// Xfce resource path retained from the C application.
pub const RELATIVE_PATH: &str = "xfce4/terminal/accels.scm";
const DEFINITION_CONTRACT: &str = include_str!("../tests/reference/accelerator-contract.tsv");

/// One accelerator path and its frozen default shortcut.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcceleratorDefinition {
    pub path: &'static str,
    pub default_accelerator: &'static str,
}

/// Returns the checked text contract used by the frozen-reference comparison.
pub fn definition_contract() -> &'static str {
    DEFINITION_CONTRACT
}

/// Returns every window and terminal-widget accelerator in reference order.
pub fn definitions() -> &'static [AcceleratorDefinition] {
    static DEFINITIONS: OnceLock<Vec<AcceleratorDefinition>> = OnceLock::new();
    DEFINITIONS.get_or_init(|| {
        DEFINITION_CONTRACT
            .lines()
            .map(|line| {
                let (path, default_accelerator) =
                    line.split_once('\t').expect("accelerator contract fields");
                AcceleratorDefinition {
                    path,
                    default_accelerator,
                }
            })
            .collect()
    })
}

/// Registers the frozen default shortcut for every application action.
pub fn register_defaults() -> Result<(), String> {
    for definition in definitions() {
        let (key, modifiers) = gtk::accelerator_parse(definition.default_accelerator);
        add_entry(definition.path, key, modifiers)?;
    }
    Ok(())
}

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

/// Registers an action path and its default accelerator in GTK's global map.
pub fn add_entry(accel_path: &str, key: u32, modifiers: gdk::ModifierType) -> Result<(), String> {
    ffi::gtk::add_accel_entry(accel_path, key, modifiers)
}

/// Replaces an accelerator and resolves conflicting entries in GTK's map.
pub fn change_entry(
    accel_path: &str,
    key: u32,
    modifiers: gdk::ModifierType,
) -> Result<bool, String> {
    ffi::gtk::change_accel_entry(accel_path, key, modifiers)
}

/// Returns the key and modifiers stored for one GTK action path.
pub fn lookup_entry(accel_path: &str) -> Result<Option<(u32, gdk::ModifierType)>, String> {
    ffi::gtk::lookup_accel_entry(accel_path)
}
