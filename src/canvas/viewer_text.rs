//! Read-only text on the viewer canvas: hit testing, selection, and styled runs.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton};

use crate::app::NotesApp;
use crate::constants::colors::note_ink;
use crate::helpers::hash_str;
use crate::text::selection::cache_index_on_line;

impl NotesApp {
    /// Renders one read-only text segment: either the whole body of a plain `CanvasItem::Text`,
    /// or a single `Text` block inside a `CanvasItem::Mixed` (identified by `seg_id`, which is
    /// `"{mixed_id}::{block_index}"` for those, so the text right after an image keeps its own
    /// click/selection/cursor behavior).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_viewer_text_segment(
        &mut self,
        seg_id: String,
        text_content: String,
        styles: crate::models::TextStyleSpans,
        item_x: f32,
        item_y: f32,
        item_w: f32,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_text_active = self.viewer_active_text_block_id.as_deref() == Some(seg_id.as_str());

        let total_chars = text_content.chars().count();
        let bold_flags = crate::text::styles::spans_to_bool_vec(&styles.bold, total_chars);
        let italic_flags = crate::text::styles::spans_to_bool_vec(&styles.italic, total_chars);
        let underline_flags =
            crate::text::styles::spans_to_bool_vec(&styles.underline, total_chars);
        let strike_flags = crate::text::styles::spans_to_bool_vec(&styles.strike, total_chars);
        let (font_families, font_sizes, font_colors, bg_colors) =
            crate::text::styles::font_runs_to_vecs(&styles.font_runs, total_chars);
        if is_text_active {
            self.ensure_viewer_hit_cache(
                window,
                &seg_id,
                &text_content,
                &bold_flags,
                &italic_flags,
                &font_families,
                &font_sizes,
            );
        }

        let logical_lines: Vec<&str> = text_content.split('\n').collect();
        let mut line_rows: Vec<AnyElement> = Vec::new();
        let mut global_offset = 0;

        for (line_idx, line) in logical_lines.iter().enumerate() {
            let line_len = line.chars().count();
            let line_global_start = global_offset;
            let line_global_end = global_offset + line_len;

            let line_bold_flags = if line_global_start < bold_flags.len() {
                let end = line_global_end.min(bold_flags.len());
                let mut f = bold_flags[line_global_start..end].to_vec();
                if f.len() < line_len {
                    f.resize(line_len, false);
                }
                f
            } else {
                vec![false; line_len]
            };

            let mut row = crate::canvas::text_editor::style_line_row(
                div()
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(self.canvas_line_height() * self.canvas_zoom))
                    .line_height(px(self.canvas_line_height() * self.canvas_zoom)),
                styles
                    .line_layouts
                    .get(line_idx)
                    .copied()
                    .unwrap_or_default(),
                self.canvas_zoom,
            );

            let line_italic = crate::text::styles::slice_flags(
                &italic_flags,
                line_global_start,
                line_global_end,
                line_len,
            );
            let line_underline = crate::text::styles::slice_flags(
                &underline_flags,
                line_global_start,
                line_global_end,
                line_len,
            );
            let line_strike = crate::text::styles::slice_flags(
                &strike_flags,
                line_global_start,
                line_global_end,
                line_len,
            );
            let line_font_families = if line_global_start < font_families.len() {
                let end = line_global_end.min(font_families.len());
                let mut values = font_families[line_global_start..end].to_vec();
                values.resize(line_len, 0);
                values
            } else {
                vec![0u8; line_len]
            };
            let line_font_sizes = if line_global_start < font_sizes.len() {
                let end = line_global_end.min(font_sizes.len());
                let mut values = font_sizes[line_global_start..end].to_vec();
                values.resize(line_len, self.canvas_body_font_size);
                values
            } else {
                vec![self.canvas_body_font_size; line_len]
            };
            let line_font_colors = crate::text::styles::slice_colors(
                &font_colors,
                line_global_start,
                line_global_end,
                line_len,
            );
            let line_bg_colors = crate::text::styles::slice_colors(
                &bg_colors,
                line_global_start,
                line_global_end,
                line_len,
            );
            let runs = crate::text::styles::split_text_into_full_runs(
                line,
                &line_bold_flags,
                &line_italic,
                &line_underline,
                &line_strike,
                &line_font_families,
                &line_font_sizes,
                &line_font_colors,
                &line_bg_colors,
            );
            row = row.child(if runs.is_empty() {
                div()
                    .text_color(rgb(note_ink()))
                    .child("\u{00A0}")
                    .into_any_element()
            } else {
                crate::canvas::text_editor::styled_line_element(&runs, note_ink(), self.canvas_zoom)
            });

            // Per-line mouse down handler for exact line targeting
            let seg_id_for_line = seg_id.clone();
            let text_for_line = text_content.clone();
            let bold_for_line = bold_flags.clone();
            let italic_for_line = italic_flags.clone();
            let families_for_line = font_families.clone();
            let sizes_for_line = font_sizes.clone();
            let line_layout = styles
                .line_layouts
                .get(line_idx)
                .copied()
                .unwrap_or_default();
            let line_box_w = item_w;

            let row = row
                .id(("viewer-line-row", hash_str(&seg_id).wrapping_add(line_idx)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        let same = this.viewer_active_text_block_id.as_deref()
                            == Some(seg_id_for_line.as_str());
                        this.viewer_active_text_block_id = Some(seg_id_for_line.clone());

                        let sidebar_w = this.layout_sidebar_w();
                        let zoom = this.canvas_zoom.max(0.25);
                        let rel_x = ((event.position.x.as_f32() - sidebar_w - this.pan_x - 5.0)
                            / zoom
                            - item_x)
                            .max(0.0);
                        this.ensure_viewer_hit_cache(
                            window,
                            &seg_id_for_line,
                            &text_for_line,
                            &bold_for_line,
                            &italic_for_line,
                            &families_for_line,
                            &sizes_for_line,
                        );
                        let advance = this
                            .viewer_hit_cache
                            .as_ref()
                            .and_then(|cache| {
                                cache.lines.iter().find(|line| line.start == line_global_start)
                            })
                            .and_then(|line| line.prefix.last().copied())
                            .unwrap_or(0.0);
                        let content = (line_box_w - 10.0).max(0.0) / zoom;
                        let origin = crate::text::selection::aligned_line_origin(
                            content,
                            line_layout,
                            advance,
                        );
                        let click_idx = this
                            .viewer_hit_cache
                            .as_ref()
                            .map(|cache| {
                                cache_index_on_line(cache, line_global_start, rel_x - origin)
                            })
                            .unwrap_or(0)
                            .min(total_chars);
                        this.viewer_drag_origin = Some((item_x, item_y));
                        this.viewer_text_cursor = click_idx;
                        this.viewer_text_anchor = Some(click_idx);
                        this.is_selecting_viewer_text = true;
                        this.cursor_visible = true;
                        if same {
                            this.refresh_viewer_selection(cx, false);
                        } else {
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }),
                );

            line_rows.push(row.into_any_element());
            global_offset += line_len + 1;
        }

        let seg_id_down = seg_id.clone();
        let text_for_down = text_content.clone();
        let bold_flags_down = bold_flags.clone();
        let italic_flags_down = italic_flags.clone();
        let families_down = font_families.clone();
        let sizes_down = font_sizes.clone();

        let mut block = div()
            .absolute()
            .left(px(self.place_x(item_x)))
            .top(px(self.place_y(item_y)))
            .w(px(item_w))
            .p(px(5.0));
        if is_text_active {
            block = block.child(self.viewer_selection_layer(cx));
        }
        block
            .text_size(px(self.scaled(self.canvas_body_font_size)))
            .text_color(rgb(note_ink()))
            .cursor_text()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    this.focus_handle.focus(window, cx);
                    this.is_panning = false;
                    this.pan_start_mouse = None;
                    this.pan_start_val = None;
                    let same =
                        this.viewer_active_text_block_id.as_deref() == Some(seg_id_down.as_str());
                    this.viewer_active_text_block_id = Some(seg_id_down.clone());
                    this.viewer_drag_origin = Some((item_x, item_y));

                    let click_idx = this.viewer_index_at_mouse(
                        window,
                        &seg_id_down,
                        &text_for_down,
                        &bold_flags_down,
                        &italic_flags_down,
                        &families_down,
                        &sizes_down,
                        event.position,
                        item_x,
                        item_y,
                        5.0,
                        5.0,
                    );
                    this.viewer_text_cursor = click_idx;
                    this.viewer_text_anchor = Some(click_idx);
                    this.is_selecting_viewer_text = true;
                    this.cursor_visible = true;
                    if same {
                        this.refresh_viewer_selection(cx, false);
                    } else {
                        cx.notify();
                    }
                    cx.stop_propagation();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if this.is_selecting_viewer_text {
                        this.end_viewer_pointer(cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .children(line_rows)
            .into_any_element()
    }
}
