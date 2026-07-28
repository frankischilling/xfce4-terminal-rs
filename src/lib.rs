//! Rust implementation of Xfce Terminal.
//!
//! The installed application remains the C reference while the Rust port is
//! developed. Public modules expose only behavior that can be tested without
//! reaching into GTK widget internals.

pub mod reference;

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
