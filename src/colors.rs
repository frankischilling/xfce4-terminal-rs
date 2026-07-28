//! Readers for installed and user-provided terminal color schemes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Source files installed by Meson as translated `.theme` files.
pub const BUILTIN_FILES: [&str; 8] = [
    "black-on-white.desktop.in",
    "dark-pastels.desktop.in",
    "green-on-black.desktop.in",
    "solarized-dark.desktop.in",
    "solarized-light.desktop.in",
    "tango.desktop.in",
    "white-on-black.desktop.in",
    "xterm.desktop.in",
];

/// One color scheme and all values from its `[Scheme]` group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorScheme {
    pub name: String,
    pub path: PathBuf,
    pub values: BTreeMap<String, String>,
}

/// Loads matching scheme files and sorts them by their translated name.
pub fn load_directory(directory: &Path, suffix: &str) -> Result<Vec<ColorScheme>, String> {
    let entries = std::fs::read_dir(directory).map_err(|error| error.to_string())?;
    let mut schemes = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_none_or(|name| !name.ends_with(suffix))
        {
            continue;
        }
        schemes.push(load_file(path)?);
    }
    schemes.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(schemes)
}

fn load_file(path: PathBuf) -> Result<ColorScheme, String> {
    let key_file = glib::KeyFile::new();
    key_file
        .load_from_file(&path, glib::KeyFileFlags::NONE)
        .map_err(|error| error.to_string())?;
    let name = key_file
        .string("Scheme", "Name")
        .map_err(|error| error.to_string())?
        .to_string();
    let mut values = BTreeMap::new();
    for key in key_file.keys("Scheme").map_err(|error| error.to_string())? {
        if key.as_str() == "Name" {
            continue;
        }
        let value = key_file
            .string("Scheme", key.as_str())
            .map_err(|error| error.to_string())?;
        values.insert(key.to_string(), value.to_string());
    }
    Ok(ColorScheme { name, path, values })
}
