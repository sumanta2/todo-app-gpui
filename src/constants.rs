//! Centralized application constants and design tokens.
//!
//! All standardized typography, layout dimensions, and weights are declared here
//! so they can be changed in one place and automatically take effect across UI rendering,
//! cursor sizing, selection boxes, and mouse hit-testing.

#[allow(dead_code)]
pub(crate) mod typography {
    /// Font size for notebook/note headings in the sidebar (default: 14.0px).
    pub(crate) const NOTE_HEADING_FONT_SIZE: f32 = 14.0;

    /// Font size for section tabs in the canvas header (default: 11.0px).
    pub(crate) const SECTION_NAME_FONT_SIZE: f32 = 11.0;

    /// Font size for the page title / heading above the canvas (default: 20.0px).
    pub(crate) const PAGE_HEADING_FONT_SIZE: f32 = 20.0;

    /// Font size for canvas text blocks (default: 12.0px).
    pub(crate) const CANVAS_BODY_FONT_SIZE: f32 = 12.0;

    /// Font size for page list items in the detail pane sidebar (default: 11.0px).
    pub(crate) const PAGE_LIST_FONT_SIZE: f32 = 11.0;

    // Standard Font Weight Multipliers used for accurate hit-testing
    pub(crate) const WEIGHT_NORMAL: f32 = 1.00;
    pub(crate) const WEIGHT_SEMIBOLD: f32 = 1.05;
    pub(crate) const WEIGHT_BOLD: f32 = 1.10;
}

#[allow(dead_code)]
pub(crate) mod layout {
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
