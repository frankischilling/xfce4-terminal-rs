//! Rust implementation of Xfce Terminal.
//!
//! The installed application remains the C reference while the Rust port is
//! developed. Public modules expose only behavior that can be tested without
//! reaching into GTK widget internals.

mod ffi;

pub mod accelerators;
pub mod child;
pub mod cli;
pub mod colors;
pub mod links;
pub mod localization;
pub mod preferences;
pub mod reference;
pub mod screen;
pub mod terminal;

/// The GLib log domain the reference builds with, from `terminal/meson.build`.
pub const LOG_DOMAIN: &str = "xfce4-terminal";

/// Returns the status text used by the development-only Rust candidate.
pub fn candidate_status() -> String {
    format!(
        "xfce4-terminal Rust candidate for {} ({})",
        reference::REFERENCE_VERSION,
        &reference::baseline_commit()[..8]
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn candidate_status_names_the_frozen_reference() {
        assert_eq!(
            super::candidate_status(),
            "xfce4-terminal Rust candidate for 1.2.0-dev (b5933b80)"
        );
    }
}
