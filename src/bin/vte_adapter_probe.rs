//! Runs the VTE adapter through the widget boundary used by its integration test.

use std::process::ExitCode;

use gtk::prelude::{ContainerExt, GtkWindowExt, WidgetExt};

use xfce4_terminal::preferences::{PreferenceValue, Preferences};
use xfce4_terminal::terminal::VteAdapter;

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), String> {
    gtk::init().map_err(|error| format!("initialize GTK: {error}"))?;

    let preferences = Preferences::new("xfce4-terminal").map_err(|error| error.to_string())?;
    preferences
        .set("misc-highlight-urls", PreferenceValue::Boolean(false))
        .map_err(|error| error.to_string())?;

    let mut terminal = VteAdapter::from_preferences(&preferences)?;
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(640, 320);
    window.add(terminal.widget());
    window.show_all();
    drain_main_context();

    println!(
        "initial-highlighted-patterns\t{}",
        terminal.highlighted_link_pattern_count()
    );

    preferences
        .set("misc-highlight-urls", PreferenceValue::Boolean(true))
        .map_err(|error| error.to_string())?;
    terminal.sync_link_highlighting(&preferences)?;
    println!(
        "enabled-patterns\t{}",
        terminal.highlighted_link_pattern_count()
    );

    terminal.copy_link("mailto:user@example.com")?;
    let display = terminal.widget().display();
    let primary = gtk::Clipboard::for_display(&display, &gdk::SELECTION_PRIMARY)
        .wait_for_text()
        .map(|text| text.to_string())
        .unwrap_or_else(|| "<none>".to_owned());
    let clipboard = gtk::Clipboard::for_display(&display, &gdk::SELECTION_CLIPBOARD)
        .wait_for_text()
        .map(|text| text.to_string())
        .unwrap_or_else(|| "<none>".to_owned());
    println!("primary\t{primary}");
    println!("clipboard\t{clipboard}");

    terminal.set_link_highlighting(false)?;
    println!(
        "highlight-disabled\t{}",
        terminal.highlighted_link_pattern_count()
    );

    Ok(())
}

fn drain_main_context() {
    while glib::MainContext::default().iteration(false) {}
}
