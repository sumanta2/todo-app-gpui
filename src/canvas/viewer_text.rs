//! Read-only text on the viewer canvas: hit testing, selection, and styled runs.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, IntoElement, MouseButton};

use crate::app::NotesApp;
use crate::constants::colors::TEXT_PRIMARY;
use crate::helpers::hash_str;
use crate::models::{CanvasItem, ContentBlock};
use crate::text::selection::{
    calculate_canvas_text_offset_full, calculate_line_text_offset_with_bold_and_font,
};

/// Resolves the live text, bold spans, position, and wrap width for whichever viewer text
/// segment is active, whether it is a plain `Text` item or a `Text` block inside a `Mixed`
/// item (identified by the compound id `"{mixed_id}::{block_index}"`).
pub(crate) fn resolve_viewer_segment(
    items: &[CanvasItem],
    active_id: &str,
) -> Option<(String, Vec<(usize, usize)>, f32, f32, f32)> {
    if let Some((base, idx_str)) = active_id.rsplit_once("::") {
        if let Ok(idx) = idx_str.parse::<usize>() {
            for item in items {
                if let CanvasItem::Mixed(m) = item {
                    if m.id == base {
                        if let Some(ContentBlock::Text { text, bold_spans, .. }) = m.blocks.get(idx) {
                            return Some((
                                text.clone(),
                                bold_spans.clone(),
                                m.x,
                                m.y,
                                m.width.unwrap_or(250.0),
                            ));
                        }
                    }
                }
            }
        }
    }
    for item in items {
        if let CanvasItem::Text(t) = item {
            if t.id == active_id {
                return Some((
                    t.text.clone(),
                    t.bold_spans.clone(),
                    t.x,
                    t.y,
                    t.width.unwrap_or(250.0),
                ));
            }
        }
    }
    None
}

impl NotesApp {
    /// Renders one read-only text segment: either the whole body of a plain `CanvasItem::Text`,
    /// or a single `Text` block inside a `CanvasItem::Mixed` (identified by `seg_id`, which is
    /// `"{mixed_id}::{block_index}"` for those, so the text right after an image keeps its own
    /// click/selection/cursor behavior).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_viewer_text_segment(
        &self,
        seg_id: String,
        text_content: String,
        styles: crate::models::TextStyleSpans,
        item_x: f32,
        item_y: f32,
        item_w: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_text_active = self.viewer_active_text_block_id.as_deref() == Some(seg_id.as_str());

        let sel_start = if is_text_active {
            self.viewer_text_anchor
                .map(|a| a.min(self.viewer_text_cursor))
                .unwrap_or(self.viewer_text_cursor)
        } else {
            0
        };
        let sel_end = if is_text_active {
            self.viewer_text_anchor
                .map(|a| a.max(self.viewer_text_cursor))
                .unwrap_or(self.viewer_text_cursor)
        } else {
            0
        };
        let has_sel = is_text_active && sel_start < sel_end;
        let cursor_idx = self.viewer_text_cursor;
        let total_chars = text_content.chars().count();
        let bold_flags = crate::text::styles::spans_to_bool_vec(&styles.bold, total_chars);
        let italic_flags = crate::text::styles::spans_to_bool_vec(&styles.italic, total_chars);
        let underline_flags = crate::text::styles::spans_to_bool_vec(&styles.underline, total_chars);
        let strike_flags = crate::text::styles::spans_to_bool_vec(&styles.strike, total_chars);

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

            let mut row = div()
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .min_h(px(self.canvas_line_height()));

            let has_sel_overlap = if has_sel {
                let sel_overlap_start = sel_start.max(line_global_start);
                let sel_overlap_end = sel_end.min(line_global_end);
                sel_overlap_start < sel_overlap_end
                    || (line_len == 0
                        && sel_start <= line_global_start
                        && sel_end > line_global_start)
            } else {
                false
            };

            if has_sel_overlap {
                let sel_overlap_start = sel_start.max(line_global_start);
                let sel_overlap_end = sel_end.min(line_global_end);

                if sel_overlap_start < sel_overlap_end {
                    let line_chars: Vec<char> = line.chars().collect();
                    let loc_start = sel_overlap_start
                        .saturating_sub(line_global_start)
                        .min(line_chars.len());
                    let loc_end = sel_overlap_end
                        .saturating_sub(line_global_start)
                        .min(line_chars.len());

                    let before_prefix: String = line_chars[..loc_start].iter().collect();
                    let sel_content: String = line_chars[loc_start..loc_end].iter().collect();

                    let before_flags = &line_bold_flags[..loc_start.min(line_bold_flags.len())];
                    let before_runs =
                        crate::text::styles::split_text_into_styled_runs(&before_prefix, before_flags);
                    let mut before_ghosts = Vec::new();
                    for run in &before_runs {
                        let disp = run.text.replace(' ', "\u{00A0}");
                        let el = div()
                            .text_color(rgba(0x00000000))
                            .font_weight(if run.is_bold {
                                gpui::FontWeight::BOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .child(disp);
                        before_ghosts.push(el.into_any_element());
                    }

                    let sel_flags = &line_bold_flags
                        [loc_start.min(line_bold_flags.len())..loc_end.min(line_bold_flags.len())];
                    let sel_runs =
                        crate::text::styles::split_text_into_styled_runs(&sel_content, sel_flags);
                    let mut sel_ghosts = Vec::new();
                    for run in &sel_runs {
                        let disp = run.text.replace(' ', "\u{00A0}");
                        let el = div()
                            .text_color(rgba(0x00000000))
                            .font_weight(if run.is_bold {
                                gpui::FontWeight::BOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .child(disp);
                        sel_ghosts.push(el.into_any_element());
                    }

                    row = row.child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(1.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .children(before_ghosts)
                            .child(
                                div()
                                    .bg(rgb(0x0078d4))
                                    .rounded(px(2.0))
                                    .h(px(self.canvas_selection_height()))
                                    .flex()
                                    .items_center()
                                    .children(sel_ghosts),
                            ),
                    );
                } else if line_len == 0
                    && sel_start <= line_global_start
                    && sel_end > line_global_start
                {
                    row = row.child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(1.0))
                            .w(px(6.0))
                            .h(px(self.canvas_selection_height()))
                            .bg(rgb(0x0078d4))
                            .rounded(px(2.0)),
                    );
                }
            }

            let line_italic =
                crate::text::styles::slice_flags(&italic_flags, line_global_start, line_global_end, line_len);
            let line_underline = crate::text::styles::slice_flags(
                &underline_flags,
                line_global_start,
                line_global_end,
                line_len,
            );
            let line_strike =
                crate::text::styles::slice_flags(&strike_flags, line_global_start, line_global_end, line_len);
            let runs = crate::text::styles::split_text_into_full_runs(
                line,
                &line_bold_flags,
                &line_italic,
                &line_underline,
                &line_strike,
            );
            let mut line_elements = Vec::new();
            if runs.is_empty() {
                line_elements.push(
                    div()
                        .text_color(rgb(TEXT_PRIMARY))
                        .child("\u{00A0}")
                        .into_any_element(),
                );
            } else {
                for run in &runs {
                    let color = if run.is_bold { 0xffffff } else { TEXT_PRIMARY };
                    line_elements.push(crate::canvas::text_editor::styled_run_element(
                        run,
                        self.canvas_body_font_size,
                        color,
                        true,
                    ));
                }
            }

            row = row.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .children(line_elements),
            );

            // If text is active and cursor is in this line, render cursor overlay
            let is_cursor_in_line =
                cursor_idx >= line_global_start && cursor_idx <= line_global_end;

            if is_text_active && is_cursor_in_line {
                let local_cursor = cursor_idx.saturating_sub(line_global_start).min(line_len);
                let line_chars: Vec<char> = line.chars().collect();
                let before_prefix: String = line_chars[..local_cursor].iter().collect();
                let before_flags = &line_bold_flags[..local_cursor.min(line_bold_flags.len())];
                let prefix_runs =
                    crate::text::styles::split_text_into_styled_runs(&before_prefix, before_flags);

                let mut ghost_elements = Vec::new();
                for run in &prefix_runs {
                    let disp = run.text.replace(' ', "\u{00A0}");
                    let el = div()
                        .text_color(rgba(0x00000000))
                        .font_weight(if run.is_bold {
                            gpui::FontWeight::BOLD
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(disp);
                    ghost_elements.push(el.into_any_element());
                }

                let show_caret = self.is_selecting_viewer_text || self.cursor_visible;

                row = row.child(
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(2.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .children(ghost_elements)
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(self.canvas_cursor_height()))
                                .bg(if show_caret {
                                    rgb(0x0078d4)
                                } else {
                                    rgba(0x00000000)
                                })
                                .flex_shrink_0()
                                .ml(px(-1.0)),
                        ),
                );
            }

            // Per-line mouse down handler for exact line targeting
            let seg_id_for_line = seg_id.clone();
            let line_str_owned = line.to_string();
            let line_flags_for_click = line_bold_flags.clone();

            let row = row
                .id(("viewer-line-row", hash_str(&seg_id).wrapping_add(line_idx)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        this.viewer_active_text_block_id = Some(seg_id_for_line.clone());

                        let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                        let rel_x = (event.position.x.as_f32()
                            - sidebar_w
                            - this.pan_x
                            - item_x
                            - 5.0)
                            .max(0.0);
                        let local_idx = calculate_line_text_offset_with_bold_and_font(
                            rel_x,
                            &line_str_owned,
                            Some(&line_flags_for_click),
                            this.canvas_body_font_size,
                            this.font_type(),
                        );
                        let click_idx = (line_global_start + local_idx).min(total_chars);
                        this.viewer_text_cursor = click_idx;
                        this.viewer_text_anchor = Some(click_idx);
                        this.is_selecting_viewer_text = true;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                );

            line_rows.push(row.into_any_element());
            global_offset += line_len + 1;
        }

        let seg_id_down = seg_id.clone();
        let text_for_down = text_content.clone();
        let bold_flags_down = bold_flags.clone();
        let bold_flags_move = bold_flags.clone();
        let seg_id_move = seg_id.clone();

        div()
            .absolute()
            .left(px(item_x + self.pan_x))
            .top(px(item_y + self.pan_y))
            .w(px(item_w))
            .p(px(5.0))
            .text_size(px(self.canvas_body_font_size))
            .text_color(rgb(TEXT_PRIMARY))
            .cursor_text()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    this.focus_handle.focus(window, cx);
                    this.is_panning = false;
                    this.pan_start_mouse = None;
                    this.pan_start_val = None;
                    this.viewer_active_text_block_id = Some(seg_id_down.clone());

                    let click_idx = calculate_canvas_text_offset_full(
                        event.position,
                        this.is_sidebar_open,
                        this.pan_x,
                        this.pan_y,
                        item_x,
                        item_y,
                        this.canvas_top_y,
                        &text_for_down,
                        Some(&bold_flags_down),
                        5.0,
                        5.0,
                        item_w,
                        this.canvas_body_font_size,
                        this.font_type(),
                    );
                    this.viewer_text_cursor = click_idx;
                    this.viewer_text_anchor = Some(click_idx);
                    this.is_selecting_viewer_text = true;
                    this.cursor_visible = true;
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(move |this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.is_selecting_viewer_text {
                    if let Some(ref active_id) = this.viewer_active_text_block_id {
                        if active_id == &seg_id_move {
                            let drag_idx = calculate_canvas_text_offset_full(
                                event.position,
                                this.is_sidebar_open,
                                this.pan_x,
                                this.pan_y,
                                item_x,
                                item_y,
                                this.canvas_top_y,
                                &text_content,
                                Some(&bold_flags_move),
                                5.0,
                                5.0,
                                item_w,
                                this.canvas_body_font_size,
                                this.font_type(),
                            );
                            this.viewer_text_cursor = drag_idx;
                            cx.notify();
                            cx.stop_propagation();
                        }
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if this.is_selecting_viewer_text {
                        if this.viewer_text_anchor == Some(this.viewer_text_cursor) {
                            this.viewer_text_anchor = None;
                        }
                        this.is_selecting_viewer_text = false;
                        cx.notify();
                        cx.stop_propagation();
                    }
                }),
            )
            .children(line_rows)
            .into_any_element()
    }
}
