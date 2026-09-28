//! Centralized application constants and design tokens.
//!
//! All standardized typography, layout dimensions, and weights are declared here
//! so they can be changed in one place and automatically take effect across UI rendering,
//! cursor sizing, selection boxes, and mouse hit-testing.

#[allow(dead_code)]
pub(crate) mod typography {
    /// Default application font family name.
    pub(crate) const DEFAULT_FONT_FAMILY: &str = "Calibri";

    /// Font categories supported by the application text engine.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum FontType {
        SegoeUI,
        Arial,
        Monospace,
        Serif,
        Calibri,
        Inter,
    }

    impl FontType {
        /// Infers the font category from a font family name string.
        pub(crate) fn from_family_name(name: &str) -> Self {
            let lower = name.to_ascii_lowercase();
            if lower.contains("consolas")
                || lower.contains("mono")
                || lower.contains("courier")
                || lower.contains("cascadia")
                || lower.contains("code")
            {
                FontType::Monospace
            } else if lower.contains("arial") || lower.contains("helvetica") {
                FontType::Arial
            } else if lower.contains("times") || lower.contains("georgia") || lower.contains("serif") {
                FontType::Serif
            } else if lower.contains("calibri") {
                FontType::Calibri
            } else if lower.contains("inter") || lower.contains("roboto") {
                FontType::Inter
            } else {
                FontType::SegoeUI
            }
        }
    }

    /// Font size for notebook/note headings in the sidebar (11pt, 14.67px).
    pub(crate) const NOTE_HEADING_FONT_SIZE: f32 = 14.67;

    /// Font size for section tabs in the canvas header (10pt, 13.33px).
    pub(crate) const SECTION_NAME_FONT_SIZE: f32 = 13.33;

    /// Font size for the page title / heading above the canvas (20pt, 26.67px, OneNote page title).
    pub(crate) const PAGE_HEADING_FONT_SIZE: f32 = 26.67;

    /// Font size for canvas text blocks (11pt, 14.67px, OneNote body text).
    pub(crate) const CANVAS_BODY_FONT_SIZE: f32 = 14.67;

    /// Font size for page list items in the detail pane sidebar (10pt, 13.33px).
    pub(crate) const PAGE_LIST_FONT_SIZE: f32 = 13.33;

    /// Font size for toolbar buttons such as Bold, Save, Cancel, Edit Note and Delete Note.
    pub(crate) const BUTTON_FONT_SIZE: f32 = 13.0;

    /// Font size for hint lines shown inside the canvas.
    pub(crate) const HINT_FONT_SIZE: f32 = 12.0;

    /// Font size for small glyph buttons such as the "×" delete buttons.
    pub(crate) const SMALL_ICON_FONT_SIZE: f32 = 12.0;

    /// Font size for small metadata labels in the sidebar.
    pub(crate) const META_FONT_SIZE: f32 = 12.0;

    /// Font size for the empty-state message in the detail pane.
    pub(crate) const EMPTY_STATE_FONT_SIZE: f32 = 16.0;

    // Standard Font Weight Multipliers used for accurate hit-testing
    pub(crate) const WEIGHT_NORMAL: f32 = 1.00;
    pub(crate) const WEIGHT_SEMIBOLD: f32 = 1.05;
    pub(crate) const WEIGHT_BOLD: f32 = 1.10;

    /// Computes the visual line height for a given font size.
    #[inline]
    pub(crate) fn line_height_for_font_size(font_size: f32) -> f32 {
        font_size + 8.0
    }

    /// Computes the cursor indicator height for a given font size.
    #[inline]
    pub(crate) fn cursor_height_for_font_size(font_size: f32) -> f32 {
        font_size + 4.0
    }

    /// Computes the selection highlight height for a given font size.
    #[inline]
    pub(crate) fn selection_height_for_font_size(font_size: f32) -> f32 {
        font_size + 6.0
    }
}

#[allow(dead_code)]
pub(crate) mod colors {
    /// Base text color for regular content.
    pub(crate) const TEXT_PRIMARY: u32 = 0xe0e0e0;

    /// Dimmed text for placeholders, inactive tabs, and secondary labels.
    pub(crate) const TEXT_SECONDARY: u32 = 0x9e9e9e;

    /// Hint text shown inside the canvas and empty text blocks.
    pub(crate) const TEXT_HINT: u32 = 0x8a8a8a;
}

#[allow(dead_code)]
pub(crate) mod layout {
    /// Initial canvas top edge in window coordinates; replaced by the measured value after the first paint.
    pub(crate) const INITIAL_CANVAS_TOP_Y: f32 = 78.0;

    /// Left padding of the sidebar container.
    pub(crate) const SIDEBAR_PADDING_LEFT: f32 = 12.0;

    /// Left padding inside a note item in the sidebar.
    pub(crate) const NOTE_ITEM_PADDING_LEFT: f32 = 8.0;

    /// Left border width for selected note items.
    pub(crate) const NOTE_ITEM_BORDER_LEFT: f32 = 2.0;

    /// Combined X offset to the start of note title text inside the sidebar.
    pub(crate) const NOTE_ITEM_TEXT_OFFSET_X: f32 =
        SIDEBAR_PADDING_LEFT + NOTE_ITEM_BORDER_LEFT + NOTE_ITEM_PADDING_LEFT; // 22.0px

    /// Left padding before the page heading input above the canvas.
    pub(crate) const HEADING_PADDING_LEFT: f32 = 5.0;

    /// Left padding before the sections tab bar in the editor header.
    pub(crate) const SECTION_BAR_PADDING_LEFT: f32 = 10.0;
}
