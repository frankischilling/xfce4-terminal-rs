//! The VTE-backed terminal widget used by future window controllers.
//!
//! This module owns the link patterns that VTE highlights and the clipboard
//! operation used by a link context menu. It keeps GTK and VTE work on the
//! initialized main thread, while pure link classification remains in
//! [`crate::links`].

use gtk::prelude::WidgetExt;
use zoha_vte::traits::TerminalExt;

use crate::links::{self, PATTERNS};
use crate::preferences::{PreferenceError, PreferenceValue, Preferences};

const PCRE2_CASELESS: u32 = 0x0000_0008;
const PCRE2_MULTILINE: u32 = 0x0000_0400;
const PCRE2_UTF: u32 = 0x0008_0000;
const PCRE2_NO_UTF_CHECK: u32 = 0x4000_0000;
const PCRE2_JIT_COMPLETE: u32 = 0x0000_0001;
const PCRE2_JIT_PARTIAL_SOFT: u32 = 0x0000_0002;
const MATCH_FLAGS: u32 = PCRE2_CASELESS | PCRE2_UTF | PCRE2_NO_UTF_CHECK | PCRE2_MULTILINE;

/// A safe facade around the VTE terminal widget.
///
/// [`Self::from_preferences`] starts the widget with the same URL-highlighting
/// preference as the frozen C application. [`Self::new`] accepts that state
/// explicitly for callers that have already read it. The struct is intentionally
/// not a window or tab model.
pub struct VteAdapter {
    terminal: zoha_vte::Terminal,
    link_tags: Vec<Option<i32>>,
}

impl VteAdapter {
    /// Creates a VTE terminal and applies the requested link-highlighting state.
    pub fn new(highlight_links: bool) -> Result<Self, String> {
        ensure_main_thread()?;
        let mut adapter = Self {
            terminal: zoha_vte::Terminal::new(),
            link_tags: vec![None; PATTERNS.len()],
        };
        adapter.set_link_highlighting(highlight_links)?;
        Ok(adapter)
    }

    /// Creates a VTE terminal from the stored URL-highlighting preference.
    ///
    /// The future screen controller is responsible for calling
    /// [`Self::sync_link_highlighting`] when Xfconf reports a changed
    /// preference. Keeping that subscription with the screen controller avoids
    /// making this widget facade own the application's preference lifetime.
    pub fn from_preferences(preferences: &Preferences) -> Result<Self, String> {
        Self::new(link_highlighting_enabled(preferences).map_err(|error| error.to_string())?)
    }

    /// Returns the VTE widget for insertion into a GTK container.
    pub fn widget(&self) -> &zoha_vte::Terminal {
        &self.terminal
    }

    /// Returns the number of frozen link patterns registered with VTE.
    pub fn highlighted_link_pattern_count(&self) -> usize {
        self.link_tags.iter().flatten().count()
    }

    /// Applies the current URL-highlighting value from the preference channel.
    pub fn sync_link_highlighting(&mut self, preferences: &Preferences) -> Result<(), String> {
        self.set_link_highlighting(
            link_highlighting_enabled(preferences).map_err(|error| error.to_string())?,
        )
    }

    /// Enables or disables the frozen set of URL-highlight patterns.
    ///
    /// Each enabled pattern gets the `hand2` cursor. VTE owns a match regex as
    /// soon as `match_add_regex` returns, so the local regex can be released at
    /// the end of each loop iteration.
    pub fn set_link_highlighting(&mut self, enabled: bool) -> Result<(), String> {
        ensure_main_thread()?;

        if !enabled {
            for tag in &mut self.link_tags {
                if let Some(tag) = tag.take() {
                    self.terminal.match_remove(tag);
                }
            }
            return Ok(());
        }

        for (index, (slot, pattern)) in self.link_tags.iter_mut().zip(PATTERNS).enumerate() {
            if slot.is_some() {
                continue;
            }

            let regex = match zoha_vte::Regex::for_match(pattern.pattern, MATCH_FLAGS) {
                Ok(regex) => regex,
                Err(error) => {
                    glib::g_critical!(
                        crate::LOG_DOMAIN,
                        "Failed to parse regular expression pattern {index}: {error}"
                    );
                    continue;
                }
            };
            if let Err(error) = regex.jit(PCRE2_JIT_COMPLETE) {
                glib::g_critical!(
                    crate::LOG_DOMAIN,
                    "Failed to JIT regular expression '{}': {error}",
                    pattern.pattern
                );
            } else if let Err(error) = regex.jit(PCRE2_JIT_PARTIAL_SOFT) {
                glib::g_critical!(
                    crate::LOG_DOMAIN,
                    "Failed to JIT regular expression '{}': {error}",
                    pattern.pattern
                );
            }

            let tag = self.terminal.match_add_regex(&regex, 0);
            self.terminal.match_set_cursor_name(tag, "hand2");
            *slot = Some(tag);
        }

        Ok(())
    }

    /// Writes a context-menu link to PRIMARY and then CLIPBOARD.
    ///
    /// The frozen terminal performs the writes in this order. The `mailto:`
    /// prefix is removed only when it is lowercase, matching the existing link
    /// helper and the C context-menu action.
    pub fn copy_link(&self, link: &str) -> Result<(), String> {
        ensure_main_thread()?;
        let display = self.terminal.display();
        let text = links::clipboard_text(link);

        gtk::Clipboard::for_display(&display, &gdk::SELECTION_PRIMARY).set_text(text);
        gtk::Clipboard::for_display(&display, &gdk::SELECTION_CLIPBOARD).set_text(text);
        Ok(())
    }
}

fn ensure_main_thread() -> Result<(), String> {
    if gtk::is_initialized_main_thread() {
        Ok(())
    } else {
        Err("VTE terminals require the initialized GTK main thread".to_owned())
    }
}

fn link_highlighting_enabled(preferences: &Preferences) -> Result<bool, PreferenceError> {
    match preferences.get("misc-highlight-urls")? {
        PreferenceValue::Boolean(enabled) => Ok(enabled),
        value => Err(PreferenceError::new(format!(
            "misc-highlight-urls is not a boolean: {value:?}"
        ))),
    }
}
