//! Layout and colors that should still be in effect the next time the app opens.
//!
//! Stored beside `notes.json` as `metadata.json`. Notes stay in their own file.
//! Edit the `colors` object (`#rrggbb`) and restart the app to restyle the window.

use std::fs;
use std::path::PathBuf;

use super::{NotesApp, RibbonPane};
use crate::constants::colors::{self, ThemeColors};

fn sidebar_starts_pinned() -> bool {
    true
}

/// Pin state for the left notebook list and the ribbon menu.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct AppMetadata {
    /// Left notebook list stays open when this is true.
    #[serde(default = "sidebar_starts_pinned")]
    pub(crate) sidebar_pinned: bool,
    /// The ribbon menu under the tab strip stays open when this is true.
    #[serde(default)]
    pub(crate) ribbon_pinned: bool,
    /// Which ribbon menu occupies the pin: Home or View.
    #[serde(default)]
    pub(crate) ribbon_pane: RibbonPane,
    /// Component colors. A missing key keeps the built-in color for that part.
    #[serde(default)]
    pub(crate) colors: ThemeColors,
}

impl Default for AppMetadata {
    fn default() -> Self {
        Self {
            sidebar_pinned: true,
            ribbon_pinned: false,
            ribbon_pane: RibbonPane::Home,
            colors: ThemeColors::default(),
        }
    }
}

impl AppMetadata {
    /// Reads the saved layout and colors. A missing or unreadable file uses the first-run defaults.
    ///
    /// The file is written back so newly added color keys show up in an older metadata file.
    pub(crate) fn load() -> Self {
        let path = metadata_path();
        let loaded = if let Ok(bytes) = fs::read(&path) {
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            Self::default()
        };
        loaded.save();
        loaded
    }

    fn save(&self) {
        let path = metadata_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_vec_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}

impl NotesApp {
    /// Writes the current left-bar and ribbon pin choices to `metadata.json`.
    pub(crate) fn save_layout(&self) {
        AppMetadata {
            sidebar_pinned: self.is_sidebar_open,
            ribbon_pinned: self.ribbon_pinned,
            ribbon_pane: self.ribbon_pane,
            colors: colors::current(),
        }
        .save();
    }
}

fn metadata_path() -> PathBuf {
    let mut path = NotesApp::get_storage_path();
    path.pop();
    path.push("metadata.json");
    path
}

#[cfg(test)]
mod tests {
    use super::{sidebar_starts_pinned, AppMetadata};
    use crate::app::RibbonPane;
    use crate::constants::colors::ThemeColors;

    #[test]
    fn a_saved_layout_roundtrips() {
        let saved = AppMetadata {
            sidebar_pinned: false,
            ribbon_pinned: true,
            ribbon_pane: RibbonPane::View,
            colors: ThemeColors::default(),
        };
        let raw = serde_json::to_string(&saved).expect("metadata serializes");
        let loaded: AppMetadata = serde_json::from_str(&raw).expect("metadata parses");
        assert!(!loaded.sidebar_pinned);
        assert!(loaded.ribbon_pinned);
        assert_eq!(loaded.ribbon_pane, RibbonPane::View);
    }

    #[test]
    fn an_empty_file_keeps_the_first_run_defaults() {
        let loaded: AppMetadata = serde_json::from_str("{}").expect("empty object parses");
        assert!(loaded.sidebar_pinned);
        assert!(!loaded.ribbon_pinned);
        assert_eq!(loaded.ribbon_pane, RibbonPane::Home);
        assert_eq!(loaded.colors, ThemeColors::default());
        assert!(sidebar_starts_pinned());
    }

    #[test]
    fn one_color_in_the_file_overrides_only_that_color() {
        let loaded: AppMetadata = serde_json::from_str(
            "{\"colors\":{\"cursor\":\"#ff0000\"}}",
        )
        .expect("metadata parses");
        assert_eq!(loaded.colors.cursor.0, 0xff0000);
        assert_eq!(loaded.colors.note_page, ThemeColors::default().note_page);
        assert!(loaded.sidebar_pinned);
    }
}
