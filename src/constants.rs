//! Centralized application constants and design tokens.
//!
//! All standardized typography, layout dimensions, and weights are declared here
//! so they can be changed in one place and automatically take effect across UI rendering,
//! cursor sizing, selection boxes, and mouse hit-testing.

pub(crate) mod typography {
    /// Default application font family name.
    pub(crate) const DEFAULT_FONT_FAMILY: &str = "Calibri";

    /// Font families offered in the Home ribbon. Append more names here later.
    pub(crate) const FONT_STYLE_OPTIONS: &[&str] = &["Calibri"];

    /// Point sizes offered in the Home ribbon, paired with the pixel size used for layout.
    /// Append more pairs here later. 12pt is an integer pixel size, so Calibri stays sharp.
    pub(crate) const FONT_SIZE_OPTIONS: &[(f32, f32)] = &[(12.0, CANVAS_BODY_FONT_SIZE)];

    /// Family name for a stored font index. Unknown indexes fall back to the first option.
    pub(crate) fn font_style_name(index: u8) -> &'static str {
        FONT_STYLE_OPTIONS
            .get(index as usize)
            .copied()
            .unwrap_or(FONT_STYLE_OPTIONS[0])
    }

    /// Index of a family name inside `FONT_STYLE_OPTIONS`. Unknown names use the first option.
    pub(crate) fn font_style_index(name: &str) -> u8 {
        FONT_STYLE_OPTIONS
            .iter()
            .position(|option| *option == name)
            .unwrap_or(0) as u8
    }

    /// Ribbon label for a pixel size, such as `11` for the current body size.
    pub(crate) fn font_size_label(size_px: f32) -> String {
        let pt = FONT_SIZE_OPTIONS
            .iter()
            .find(|(_, px)| (*px - size_px).abs() < 0.05)
            .map(|(pt, _)| *pt)
            .unwrap_or(12.0);
        if (pt - pt.round()).abs() < 0.05 {
            format!("{}", pt.round() as i32)
        } else {
            format!("{pt:.1}")
        }
    }

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
            } else if lower.contains("times")
                || lower.contains("georgia")
                || lower.contains("serif")
            {
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

    /// Notebook title in the left sidebar. Lato, about 16px.
    pub(crate) const NOTE_HEADING_FONT_SIZE: f32 = 16.0;

    /// Font size for section tabs in the canvas header (10pt, 13.33px).
    pub(crate) const SECTION_NAME_FONT_SIZE: f32 = 13.33;

    /// Font size for the page title / heading above the canvas (20pt, 26.67px, OneNote page title).
    pub(crate) const PAGE_HEADING_FONT_SIZE: f32 = 26.67;

    /// Font size for canvas text blocks (12pt, 16px). An integer size keeps Calibri on the pixel grid.
    pub(crate) const CANVAS_BODY_FONT_SIZE: f32 = 16.0;

    /// Font size for page list items in the detail pane sidebar (10pt, 13.33px).
    pub(crate) const PAGE_LIST_FONT_SIZE: f32 = 13.33;

    /// Font size for toolbar buttons such as Delete Note.
    pub(crate) const BUTTON_FONT_SIZE: f32 = 13.0;

    /// Shared ribbon metrics from the layout sample: 14px text, 18px icons,
    /// 12px horizontal padding, 6px vertical padding, and 8px between items.
    pub(crate) const RIBBON_TEXT: f32 = 14.0;
    pub(crate) const RIBBON_ICON: f32 = 18.0;
    pub(crate) const RIBBON_PAD_H: f32 = 12.0;
    pub(crate) const RIBBON_PAD_V: f32 = 6.0;
    pub(crate) const RIBBON_GAP: f32 = 8.0;

    /// Width of the popup panel opened by a ribbon dropdown (font, size, colors, zoom).
    /// Height follows the content.
    pub(crate) const RIBBON_DROPDOWN_W: f32 = 80.0;

    /// Font size for small glyph buttons such as the "×" delete buttons.
    pub(crate) const SMALL_ICON_FONT_SIZE: f32 = 12.0;

    /// Font size for the empty-state message in the detail pane.
    pub(crate) const EMPTY_STATE_FONT_SIZE: f32 = 16.0;

    /// Horizontal shift of one indent level inside a text box, in pixels.
    pub(crate) const INDENT_STEP: f32 = 24.0;

    /// Screen-pixel inset of the editable text column inside a text box.
    /// The column is `box_width * zoom - TEXT_COLUMN_INSET` wide.
    pub(crate) const TEXT_COLUMN_INSET: f32 = 16.0;

    /// Deepest indent level the Home ribbon will apply.
    pub(crate) const MAX_INDENT_LEVEL: u8 = 16;

    // Standard Font Weight Multipliers used for accurate hit-testing
    pub(crate) const WEIGHT_NORMAL: f32 = 1.00;
    pub(crate) const WEIGHT_SEMIBOLD: f32 = 1.05;
    pub(crate) const WEIGHT_BOLD: f32 = 1.10;

    /// Width of the text caret, in pixels. One pixel sits on the character boundary.
    pub(crate) const CURSOR_WIDTH: f32 = 1.0;

    /// Line box for body text. The extra 8px is the leading GPUI adds around Calibri,
    /// so the caret steps down by the same amount as each painted row.
    #[inline]
    pub(crate) fn line_height_for_font_size(font_size: f32) -> f32 {
        font_size + 8.0
    }

    /// Caret height covers the glyph inside the line box. The caret stays 1px wide.
    #[inline]
    pub(crate) fn cursor_height_for_font_size(font_size: f32) -> f32 {
        line_height_for_font_size(font_size) - 4.0
    }

    /// Selection bar sits inside the line box with a small gap above and below.
    #[inline]
    pub(crate) fn selection_height_for_font_size(font_size: f32) -> f32 {
        line_height_for_font_size(font_size) - 2.0
    }
}

pub(crate) mod colors {
    /// Cream note page and brown ink from the Mineral & Ochre sample.
    pub(crate) const NOTE_PAGE: u32 = 0xfdfcf3;
    pub(crate) const NOTE_INK: u32 = 0x6b5b3e;
    pub(crate) const NOTE_HINT: u32 = 0xa2996e;
    /// Olive highlight behind selected note text.
    pub(crate) const NOTE_SELECTION: u32 = 0xa8a085;
    pub(crate) const NOTE_CHROME_BORDER: u32 = 0xded7c5;
    pub(crate) const NOTE_CHROME_BAR: u32 = 0xd3cca9;

    /// Ribbon, section tabs, and page list using the same sample.
    pub(crate) const ONENOTE_BAR: u32 = 0xe8e3d7;
    pub(crate) const ONENOTE_BAR_LINE: u32 = 0xded7c5;
    /// Slightly darker than the ribbon bar. Used when a ribbon tab is hovered.
    pub(crate) const ONENOTE_BAR_HOVER: u32 = 0xddd6c6;
    /// A little darker again. Used when a ribbon tab is open.
    pub(crate) const ONENOTE_BAR_SELECTED: u32 = 0xd4cbb8;
    pub(crate) const ONENOTE_INK: u32 = 0x6b5b3e;
    pub(crate) const ONENOTE_INK_MUTED: u32 = 0xa2996e;
    pub(crate) const ONENOTE_ACCENT: u32 = 0xa2996e;
    pub(crate) const ONENOTE_TAB_IDLE: u32 = 0xded7c5;
    pub(crate) const ONENOTE_TAB_HOVER: u32 = 0xd3cca9;
    pub(crate) const ONENOTE_PAGE_LIST: u32 = 0xfdfcf3;
    pub(crate) const ONENOTE_PAGE_SELECTED: u32 = 0xd3cca9;

    /// Left sidebar from the note-list style guide.
    pub(crate) const SIDEBAR_BG: u32 = 0xf2efe8;
    pub(crate) const SIDEBAR_CREATE: u32 = 0x4c592c;
    pub(crate) const SIDEBAR_CREATE_HOVER: u32 = 0x5d6d3a;
    pub(crate) const SIDEBAR_ITEM: u32 = 0xf2efe8;
    pub(crate) const SIDEBAR_ITEM_TEXT: u32 = 0x1f1f1f;
    pub(crate) const SIDEBAR_HOVER: u32 = 0xebe6d8;
    pub(crate) const SIDEBAR_SELECTED: u32 = 0xe1ddd1;
    pub(crate) const SIDEBAR_ON_SELECTED: u32 = 0xffffff;
    pub(crate) const SIDEBAR_RULE: u32 = 0xc5c9c1;

    /// Font colors offered in the Home ribbon. Append more values later.
    /// `0` means the default body text color.
    pub(crate) const FONT_COLOR_OPTIONS: &[u32] = &[NOTE_INK, 0x7ec8ff, 0xc0392b];

    /// Light highlight colors offered in the Home ribbon. Append more values later.
    pub(crate) const BG_COLOR_OPTIONS: &[u32] = &[0xa8a085, 0xd3cca9, 0xded7c5];
}

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

    /// Left inset of the page title drawn at the top of the canvas.
    pub(crate) const HEADING_PADDING_LEFT: f32 = 16.0;

    /// Top inset of the page title drawn at the top of the canvas.
    pub(crate) const HEADING_PADDING_TOP: f32 = 10.0;
}
