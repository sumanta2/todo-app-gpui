//! One editable text segment, either a plain text block or a text block inside a mixed item.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, MouseButton};

use crate::app::NotesApp;
use crate::constants::colors::note_ink;
use crate::models::{CanvasItem, ContentBlock, TextStyleSpans};

impl NotesApp {
    /// Looks up the live text and bold spans for a text segment, whether it belongs to a plain
    /// `CanvasItem::Text` (`block_index` is `None`) or to a `Text` block inside a
    /// `CanvasItem::Mixed` (`block_index` is `Some(idx)`), along with the owning item's position.
    fn lookup_segment_text(
        &self,
        item_id: &str,
        block_index: Option<usize>,
    ) -> Option<(String, TextStyleSpans, f32, f32)> {
        for item in &self.edit_canvas_items {
            match item {
                CanvasItem::Text(t) if block_index.is_none() && t.id == item_id => {
                    return Some((
                        t.text.clone(),
                        TextStyleSpans {
                            bold: t.bold_spans.clone(),
                            italic: t.italic_spans.clone(),
                            underline: t.underline_spans.clone(),
                            strike: t.strike_spans.clone(),
                            font_runs: t.font_runs.clone(),
                            line_layouts: t.line_layouts.clone(),
                        },
                        t.x,
                        t.y,
                    ));
                }
                CanvasItem::Mixed(m) if m.id == item_id => {
                    if let Some(idx) = block_index {
                        if let Some(ContentBlock::Text {
                            text,
                            bold_spans,
                            italic_spans,
                            underline_spans,
                            strike_spans,
                            font_runs,
                            line_layouts,
                        }) = m.blocks.get(idx)
                        {
                            return Some((
                                text.clone(),
                                TextStyleSpans {
                                    bold: bold_spans.clone(),
                                    italic: italic_spans.clone(),
                                    underline: underline_spans.clone(),
                                    strike: strike_spans.clone(),
                                    font_runs: font_runs.clone(),
                                    line_layouts: line_layouts.clone(),
                                },
                                m.x,
                                m.y,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Copies a segment into the live editor buffers and returns its canvas position.
    pub(crate) fn capture_segment(
        &mut self,
        item_id: &str,
        block_index: Option<usize>,
    ) -> Option<(f32, f32)> {
        let (text, styles, x, y) = self.lookup_segment_text(item_id, block_index)?;
        let len = text.chars().count();
        self.load_body_styles(&styles, len);
        self.edit_body = text;
        self.bind_line_layouts();
        Some((x, y))
    }

    /// Renders one text segment: either the whole body of a plain `CanvasItem::Text`, or a
    /// single `Text` block that lives inside a `CanvasItem::Mixed`, right after an image.
    ///
    /// This is shared by both cases so an image inserted mid-typing can split the text into a
    /// "before" and "after" segment without duplicating the click/selection/editing logic.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_text_segment(
        &self,
        id_num: usize,
        item_id: String,
        block_index: Option<usize>,
        text: String,
        styles: TextStyleSpans,
        item_x: f32,
        item_y: f32,
        textbox_width: f32,
        is_active: bool,
        is_body_focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let item_pos = (item_x, item_y);
        let zoom = self.canvas_zoom.max(0.25);
        let textbox_width = textbox_width * zoom;
        let body_font = self.canvas_body_font_size * zoom;

        if is_active {
            let text_editor = crate::canvas::text_editor::TextEditor {
                text: self.edit_body.clone(),
                bold_flags: self.edit_body_bold.clone(),
                italic_flags: self.edit_body_italic.clone(),
                underline_flags: self.edit_body_underline.clone(),
                strike_flags: self.edit_body_strike.clone(),
                font_families: self.edit_body_font_family.clone(),
                font_sizes: self.edit_body_font_size.clone(),
                font_colors: self.edit_body_font_color.clone(),
                bg_colors: self.edit_body_bg_color.clone(),
                line_layouts: self.edit_body_line_layouts.clone(),
                zoom: self.canvas_zoom,
                cursor_visible: self.cursor_visible,
                font_size: body_font,
            };

            let id_for_line = item_id.clone();
            let id_for_down_left = item_id.clone();
            let id_for_down_right = item_id.clone();
            let block_index_for_line = block_index;
            let block_index_for_down_left = block_index;
            let block_index_for_down_right = block_index;
            let selection_layer = self.body_selection_layer(cx);

            let line_wrapper =
                |line_idx: usize, line_start: usize, _line_str: &str, row: gpui::Div| {
                    let id_for_line = id_for_line.clone();
                    row.id(("editor-line-row", id_num.wrapping_add(line_idx)))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                                this.focus_handle.focus(window, cx);
                                this.is_panning = false;
                                this.pan_start_mouse = None;
                                this.pan_start_val = None;
                                let (item_x, _item_y, same_block) = this.place_body_pointer(
                                    &id_for_line,
                                    block_index_for_line,
                                    item_pos,
                                );

                                let sidebar_w = this.layout_sidebar_w();
                                let rel_x =
                                    ((event.position.x.as_f32() - sidebar_w - this.pan_x - 6.0)
                                        / this.canvas_zoom.max(0.25)
                                        - item_x)
                                        .max(0.0);
                                let click_idx = this.body_index_on_line(line_start, rel_x);

                                this.edit_body_cursor = click_idx;
                                this.edit_body_anchor = Some(click_idx);
                                this.is_selecting_body = true;
                                this.show_body_pointer(same_block, cx);
                                cx.stop_propagation();
                            }),
                        )
                        .into_any_element()
                };

            div()
                .id(("text-content", id_num))
                .relative()
                .p(px(5.0))
                .text_size(px(body_font))
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        let (item_x, item_y, same_block) = this.place_body_pointer(
                            &id_for_down_left,
                            block_index_for_down_left,
                            item_pos,
                        );
                        let click_idx =
                            this.body_index_at_mouse(event.position, item_x, item_y, 6.0, 18.0);
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.show_body_pointer(same_block, cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        let (item_x, item_y, same_block) = this.place_body_pointer(
                            &id_for_down_right,
                            block_index_for_down_right,
                            item_pos,
                        );
                        let click_idx =
                            this.body_index_at_mouse(event.position, item_x, item_y, 6.0, 18.0);
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.show_body_pointer(same_block, cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if this.is_selecting_body {
                            this.end_body_pointer(cx);
                            cx.stop_propagation();
                        }
                    }),
                )
                .on_mouse_up(
                    MouseButton::Right,
                    cx.listener(move |this, _, _, cx| {
                        if this.is_selecting_body {
                            this.end_body_pointer(cx);
                            cx.stop_propagation();
                        }
                    }),
                )
                .child(selection_layer)
                .child(
                    div()
                        .w(px(
                            textbox_width - crate::constants::typography::TEXT_COLUMN_INSET
                        ))
                        .child(
                            text_editor
                                .render_editor_with_line_wrapper(is_body_focused, line_wrapper),
                        ),
                )
                .into_any_element()
        } else {
            let id_for_inactive_left = item_id.clone();
            let id_for_inactive_right = item_id.clone();
            let block_index_for_inactive_left = block_index;
            let block_index_for_inactive_right = block_index;
            let item_pos_move = item_pos;
            let total_chars = text.chars().count();
            let bold_flags = crate::text::styles::spans_to_bool_vec(&styles.bold, total_chars);
            let italic_flags = crate::text::styles::spans_to_bool_vec(&styles.italic, total_chars);
            let underline_flags =
                crate::text::styles::spans_to_bool_vec(&styles.underline, total_chars);
            let strike_flags = crate::text::styles::spans_to_bool_vec(&styles.strike, total_chars);
            let (font_families, font_sizes, font_colors, bg_colors) =
                crate::text::styles::font_runs_to_vecs(&styles.font_runs, total_chars);

            let mut inactive_line_rows = Vec::new();
            let mut global_offset = 0;
            for (line_idx, line_str) in text.split('\n').enumerate() {
                let line_len = line_str.chars().count();
                let line_start = global_offset;
                let line_end = global_offset + line_len;
                let id_for_line = item_id.clone();
                let block_index_for_line = block_index;

                let line_flags =
                    crate::text::styles::slice_flags(&bold_flags, line_start, line_end, line_len);
                let line_italic =
                    crate::text::styles::slice_flags(&italic_flags, line_start, line_end, line_len);
                let line_underline = crate::text::styles::slice_flags(
                    &underline_flags,
                    line_start,
                    line_end,
                    line_len,
                );
                let line_strike =
                    crate::text::styles::slice_flags(&strike_flags, line_start, line_end, line_len);

                let line_font_families = if line_start < font_families.len() {
                    let end = line_end.min(font_families.len());
                    let mut values = font_families[line_start..end].to_vec();
                    values.resize(line_len, 0);
                    values
                } else {
                    vec![0u8; line_len]
                };
                let line_font_sizes = if line_start < font_sizes.len() {
                    let end = line_end.min(font_sizes.len());
                    let mut values = font_sizes[line_start..end].to_vec();
                    values.resize(line_len, self.canvas_body_font_size);
                    values
                } else {
                    vec![self.canvas_body_font_size; line_len]
                };
                let line_font_colors =
                    crate::text::styles::slice_colors(&font_colors, line_start, line_end, line_len);
                let line_bg_colors =
                    crate::text::styles::slice_colors(&bg_colors, line_start, line_end, line_len);
                let runs = crate::text::styles::split_text_into_full_runs(
                    line_str,
                    &line_flags,
                    &line_italic,
                    &line_underline,
                    &line_strike,
                    &line_font_families,
                    &line_font_sizes,
                    &line_font_colors,
                    &line_bg_colors,
                );
                let mut line_runs_els = Vec::new();
                if runs.is_empty() {
                    line_runs_els.push(
                        div()
                            .text_color(rgb(note_ink()))
                            .child("\u{00A0}")
                            .into_any_element(),
                    );
                } else {
                    for run in &runs {
                        let color = note_ink();
                        line_runs_els.push(crate::canvas::text_editor::styled_run_element(
                            run, color, true, zoom,
                        ));
                    }
                }

                let line_el = crate::canvas::text_editor::style_line_row(
                    div()
                        .h(px(self.canvas_line_height() * zoom))
                        .line_height(px(self.canvas_line_height() * zoom))
                        .flex()
                        .items_center(),
                    styles
                        .line_layouts
                        .get(line_idx)
                        .copied()
                        .unwrap_or_default(),
                    zoom,
                )
                .id(("inactive-line-row", id_num.wrapping_add(line_idx)))
                .children(line_runs_els)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        let (item_x, _item_y, same_block) = this.place_body_pointer(
                            &id_for_line,
                            block_index_for_line,
                            item_pos_move,
                        );

                        let sidebar_w = this.layout_sidebar_w();
                        let rel_x = ((event.position.x.as_f32() - sidebar_w - this.pan_x - 6.0)
                            / this.canvas_zoom.max(0.25)
                            - item_x)
                            .max(0.0);
                        let click_idx = this.body_index_on_line(line_start, rel_x);

                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.show_body_pointer(same_block, cx);
                        cx.stop_propagation();
                    }),
                );

                inactive_line_rows.push(line_el.into_any_element());
                global_offset += line_len + 1;
            }

            div()
                .id(("text-content", id_num))
                .p(px(5.0))
                .text_size(px(body_font))
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        let (item_x, item_y, same_block) = this.place_body_pointer(
                            &id_for_inactive_left,
                            block_index_for_inactive_left,
                            item_pos,
                        );
                        let click_idx =
                            this.body_index_at_mouse(event.position, item_x, item_y, 6.0, 18.0);
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.show_body_pointer(same_block, cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        let (item_x, item_y, same_block) = this.place_body_pointer(
                            &id_for_inactive_right,
                            block_index_for_inactive_right,
                            item_pos,
                        );
                        let click_idx =
                            this.body_index_at_mouse(event.position, item_x, item_y, 6.0, 18.0);
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.show_body_pointer(same_block, cx);
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .w(px(
                            textbox_width - crate::constants::typography::TEXT_COLUMN_INSET
                        ))
                        .flex()
                        .flex_col()
                        .children(inactive_line_rows),
                )
                .into_any_element()
        }
    }
}
