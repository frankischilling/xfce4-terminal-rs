fn main() {
    use std::io::Write;
    use xfce4_terminal::cli::{ImmediateAction, parse_immediate};

    xfce4_terminal::localization::initialize().expect("initialize the fixed gettext configuration");

    let os_arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let display_arguments = os_arguments
        .iter()
        .map(|argument| argument.to_string_lossy())
        .collect::<Vec<_>>();
    let arguments = display_arguments
        .iter()
        .map(|argument| argument.as_ref())
        .collect::<Vec<_>>();
    let immediate = parse_immediate(&arguments);

    match immediate.action {
        Some(ImmediateAction::Help) => print!("{}", xfce4_terminal::cli::help_text()),
        Some(ImmediateAction::Version) => print!(
            "{}",
            xfce4_terminal::cli::version_text(&xfce4_terminal::cli::native_xfce_version())
        ),
        Some(ImmediateAction::ColorTable) => {
            print!("{}", xfce4_terminal::cli::color_table());
        }
        None => match xfce4_terminal::cli::parse_launch_os(&os_arguments, false) {
            Ok(_) => println!("{}", xfce4_terminal::candidate_status()),
            Err(error) => {
                let mut stderr = std::io::stderr().lock();
                stderr.write_all(b"xfce4-terminal: ").unwrap();
                stderr.write_all(error.as_bytes()).unwrap();
                stderr.write_all(b"\n").unwrap();
                std::process::exit(1);
            }
        },
    }
}
