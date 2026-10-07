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

fn uniform_color(values: &[u32], start: usize, end: usize) -> Option<u32> {
    if start >= end {
        return None;
    }
    let end = end.min(values.len());
    if start >= end {
        return Some(0);
    }
    let first = values[start];
    if values[start..end].iter().all(|color| *color == first) {
        Some(first)
    } else {
        None
    }
}

impl NotesApp {
    /// Clears every per-character style flag for the text that is currently being edited.
    pub(crate) fn reset_body_styles(&mut self) {
        self.edit_body_bold.clear();
        self.edit_body_italic.clear();
        self.edit_body_underline.clear();
        self.edit_body_strike.clear();
        self.edit_body_font_family.clear();
        self.edit_body_font_size.clear();
        self.edit_body_font_color.clear();
        self.edit_body_bg_color.clear();
        self.edit_body_line_layouts.clear();
        self.line_layout_anchor.clear();
        self.touch_body_layout();
    }

    /// Loads bold, italic, underline, and strikethrough spans into the active editor buffers.
    pub(crate) fn load_body_styles(&mut self, styles: &crate::models::TextStyleSpans, len: usize) {
        self.edit_body_bold = crate::text::styles::spans_to_bool_vec(&styles.bold, len);
        self.edit_body_italic = crate::text::styles::spans_to_bool_vec(&styles.italic, len);
        self.edit_body_underline = crate::text::styles::spans_to_bool_vec(&styles.underline, len);
        self.edit_body_strike = crate::text::styles::spans_to_bool_vec(&styles.strike, len);
        let (families, sizes, colors, backgrounds) =
            crate::text::styles::font_runs_to_vecs(&styles.font_runs, len);
        self.edit_body_font_family = families;
        self.edit_body_font_size = sizes;
        self.edit_body_font_color = colors;
        self.edit_body_bg_color = backgrounds;
        self.edit_body_line_layouts = styles.line_layouts.clone();
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

    /// Toggles one character style on the selection, or on the character at the caret.
    pub(crate) fn toggle_text_style(&mut self, kind: TextStyleKind, cx: &mut Context<Self>) {
        if !self.is_editing || self.active_field != ActiveField::Body {
            return;
        }
        self.record_edit(crate::app::history::EditKind::Format);

        let char_count = self.edit_body.chars().count();
        let cursor = self.edit_body_cursor;
        let anchor = self.edit_body_anchor;
        {
            let flags = self.body_style_flags_mut(kind);
            if flags.len() < char_count {
                flags.resize(char_count, false);
            }

            if let Some((start, end)) = crate::text::selection::get_selection_range(cursor, anchor)
            {
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
        self.schedule_autosave(cx);
        self.discard_unchanged_edit();
        cx.notify();
    }

    /// Text color shown on the ribbon: the pinned choice, or the color at the caret.
    pub(crate) fn shown_font_color(&self) -> u32 {
        self.shown_color(true)
    }

    /// Highlight shown on the ribbon: the pinned choice, or the highlight at the caret.
    pub(crate) fn shown_bg_color(&self) -> u32 {
        self.shown_color(false)
    }

    fn shown_color(&self, font: bool) -> u32 {
        let typing = if font {
            self.typing_font_color
        } else {
            self.typing_bg_color
        };
        let pinned = if font {
            self.font_color_pinned && self.font_color_pin_at == Some(self.edit_body_cursor)
        } else {
            self.bg_color_pinned && self.bg_color_pin_at == Some(self.edit_body_cursor)
        };
        if !self.is_editing || self.active_field != ActiveField::Body || pinned {
            return typing;
        }
        let values = if font {
            &self.edit_body_font_color
        } else {
            &self.edit_body_bg_color
        };
        if let Some((start, end)) = crate::text::selection::get_selection_range(
            self.edit_body_cursor,
            self.edit_body_anchor,
        ) {
            if let Some(color) = uniform_color(values, start, end) {
                return color;
            }
        }
        if self.edit_body_cursor > 0 {
            return values
                .get(self.edit_body_cursor - 1)
                .copied()
                .unwrap_or(typing);
        }
        typing
    }

    /// Sets the text color used for new characters, and paints the current selection.
    pub(crate) fn set_font_color(&mut self, color: u32, cx: &mut Context<Self>) {
        self.typing_font_color = color;
        self.font_color_pinned = true;
        self.font_color_pin_at = Some(self.edit_body_cursor);
        self.paint_color(true, color, cx);
    }

    /// Sets the highlight used for new characters, and paints the current selection.
    pub(crate) fn set_bg_color(&mut self, color: u32, cx: &mut Context<Self>) {
        self.typing_bg_color = color;
        self.bg_color_pinned = true;
        self.bg_color_pin_at = Some(self.edit_body_cursor);
        self.paint_color(false, color, cx);
    }

    fn paint_color(&mut self, font: bool, color: u32, cx: &mut Context<Self>) {
        if self.is_editing && self.active_field == ActiveField::Body {
            let char_count = self.edit_body.chars().count();
            let cursor = self.edit_body_cursor;
            let anchor = self.edit_body_anchor;
            if let Some((start, end)) = crate::text::selection::get_selection_range(cursor, anchor)
            {
                let start = start.min(char_count);
                let end = end.min(char_count);
                if start < end {
                    self.record_edit(crate::app::history::EditKind::Format);
                    let values = if font {
                        &mut self.edit_body_font_color
                    } else {
                        &mut self.edit_body_bg_color
                    };
                    if values.len() < char_count {
                        values.resize(char_count, 0);
                    }
                    for slot in &mut values[start..end] {
                        *slot = color;
                    }
                    self.touch_body_layout();
                    self.sync_active_text_block();
                    self.schedule_autosave(cx);
                    self.discard_unchanged_edit();
                }
            }
        }
        cx.notify();
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
        let header_y = self.body_text_header_y().unwrap_or(header_y);
        let (rel_x, rel_y) = crate::text::selection::canvas_text_rel(
            mouse,
            self.layout_sidebar_w(),
            self.pan_x,
            self.pan_y,
            item_x,
            item_y,
            self.canvas_top_y,
            header_x,
            header_y,
            self.canvas_zoom,
        );
        let (line_idx, advance) = {
            let Some(cache) = self.body_hit_cache.as_ref() else {
                return 0;
            };
            let line_idx = crate::text::selection::cache_line_at_y(cache, rel_y);
            let advance = cache
                .lines
                .get(line_idx)
                .and_then(|line| line.prefix.last().copied())
                .unwrap_or(0.0);
            (line_idx, advance)
        };
        let origin = self.body_line_origin(line_idx, advance);
        match self.body_hit_cache.as_ref() {
            Some(cache) => crate::text::selection::cache_index_at(cache, rel_x - origin, rel_y),
            None => 0,
        }
    }

    /// Character index on a known line. `rel_x` is already local to the text block.
    pub(crate) fn body_index_on_line(&mut self, line_start: usize, rel_x: f32) -> usize {
        self.ensure_body_hit_cache();
        let found = self.body_hit_cache.as_ref().and_then(|cache| {
            cache
                .lines
                .iter()
                .enumerate()
                .find(|(_, line)| line.start == line_start)
                .map(|(line_idx, line)| (line_idx, line.prefix.last().copied().unwrap_or(0.0)))
        });
        let Some((line_idx, advance)) = found else {
            return 0;
        };
        let origin = self.body_line_origin(line_idx, advance);
        match self.body_hit_cache.as_ref() {
            Some(cache) => {
                crate::text::selection::cache_index_on_line(cache, line_start, rel_x - origin)
            }
            None => 0,
        }
    }

    /// Screen-pixel distance from a mixed box to the text segment being edited.
    ///
    /// Plain text boxes keep the caller's header offset. Text under an image starts lower.
    fn body_text_header_y(&self) -> Option<f32> {
        let id = self.active_text_block_id.as_deref()?;
        let block_index = self.active_block_index?;
        let mixed = self.edit_canvas_items.iter().find_map(|item| match item {
            crate::models::CanvasItem::Mixed(mixed) if mixed.id == id => Some(mixed),
            _ => None,
        })?;
        Some(crate::app::editing::mixed_text_line_screen_top(
            &mixed.blocks,
            block_index,
            self.canvas_line_height(),
            self.canvas_zoom,
            1.0,
        ))
    }

    /// Where the glyphs of one body line start, matching center and right alignment.
    pub(crate) fn body_line_origin(&self, line_idx: usize, text_advance: f32) -> f32 {
        let layout = self
            .edit_body_line_layouts
            .get(line_idx)
            .copied()
            .unwrap_or_default();
        crate::text::selection::aligned_line_origin(
            self.body_align_content_width(),
            layout,
            text_advance,
        )
    }

    /// Flex-row width in character-advance units for the text box being edited.
    pub(crate) fn body_align_content_width(&self) -> f32 {
        let zoom = self.canvas_zoom.max(0.25);
        let (base, item_x) = self.active_body_box();
        let fitted = if self.active_field == crate::models::ActiveField::Body {
            self.fitted_text_box_width(base, item_x)
        } else {
            base
        };
        ((fitted * zoom) - crate::constants::typography::TEXT_COLUMN_INSET).max(0.0) / zoom
    }

    /// Stored width and canvas x of the text box that owns the body editor.
    fn active_body_box(&self) -> (f32, f32) {
        let Some(id) = self.active_text_block_id.as_deref() else {
            return (250.0, 0.0);
        };
        for item in &self.edit_canvas_items {
            match item {
                crate::models::CanvasItem::Text(text) if text.id == id => {
                    return (text.width.unwrap_or(250.0), text.x);
                }
                crate::models::CanvasItem::Mixed(mixed) if mixed.id == id => {
                    return (mixed.width.unwrap_or(250.0), mixed.x);
                }
                _ => {}
            }
        }
        (250.0, 0.0)
    }

    /// Width the active text box grows to so the longest line stays on one row.
    pub(crate) fn fitted_text_box_width(&self, base_width: f32, item_x: f32) -> f32 {
        let line_text_w = crate::text::selection::max_text_advance(
            &self.edit_body,
            Some(&self.edit_body_bold),
            self.canvas_body_font_size,
            self.font_type(),
        );
        let needed_width = line_text_w + 24.0;
        let desired_w = needed_width.max(base_width).max(250.0);
        let sidebar_w = self.layout_sidebar_w();
        let page_sidebar_w = self.layout_page_sidebar_w();
        let canvas_visible_w = (self.window_w - sidebar_w - page_sidebar_w - 30.0).max(300.0);
        let canvas_max_right = canvas_visible_w - self.pan_x;
        let max_allowed_width = (canvas_max_right - item_x).max(150.0);
        desired_w.min(max_allowed_width)
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
            self.layout_sidebar_w(),
            self.pan_x,
            self.pan_y,
            item_x,
            item_y,
            self.canvas_top_y,
            header_x,
            header_y,
            self.canvas_zoom,
        );
        self.viewer_index_at_local(segment_id, rel_x, rel_y)
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
            self.layout_sidebar_w(),
            self.pan_x,
            self.pan_y,
            item_x,
            item_y,
            self.canvas_top_y,
            5.0,
            5.0,
            self.canvas_zoom,
        );
        let segment_id = self.viewer_hit_cache_id.clone().unwrap_or_default();
        self.viewer_index_at_local(&segment_id, rel_x, rel_y)
    }

    fn viewer_index_at_local(&self, segment_id: &str, rel_x: f32, rel_y: f32) -> usize {
        let (line_idx, advance) = {
            let Some(cache) = self.viewer_hit_cache.as_ref() else {
                return 0;
            };
            let line_idx = crate::text::selection::cache_line_at_y(cache, rel_y);
            let advance = cache
                .lines
                .get(line_idx)
                .and_then(|line| line.prefix.last().copied())
                .unwrap_or(0.0);
            (line_idx, advance)
        };
        let origin = self.viewer_line_origin(segment_id, line_idx, advance);
        match self.viewer_hit_cache.as_ref() {
            Some(cache) => crate::text::selection::cache_index_at(cache, rel_x - origin, rel_y),
            None => 0,
        }
    }

    /// Glyph start for one viewer line. `segment_id` is a text id or `{mixed}::{block}`.
    pub(crate) fn viewer_line_origin(
        &self,
        segment_id: &str,
        line_idx: usize,
        text_advance: f32,
    ) -> f32 {
        let Some((box_width, layouts)) = self.viewer_segment_align(segment_id) else {
            return 0.0;
        };
        let layout = layouts.get(line_idx).copied().unwrap_or_default();
        let zoom = self.canvas_zoom.max(0.25);
        // The viewer block width is already in screen pixels, with 5px of padding on each side.
        let content = (box_width - 10.0).max(0.0) / zoom;
        crate::text::selection::aligned_line_origin(content, layout, text_advance)
    }

    fn viewer_segment_align(
        &self,
        segment_id: &str,
    ) -> Option<(f32, Vec<crate::models::LineLayout>)> {
        let items = &self.page_items_cache.as_ref()?.1;
        if let Some((mixed_id, index)) = segment_id.rsplit_once("::") {
            if let Ok(index) = index.parse::<usize>() {
                for item in items {
                    if let crate::models::CanvasItem::Mixed(mixed) = item {
                        if mixed.id == mixed_id {
                            if let Some(crate::models::ContentBlock::Text {
                                line_layouts, ..
                            }) = mixed.blocks.get(index)
                            {
                                return Some((mixed.width.unwrap_or(250.0), line_layouts.clone()));
                            }
                        }
                    }
                }
            }
        }
        for item in items {
            if let crate::models::CanvasItem::Text(text) = item {
                if text.id == segment_id {
                    return Some((text.width.unwrap_or(250.0), text.line_layouts.clone()));
                }
            }
        }
        None
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
