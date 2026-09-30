//! Character styles (bold, italic, underline, strike) and font settings.

/// One of the character styles toggled from the Home ribbon or a keyboard shortcut.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextStyleKind {
    Bold,
    Italic,
    Underline,
    Strike,
}

use gpui::Context;

use crate::app::NotesApp;
use crate::models::ActiveField;

impl NotesApp {
    /// Clears every per-character style flag for the text that is currently being edited.
    pub(crate) fn reset_body_styles(&mut self) {
        self.edit_body_bold.clear();
        self.edit_body_italic.clear();
        self.edit_body_underline.clear();
        self.edit_body_strike.clear();
        self.touch_body_layout();
    }

    /// Loads bold, italic, underline, and strikethrough spans into the active editor buffers.
    pub(crate) fn load_body_styles(&mut self, styles: &crate::models::TextStyleSpans, len: usize) {
        self.edit_body_bold = crate::text::styles::spans_to_bool_vec(&styles.bold, len);
        self.edit_body_italic = crate::text::styles::spans_to_bool_vec(&styles.italic, len);
        self.edit_body_underline = crate::text::styles::spans_to_bool_vec(&styles.underline, len);
        self.edit_body_strike = crate::text::styles::spans_to_bool_vec(&styles.strike, len);
        self.touch_body_layout();
    }

    fn body_style_flags_mut(&mut self, kind: TextStyleKind) -> &mut Vec<bool> {
        match kind {
            TextStyleKind::Bold => &mut self.edit_body_bold,
            TextStyleKind::Italic => &mut self.edit_body_italic,
            TextStyleKind::Underline => &mut self.edit_body_underline,
            TextStyleKind::Strike => &mut self.edit_body_strike,
        }
    }

    fn body_style_flags(&self, kind: TextStyleKind) -> &Vec<bool> {
        match kind {
            TextStyleKind::Bold => &self.edit_body_bold,
            TextStyleKind::Italic => &self.edit_body_italic,
            TextStyleKind::Underline => &self.edit_body_underline,
            TextStyleKind::Strike => &self.edit_body_strike,
        }
    }

    /// True when the current selection (or the character before the caret) uses this style.
    pub(crate) fn text_style_is_on(&self, kind: TextStyleKind) -> bool {
        if !self.is_editing || self.active_field != ActiveField::Body {
            return false;
        }
        let flags = self.body_style_flags(kind);
        let char_count = self.edit_body.chars().count();
        if let Some((start, end)) = crate::text::selection::get_selection_range(
            self.edit_body_cursor,
            self.edit_body_anchor,
        ) {
            let start = start.min(flags.len()).min(char_count);
            let end = end.min(flags.len()).min(char_count);
            if start < end {
                return flags[start..end].iter().all(|flag| *flag);
            }
        }
        let pos = self.edit_body_cursor;
        pos > 0 && pos - 1 < flags.len() && flags[pos - 1]
    }

    /// Toggles bold formatting on the current selection or, if no selection exists, on the
    /// active character.
    pub(crate) fn toggle_bold(&mut self, cx: &mut Context<Self>) {
        self.toggle_text_style(TextStyleKind::Bold, cx);
    }

    /// Toggles one character style on the selection, or on the character at the caret.
    pub(crate) fn toggle_text_style(&mut self, kind: TextStyleKind, cx: &mut Context<Self>) {
        if !self.is_editing || self.active_field != ActiveField::Body {
            return;
        }

        let char_count = self.edit_body.chars().count();
        let cursor = self.edit_body_cursor;
        let anchor = self.edit_body_anchor;
        {
            let flags = self.body_style_flags_mut(kind);
            if flags.len() < char_count {
                flags.resize(char_count, false);
            }

            if let Some((start, end)) = crate::text::selection::get_selection_range(cursor, anchor) {
                let start = start.min(char_count);
                let end = end.min(char_count);
                if start < end {
                    let all_on = flags[start..end].iter().all(|flag| *flag);
                    let new_value = !all_on;
                    for flag in &mut flags[start..end] {
                        *flag = new_value;
                    }
                }
            } else if cursor < char_count && cursor < flags.len() {
                flags[cursor] = !flags[cursor];
            } else if cursor > 0 && cursor - 1 < char_count && cursor - 1 < flags.len() {
                flags[cursor - 1] = !flags[cursor - 1];
            }
        }

        self.touch_body_layout();
        self.sync_active_text_block();
        cx.notify();
    }

    /// Updates the font size used for the notebook name in the sidebar.
    #[allow(dead_code)]
    pub(crate) fn set_note_heading_font_size(&mut self, size: f32) {
        self.note_heading_font_size = size;
    }

    /// Updates the font size used for section tabs.
    #[allow(dead_code)]
    pub(crate) fn set_section_name_font_size(&mut self, size: f32) {
        self.section_name_font_size = size;
    }

    /// Updates the font size used for page headings.
    #[allow(dead_code)]
    pub(crate) fn set_page_heading_font_size(&mut self, size: f32) {
        self.page_heading_font_size = size;
    }

    /// Updates the font size used for canvas text blocks.
    #[allow(dead_code)]
    pub(crate) fn set_canvas_body_font_size(&mut self, size: f32) {
        self.canvas_body_font_size = size;
        self.touch_body_layout();
    }

    /// Updates the font size used for items in the page sidebar list.
    #[allow(dead_code)]
    pub(crate) fn set_page_list_font_size(&mut self, size: f32) {
        self.page_list_font_size = size;
    }

    /// Updates the global font family and updates associated metrics.
    #[allow(dead_code)]
    pub(crate) fn set_font_family(&mut self, family: impl Into<String>) {
        self.font_family = family.into();
        self.touch_body_layout();
    }

    /// Marks the body hit-test cache stale. The next pointer sample rebuilds it once.
    pub(crate) fn touch_body_layout(&mut self) {
        self.body_layout_stamp = self.body_layout_stamp.wrapping_add(1);
    }

    pub(crate) fn ensure_body_hit_cache(&mut self) {
        let font = self.font_type();
        let size = self.canvas_body_font_size;
        let stamp = self.body_layout_stamp;
        let fresh = self.body_hit_cache.as_ref().is_some_and(|cache| {
            cache.stamp == stamp && cache.font_size == size && cache.font_type == font
        });
        if !fresh {
            self.body_hit_cache = Some(crate::text::selection::build_text_hit_cache(
                &self.edit_body,
                Some(&self.edit_body_bold),
                size,
                font,
                stamp,
            ));
        }
    }

    /// Character index under a window point, using the cached line advances.
    pub(crate) fn body_index_at_mouse(
        &mut self,
        mouse: gpui::Point<gpui::Pixels>,
        item_x: f32,
        item_y: f32,
        header_x: f32,
        header_y: f32,
    ) -> usize {
        self.ensure_body_hit_cache();
        let (rel_x, rel_y) = crate::text::selection::canvas_text_rel(
            mouse,
            self.is_sidebar_open,
            self.pan_x,
            self.pan_y,
            item_x,
            item_y,
            self.canvas_top_y,
            header_x,
            header_y,
        );
        match self.body_hit_cache.as_ref() {
            Some(cache) => crate::text::selection::cache_index_at(cache, rel_x, rel_y),
            None => 0,
        }
    }

    /// Character index on a known line. `rel_x` is already local to the text block.
    pub(crate) fn body_index_on_line(&mut self, line_start: usize, rel_x: f32) -> usize {
        self.ensure_body_hit_cache();
        match self.body_hit_cache.as_ref() {
            Some(cache) => {
                crate::text::selection::cache_index_on_line(cache, line_start, rel_x)
            }
            None => 0,
        }
    }

    /// Viewer hit test. The cache is reused while the same segment and font stay put.
    pub(crate) fn viewer_index_at_mouse(
        &mut self,
        segment_id: &str,
        text: &str,
        bold_flags: &[bool],
        mouse: gpui::Point<gpui::Pixels>,
        item_x: f32,
        item_y: f32,
        header_x: f32,
        header_y: f32,
    ) -> usize {
        let font = self.font_type();
        let size = self.canvas_body_font_size;
        let fresh = self.viewer_hit_cache.as_ref().is_some_and(|cache| {
            self.viewer_hit_cache_id.as_deref() == Some(segment_id)
                && cache.byte_len == text.len()
                && cache.bold_len == bold_flags.len()
                && cache.font_size == size
                && cache.font_type == font
        });
        if !fresh {
            self.viewer_hit_cache = Some(crate::text::selection::build_text_hit_cache(
                text,
                Some(bold_flags),
                size,
                font,
                0,
            ));
            self.viewer_hit_cache_id = Some(segment_id.to_string());
        }
        let (rel_x, rel_y) = crate::text::selection::canvas_text_rel(
            mouse,
            self.is_sidebar_open,
            self.pan_x,
            self.pan_y,
            item_x,
            item_y,
            self.canvas_top_y,
            header_x,
            header_y,
        );
        match self.viewer_hit_cache.as_ref() {
            Some(cache) => crate::text::selection::cache_index_at(cache, rel_x, rel_y),
            None => 0,
        }
    }

    /// Viewer drag hit test using the cache built when the drag started. Does not copy the text.
    pub(crate) fn viewer_index_cached(
        &mut self,
        mouse: gpui::Point<gpui::Pixels>,
        item_x: f32,
        item_y: f32,
    ) -> usize {
        let (rel_x, rel_y) = crate::text::selection::canvas_text_rel(
            mouse,
            self.is_sidebar_open,
            self.pan_x,
            self.pan_y,
            item_x,
            item_y,
            self.canvas_top_y,
            5.0,
            5.0,
        );
        match self.viewer_hit_cache.as_ref() {
            Some(cache) => crate::text::selection::cache_index_at(cache, rel_x, rel_y),
            None => 0,
        }
    }

    /// Returns the active FontType inferred from the currently configured font family.
    #[inline]
    pub(crate) fn font_type(&self) -> crate::constants::typography::FontType {
        crate::constants::typography::FontType::from_family_name(&self.font_family)
    }

    /// Computes the visual line height for canvas text blocks at current standardized font size.
    #[inline]
    pub(crate) fn canvas_line_height(&self) -> f32 {
        crate::constants::typography::line_height_for_font_size(self.canvas_body_font_size)
    }

    /// Computes the cursor indicator height for canvas text blocks at current standardized font size.
    #[inline]
    pub(crate) fn canvas_cursor_height(&self) -> f32 {
        crate::constants::typography::cursor_height_for_font_size(self.canvas_body_font_size)
    }

    /// Computes the selection highlight height for canvas text blocks at current standardized font size.
    #[inline]
    pub(crate) fn canvas_selection_height(&self) -> f32 {
        crate::constants::typography::selection_height_for_font_size(self.canvas_body_font_size)
    }
}
