use std::path::PathBuf;

use xfce4_terminal::colors;

fn main() -> Result<(), String> {
    let mut data_directories = vec![glib::user_data_dir()];
    data_directories.extend(glib::system_data_dirs());
    let mut config_directories = vec![glib::user_config_dir()];
    config_directories.extend(glib::system_config_dirs());

    let data_paths = colors::discover(&data_directories, &[])?
        .into_iter()
        .map(|scheme| ("data", scheme.path));
    let config_paths = colors::discover(&[], &config_directories)?
        .into_iter()
        .map(|scheme| ("config", scheme.path));
    let mut paths = data_paths
        .chain(config_paths)
        .collect::<Vec<(&str, PathBuf)>>();
    paths.sort();

    for (resource_type, path) in paths {
        println!("{resource_type}\t{}", path.display());
    }
    Ok(())
}
