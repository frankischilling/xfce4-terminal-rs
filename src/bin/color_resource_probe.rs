use std::path::PathBuf;

use xfce4_terminal::colors;

fn main() -> Result<(), String> {
    let mut data_directories = vec![glib::user_data_dir()];
    data_directories.extend(glib::system_data_dirs());
    let mut config_directories = vec![glib::user_config_dir()];
    config_directories.extend(glib::system_config_dirs());

    let mut paths = colors::discover(&data_directories, &config_directories)?
        .into_iter()
        .map(|scheme| {
            let resource_type = if config_directories
                .iter()
                .any(|directory| scheme.path.starts_with(directory))
            {
                "config"
            } else {
                "data"
            };
            (resource_type, scheme.path)
        })
        .collect::<Vec<(&str, PathBuf)>>();
    paths.sort();

    for (resource_type, path) in paths {
        println!("{resource_type}\t{}", path.display());
    }
    Ok(())
}
