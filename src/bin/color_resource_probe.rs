use std::path::PathBuf;

use xfce4_terminal::colors;

fn main() -> Result<(), String> {
    let config_home = glib::user_config_dir();
    let mut data_directories = vec![glib::user_data_dir()];
    data_directories.extend(glib::system_data_dirs());

    let config_scheme_dir = config_home.join("xfce4/terminal/colorschemes");
    let mut paths = colors::discover(&data_directories, &config_home)?
        .into_iter()
        .map(|scheme| {
            let resource_type = if scheme.path.starts_with(&config_scheme_dir) {
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
