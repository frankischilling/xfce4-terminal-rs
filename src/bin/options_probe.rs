fn main() {
    use std::io::Write;

    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let can_reuse = std::env::var_os("CAN_REUSE_WINDOW").is_some();

    match xfce4_terminal::cli::parse_launch_os(&arguments, can_reuse) {
        Ok(windows) => std::io::stdout()
            .lock()
            .write_all(&xfce4_terminal::cli::format_launch_specs_bytes(&windows))
            .unwrap(),
        Err(error) => {
            let mut stderr = std::io::stderr().lock();
            stderr.write_all(error.as_bytes()).unwrap();
            stderr.write_all(b"\n").unwrap();
            std::process::exit(1);
        }
    }
}
