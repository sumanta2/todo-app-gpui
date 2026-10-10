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
    //! Component colors. The values in `metadata.json` replace these defaults at startup.
    //!
    //! Each getter reads the installed theme, so a restart after editing that file is enough.

    use std::sync::{OnceLock, RwLock};

    /// `#rrggbb` in the metadata file. A plain integer is accepted too.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) struct RgbColor(pub u32);

    impl serde::Serialize for RgbColor {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&format!("#{:06x}", self.0 & 0x00ff_ffff))
        }
    }

    impl<'de> serde::Deserialize<'de> for RgbColor {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct ColorVisitor;
            impl serde::de::Visitor<'_> for ColorVisitor {
                type Value = RgbColor;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a #rrggbb color or an integer")
                }

                fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<RgbColor, E> {
                    let hex = value.trim().trim_start_matches('#');
                    if hex.len() != 6 {
                        return Err(E::custom("color must be #rrggbb"));
                    }
                    u32::from_str_radix(hex, 16)
                        .map(RgbColor)
                        .map_err(|_| E::custom("color must be #rrggbb"))
                }

                fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<RgbColor, E> {
                    Ok(RgbColor(value as u32))
                }

                fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<RgbColor, E> {
                    Ok(RgbColor(value as u32))
                }
            }
            deserializer.deserialize_any(ColorVisitor)
        }
    }

    fn rgb(hex: u32) -> RgbColor {
        RgbColor(hex)
    }

    /// Every paintable color. Missing keys in an older metadata file keep these defaults.
    #[derive(Clone, Debug, PartialEq, serde::Serialize)]
    pub(crate) struct ThemeColors {
        pub(crate) note_page: RgbColor,
        pub(crate) note_ink: RgbColor,
        pub(crate) note_hint: RgbColor,
        pub(crate) note_selection: RgbColor,
        pub(crate) cursor: RgbColor,
        pub(crate) note_chrome_border: RgbColor,
        pub(crate) note_chrome_bar: RgbColor,
        pub(crate) onenote_bar: RgbColor,
        pub(crate) onenote_bar_line: RgbColor,
        pub(crate) onenote_bar_hover: RgbColor,
        pub(crate) onenote_bar_selected: RgbColor,
        /// Background of the title bar and the ribbon tab row.
        pub(crate) header_ribbon_bg: RgbColor,
        /// Hover background on the title bar and on a ribbon tab that is not selected.
        pub(crate) header_ribbon_bg_hover: RgbColor,
        /// Text and icons on the title bar and the ribbon tab row.
        pub(crate) header_ribbon_text: RgbColor,
        /// Disabled undo and redo icons on the title bar.
        pub(crate) header_ribbon_text_muted: RgbColor,
        pub(crate) onenote_ink: RgbColor,
        pub(crate) onenote_ink_muted: RgbColor,
        pub(crate) onenote_accent: RgbColor,
        pub(crate) onenote_tab_idle: RgbColor,
        pub(crate) onenote_tab_hover: RgbColor,
        pub(crate) onenote_page_list: RgbColor,
        pub(crate) onenote_page_selected: RgbColor,
        pub(crate) sidebar_bg: RgbColor,
        pub(crate) sidebar_create: RgbColor,
        pub(crate) sidebar_create_hover: RgbColor,
        pub(crate) sidebar_item: RgbColor,
        pub(crate) sidebar_item_text: RgbColor,
        pub(crate) sidebar_hover: RgbColor,
        pub(crate) sidebar_selected: RgbColor,
        pub(crate) sidebar_on_selected: RgbColor,
        pub(crate) sidebar_rule: RgbColor,
        pub(crate) font_color_options: Vec<RgbColor>,
        pub(crate) bg_color_options: Vec<RgbColor>,
    }

    impl Default for ThemeColors {
        fn default() -> Self {
            Self {
                note_page: rgb(0xfdfcf3),
                note_ink: rgb(0x6b5b3e),
                note_hint: rgb(0xa2996e),
                note_selection: rgb(0xa8a085),
                cursor: rgb(0x6b5b3e),
                note_chrome_border: rgb(0xded7c5),
                note_chrome_bar: rgb(0xd3cca9),
                onenote_bar: rgb(0xe8e3d7),
                onenote_bar_line: rgb(0xded7c5),
                onenote_bar_hover: rgb(0xddd6c6),
                onenote_bar_selected: rgb(0xd4cbb8),
                header_ribbon_bg: rgb(0x807159),
                header_ribbon_bg_hover: rgb(0x524938),
                header_ribbon_text: rgb(0xf7f3e8),
                header_ribbon_text_muted: rgb(0xb5a890),
                onenote_ink: rgb(0x6b5b3e),
                onenote_ink_muted: rgb(0xa2996e),
                onenote_accent: rgb(0xa2996e),
                onenote_tab_idle: rgb(0xded7c5),
                onenote_tab_hover: rgb(0xd3cca9),
                onenote_page_list: rgb(0xfdfcf3),
                onenote_page_selected: rgb(0xd3cca9),
                sidebar_bg: rgb(0xf2efe8),
                sidebar_create: rgb(0x4c592c),
                sidebar_create_hover: rgb(0x5d6d3a),
                sidebar_item: rgb(0xf2efe8),
                sidebar_item_text: rgb(0x1f1f1f),
                sidebar_hover: rgb(0xebe6d8),
                sidebar_selected: rgb(0xe1ddd1),
                sidebar_on_selected: rgb(0xffffff),
                sidebar_rule: rgb(0xc5c9c1),
                font_color_options: vec![rgb(0x6b5b3e), rgb(0x7ec8ff), rgb(0xc0392b)],
                bg_color_options: vec![rgb(0xa8a085), rgb(0xd3cca9), rgb(0xded7c5)],
            }
        }
    }

    #[derive(serde::Deserialize)]
    struct ThemeColorsFile {
        note_page: Option<RgbColor>,
        note_ink: Option<RgbColor>,
        note_hint: Option<RgbColor>,
        note_selection: Option<RgbColor>,
        cursor: Option<RgbColor>,
        note_chrome_border: Option<RgbColor>,
        note_chrome_bar: Option<RgbColor>,
        onenote_bar: Option<RgbColor>,
        onenote_bar_line: Option<RgbColor>,
        onenote_bar_hover: Option<RgbColor>,
        onenote_bar_selected: Option<RgbColor>,
        #[serde(alias = "chrome_band", alias = "header_bg")]
        header_ribbon_bg: Option<RgbColor>,
        #[serde(alias = "chrome_band_hover", alias = "header_bg_hover")]
        header_ribbon_bg_hover: Option<RgbColor>,
        #[serde(alias = "chrome_on_band", alias = "header_text")]
        header_ribbon_text: Option<RgbColor>,
        #[serde(alias = "chrome_on_band_muted", alias = "header_text_muted")]
        header_ribbon_text_muted: Option<RgbColor>,
        onenote_ink: Option<RgbColor>,
        onenote_ink_muted: Option<RgbColor>,
        onenote_accent: Option<RgbColor>,
        onenote_tab_idle: Option<RgbColor>,
        onenote_tab_hover: Option<RgbColor>,
        onenote_page_list: Option<RgbColor>,
        onenote_page_selected: Option<RgbColor>,
        sidebar_bg: Option<RgbColor>,
        sidebar_create: Option<RgbColor>,
        sidebar_create_hover: Option<RgbColor>,
        sidebar_item: Option<RgbColor>,
        sidebar_item_text: Option<RgbColor>,
        sidebar_hover: Option<RgbColor>,
        sidebar_selected: Option<RgbColor>,
        sidebar_on_selected: Option<RgbColor>,
        sidebar_rule: Option<RgbColor>,
        font_color_options: Option<Vec<RgbColor>>,
        bg_color_options: Option<Vec<RgbColor>>,
    }

    impl<'de> serde::Deserialize<'de> for ThemeColors {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let file = ThemeColorsFile::deserialize(deserializer)?;
            let base = ThemeColors::default();
            Ok(Self {
                note_page: file.note_page.unwrap_or(base.note_page),
                note_ink: file.note_ink.unwrap_or(base.note_ink),
                note_hint: file.note_hint.unwrap_or(base.note_hint),
                note_selection: file.note_selection.unwrap_or(base.note_selection),
                cursor: file.cursor.unwrap_or(base.cursor),
                note_chrome_border: file.note_chrome_border.unwrap_or(base.note_chrome_border),
                note_chrome_bar: file.note_chrome_bar.unwrap_or(base.note_chrome_bar),
                onenote_bar: file.onenote_bar.unwrap_or(base.onenote_bar),
                onenote_bar_line: file.onenote_bar_line.unwrap_or(base.onenote_bar_line),
                onenote_bar_hover: file.onenote_bar_hover.unwrap_or(base.onenote_bar_hover),
                onenote_bar_selected: file.onenote_bar_selected.unwrap_or(base.onenote_bar_selected),
                header_ribbon_bg: file.header_ribbon_bg.unwrap_or(base.header_ribbon_bg),
                header_ribbon_bg_hover: file.header_ribbon_bg_hover.unwrap_or(base.header_ribbon_bg_hover),
                header_ribbon_text: file.header_ribbon_text.unwrap_or(base.header_ribbon_text),
                header_ribbon_text_muted: file
                    .header_ribbon_text_muted
                    .unwrap_or(base.header_ribbon_text_muted),
                onenote_ink: file.onenote_ink.unwrap_or(base.onenote_ink),
                onenote_ink_muted: file.onenote_ink_muted.unwrap_or(base.onenote_ink_muted),
                onenote_accent: file.onenote_accent.unwrap_or(base.onenote_accent),
                onenote_tab_idle: file.onenote_tab_idle.unwrap_or(base.onenote_tab_idle),
                onenote_tab_hover: file.onenote_tab_hover.unwrap_or(base.onenote_tab_hover),
                onenote_page_list: file.onenote_page_list.unwrap_or(base.onenote_page_list),
                onenote_page_selected: file
                    .onenote_page_selected
                    .unwrap_or(base.onenote_page_selected),
                sidebar_bg: file.sidebar_bg.unwrap_or(base.sidebar_bg),
                sidebar_create: file.sidebar_create.unwrap_or(base.sidebar_create),
                sidebar_create_hover: file
                    .sidebar_create_hover
                    .unwrap_or(base.sidebar_create_hover),
                sidebar_item: file.sidebar_item.unwrap_or(base.sidebar_item),
                sidebar_item_text: file.sidebar_item_text.unwrap_or(base.sidebar_item_text),
                sidebar_hover: file.sidebar_hover.unwrap_or(base.sidebar_hover),
                sidebar_selected: file.sidebar_selected.unwrap_or(base.sidebar_selected),
                sidebar_on_selected: file.sidebar_on_selected.unwrap_or(base.sidebar_on_selected),
                sidebar_rule: file.sidebar_rule.unwrap_or(base.sidebar_rule),
                font_color_options: file.font_color_options.unwrap_or(base.font_color_options),
                bg_color_options: file.bg_color_options.unwrap_or(base.bg_color_options),
            })
        }
    }

    fn installed() -> &'static RwLock<ThemeColors> {
        static THEME: OnceLock<RwLock<ThemeColors>> = OnceLock::new();
        THEME.get_or_init(|| RwLock::new(ThemeColors::default()))
    }

    /// Replaces the colors used by the window. Called once when metadata is loaded.
    pub(crate) fn install(colors: ThemeColors) {
        if let Ok(mut theme) = installed().write() {
            *theme = colors;
        }
    }

    /// The colors last installed, or the built-in defaults.
    pub(crate) fn current() -> ThemeColors {
        installed()
            .read()
            .map(|theme| theme.clone())
            .unwrap_or_else(|_| ThemeColors::default())
    }

    fn pick(read: impl FnOnce(&ThemeColors) -> u32) -> u32 {
        match installed().read() {
            Ok(theme) => read(&theme),
            Err(_) => read(&ThemeColors::default()),
        }
    }

    pub(crate) fn note_page() -> u32 {
        pick(|theme| theme.note_page.0)
    }
    pub(crate) fn note_ink() -> u32 {
        pick(|theme| theme.note_ink.0)
    }
    pub(crate) fn note_hint() -> u32 {
        pick(|theme| theme.note_hint.0)
    }
    pub(crate) fn note_selection() -> u32 {
        pick(|theme| theme.note_selection.0)
    }
    pub(crate) fn cursor() -> u32 {
        pick(|theme| theme.cursor.0)
    }
    pub(crate) fn note_chrome_border() -> u32 {
        pick(|theme| theme.note_chrome_border.0)
    }
    pub(crate) fn note_chrome_bar() -> u32 {
        pick(|theme| theme.note_chrome_bar.0)
    }
    pub(crate) fn onenote_bar() -> u32 {
        pick(|theme| theme.onenote_bar.0)
    }
    pub(crate) fn onenote_bar_line() -> u32 {
        pick(|theme| theme.onenote_bar_line.0)
    }
    pub(crate) fn header_ribbon_bg() -> u32 {
        pick(|theme| theme.header_ribbon_bg.0)
    }
    pub(crate) fn header_ribbon_bg_hover() -> u32 {
        pick(|theme| theme.header_ribbon_bg_hover.0)
    }
    pub(crate) fn header_ribbon_text() -> u32 {
        pick(|theme| theme.header_ribbon_text.0)
    }
    pub(crate) fn header_ribbon_text_muted() -> u32 {
        pick(|theme| theme.header_ribbon_text_muted.0)
    }
    pub(crate) fn onenote_ink() -> u32 {
        pick(|theme| theme.onenote_ink.0)
    }
    pub(crate) fn onenote_ink_muted() -> u32 {
        pick(|theme| theme.onenote_ink_muted.0)
    }
    pub(crate) fn onenote_accent() -> u32 {
        pick(|theme| theme.onenote_accent.0)
    }
    pub(crate) fn onenote_tab_idle() -> u32 {
        pick(|theme| theme.onenote_tab_idle.0)
    }
    pub(crate) fn onenote_tab_hover() -> u32 {
        pick(|theme| theme.onenote_tab_hover.0)
    }
    pub(crate) fn onenote_page_list() -> u32 {
        pick(|theme| theme.onenote_page_list.0)
    }
    pub(crate) fn onenote_page_selected() -> u32 {
        pick(|theme| theme.onenote_page_selected.0)
    }
    pub(crate) fn sidebar_bg() -> u32 {
        pick(|theme| theme.sidebar_bg.0)
    }
    pub(crate) fn sidebar_create() -> u32 {
        pick(|theme| theme.sidebar_create.0)
    }
    pub(crate) fn sidebar_create_hover() -> u32 {
        pick(|theme| theme.sidebar_create_hover.0)
    }
    pub(crate) fn sidebar_item() -> u32 {
        pick(|theme| theme.sidebar_item.0)
    }
    pub(crate) fn sidebar_item_text() -> u32 {
        pick(|theme| theme.sidebar_item_text.0)
    }
    pub(crate) fn sidebar_hover() -> u32 {
        pick(|theme| theme.sidebar_hover.0)
    }
    pub(crate) fn sidebar_selected() -> u32 {
        pick(|theme| theme.sidebar_selected.0)
    }
    pub(crate) fn sidebar_on_selected() -> u32 {
        pick(|theme| theme.sidebar_on_selected.0)
    }
    pub(crate) fn sidebar_rule() -> u32 {
        pick(|theme| theme.sidebar_rule.0)
    }
    pub(crate) fn font_color_options() -> Vec<u32> {
        match installed().read() {
            Ok(theme) => theme.font_color_options.iter().map(|color| color.0).collect(),
            Err(_) => ThemeColors::default()
                .font_color_options
                .iter()
                .map(|color| color.0)
                .collect(),
        }
    }
    pub(crate) fn bg_color_options() -> Vec<u32> {
        match installed().read() {
            Ok(theme) => theme.bg_color_options.iter().map(|color| color.0).collect(),
            Err(_) => ThemeColors::default()
                .bg_color_options
                .iter()
                .map(|color| color.0)
                .collect(),
        }
    }
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
