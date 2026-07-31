//! Runs the VTE search state through the screen-facing adapter boundary.

use std::process::ExitCode;

use xfce4_terminal::terminal::VteAdapter;

const PCRE2_MULTILINE: u32 = 0x0000_0400;
const PCRE2_UTF: u32 = 0x0008_0000;
const PCRE2_NO_UTF_CHECK: u32 = 0x4000_0000;
const SEARCH_FLAGS: u32 = PCRE2_UTF | PCRE2_NO_UTF_CHECK | PCRE2_MULTILINE;

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), String> {
    gtk::init().map_err(|error| format!("initialize GTK: {error}"))?;

    let terminal = VteAdapter::new(false)?;
    println!(
        "initial\t{}\t{}",
        terminal.has_search_regex()?,
        terminal.search_wraps()?
    );

    let regex = zoha_vte::Regex::for_search("needle", SEARCH_FLAGS)
        .map_err(|error| format!("create search regular expression: {error}"))?;
    terminal.set_search_regex(Some(&regex), true)?;
    println!(
        "configured\t{}\t{}",
        terminal.has_search_regex()?,
        terminal.search_wraps()?
    );

    terminal.find_next()?;
    terminal.find_previous()?;
    println!("moves\tcalled");

    terminal.reset(false)?;
    println!(
        "reset-keeps\t{}\t{}",
        terminal.has_search_regex()?,
        terminal.search_wraps()?
    );

    terminal.reset(true)?;
    println!(
        "reset-clears\t{}\t{}",
        terminal.has_search_regex()?,
        terminal.search_wraps()?
    );

    terminal.set_search_regex(None, false)?;
    println!(
        "explicit-clear\t{}\t{}",
        terminal.has_search_regex()?,
        terminal.search_wraps()?
    );

    Ok(())
}
