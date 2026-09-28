use gpui::{div, img, prelude::*, px, rgb, AnyElement, Context, MouseButton};
use std::sync::Arc;

use crate::app::NotesApp;
use crate::constants::{colors::TEXT_PRIMARY, typography::SMALL_ICON_FONT_SIZE};
use crate::models::{ActiveField, CanvasItem, ContentBlock, TextStyleSpans};
use crate::text_selection::{
    calculate_canvas_drag_offset_full, calculate_canvas_text_offset_full,
    calculate_line_text_offset_with_bold_and_font,
};

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
                        }) = m.blocks.get(idx)
                        {
                            return Some((
                                text.clone(),
                                TextStyleSpans {
                                    bold: bold_spans.clone(),
                                    italic: italic_spans.clone(),
                                    underline: underline_spans.clone(),
                                    strike: strike_spans.clone(),
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
    fn capture_segment(&mut self, item_id: &str, block_index: Option<usize>) -> Option<(f32, f32)> {
        let (text, styles, x, y) = self.lookup_segment_text(item_id, block_index)?;
        let len = text.chars().count();
        self.load_body_styles(&styles, len);
        self.edit_body = text;
        Some((x, y))
    }

    /// Looks up an item's position and text-wrap width regardless of whether it is a plain
    /// `Text` item or a `Mixed` box; used while drag-selecting text so the math always follows
    /// the box that currently owns the active text segment.
    fn lookup_item_pos_width(&self, item_id: &str) -> Option<(f32, f32, f32)> {
        for item in &self.edit_canvas_items {
            match item {
                CanvasItem::Text(t) if t.id == item_id => {
                    return Some((t.x, t.y, t.width.unwrap_or(250.0)));
                }
                CanvasItem::Mixed(m) if m.id == item_id => {
                    return Some((m.x, m.y, m.width.unwrap_or(250.0)));
                }
                _ => {}
            }
        }
        None
    }

    /// Renders one text segment: either the whole body of a plain `CanvasItem::Text`, or a
    /// single `Text` block that lives inside a `CanvasItem::Mixed`, right after an image.
    ///
    /// This is shared by both cases so an image inserted mid-typing can split the text into a
    /// "before" and "after" segment without duplicating the click/selection/editing logic.
    #[allow(clippy::too_many_arguments)]
    fn render_text_segment(
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

        if is_active {
            let text_editor = crate::canvas::text_editor::TextEditor {
                text: self.edit_body.clone(),
                bold_flags: self.edit_body_bold.clone(),
                italic_flags: self.edit_body_italic.clone(),
                underline_flags: self.edit_body_underline.clone(),
                strike_flags: self.edit_body_strike.clone(),
                cursor: self.edit_body_cursor,
                anchor: self.edit_body_anchor,
                focus_handle: self.focus_handle.clone(),
                is_selecting: self.is_selecting_body,
                cursor_visible: self.cursor_visible,
                font_size: self.canvas_body_font_size,
            };

            let id_for_line = item_id.clone();
            let id_for_down_left = item_id.clone();
            let id_for_down_right = item_id.clone();
            let block_index_for_line = block_index;
            let block_index_for_down_left = block_index;
            let block_index_for_down_right = block_index;
            let item_pos_move = item_pos;

            let line_wrapper = |line_idx: usize, line_start: usize, line_str: &str, row: gpui::Div| {
                let id_for_line = id_for_line.clone();
                let line_str_owned = line_str.to_string();
                row.id(("editor-line-row", id_num.wrapping_add(line_idx)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                            this.focus_handle.focus(window, cx);
                            this.is_panning = false;
                            this.pan_start_mouse = None;
                            this.pan_start_val = None;
                            this.sync_active_text_block();
                            this.active_text_block_id = Some(id_for_line.clone());
                            this.active_block_index = block_index_for_line;
                            this.active_field = ActiveField::Body;

                            let item_x = this
                                .capture_segment(&id_for_line, block_index_for_line)
                                .map(|(x, _)| x)
                                .unwrap_or(item_pos.0);

                            let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                            let rel_x = (event.position.x.as_f32() - sidebar_w - this.pan_x - item_x - 6.0).max(0.0);
                            let line_len = line_str_owned.chars().count();
                            let line_flags = if line_start < this.edit_body_bold.len() {
                                let end = (line_start + line_len).min(this.edit_body_bold.len());
                                Some(&this.edit_body_bold[line_start..end])
                            } else {
                                None
                            };
                            let local_idx = calculate_line_text_offset_with_bold_and_font(
                                rel_x,
                                &line_str_owned,
                                line_flags,
                                this.canvas_body_font_size,
                                this.font_type(),
                            );
                            let click_idx = (line_start + local_idx).min(this.edit_body.chars().count());

                            this.edit_body_cursor = click_idx;
                            this.edit_body_anchor = Some(click_idx);
                            this.is_selecting_body = true;
                            this.cursor_visible = true;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .into_any_element()
            };

            div()
                .id(("text-content", id_num))
                .p(px(5.0))
                .text_size(px(self.canvas_body_font_size))
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        this.sync_active_text_block();
                        this.active_text_block_id = Some(id_for_down_left.clone());
                        this.active_block_index = block_index_for_down_left;
                        this.active_field = ActiveField::Body;
                        let (item_x, item_y) = this
                            .capture_segment(&id_for_down_left, block_index_for_down_left)
                            .unwrap_or(item_pos);
                        let click_idx = calculate_canvas_text_offset_full(
                            event.position,
                            this.is_sidebar_open,
                            this.pan_x,
                            this.pan_y,
                            item_x,
                            item_y,
                            this.canvas_top_y,
                            &this.edit_body,
                            Some(&this.edit_body_bold),
                            6.0,
                            18.0,
                            textbox_width,
                            this.canvas_body_font_size,
                            this.font_type(),
                        );
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.cursor_visible = true;
                        cx.notify();
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
                        this.sync_active_text_block();
                        this.active_text_block_id = Some(id_for_down_right.clone());
                        this.active_block_index = block_index_for_down_right;
                        this.active_field = ActiveField::Body;
                        let (item_x, item_y) = this
                            .capture_segment(&id_for_down_right, block_index_for_down_right)
                            .unwrap_or(item_pos);
                        let click_idx = calculate_canvas_text_offset_full(
                            event.position,
                            this.is_sidebar_open,
                            this.pan_x,
                            this.pan_y,
                            item_x,
                            item_y,
                            this.canvas_top_y,
                            &this.edit_body,
                            Some(&this.edit_body_bold),
                            6.0,
                            18.0,
                            textbox_width,
                            this.canvas_body_font_size,
                            this.font_type(),
                        );
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_move(cx.listener(move |this, event: &gpui::MouseMoveEvent, _, cx| {
                    if this.is_selecting_body {
                        let (item_x, item_y, item_w) = if let Some(ref active_id) = this.active_text_block_id {
                            this.lookup_item_pos_width(active_id)
                                .unwrap_or((item_pos_move.0, item_pos_move.1, textbox_width))
                        } else {
                            (item_pos_move.0, item_pos_move.1, textbox_width)
                        };
                        let drag_idx = calculate_canvas_drag_offset_full(
                            event.position,
                            this.is_sidebar_open,
                            this.pan_x,
                            this.pan_y,
                            item_x,
                            item_y,
                            this.canvas_top_y,
                            &this.edit_body,
                            Some(&this.edit_body_bold),
                            item_w,
                            this.canvas_body_font_size,
                            this.font_type(),
                        );
                        this.edit_body_cursor = drag_idx;
                        cx.notify();
                        cx.stop_propagation();
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if this.is_selecting_body {
                            if this.edit_body_anchor == Some(this.edit_body_cursor) {
                                this.edit_body_anchor = None;
                            }
                            this.is_selecting_body = false;
                            cx.notify();
                            cx.stop_propagation();
                        }
                    }),
                )
                .on_mouse_up(
                    MouseButton::Right,
                    cx.listener(move |this, _, _, cx| {
                        if this.is_selecting_body {
                            if this.edit_body_anchor == Some(this.edit_body_cursor) {
                                this.edit_body_anchor = None;
                            }
                            this.is_selecting_body = false;
                            cx.notify();
                            cx.stop_propagation();
                        }
                    }),
                )
                .child(div().w(px(textbox_width - 16.0)).child(text_editor.render_editor_with_line_wrapper(is_body_focused, line_wrapper)))
                .into_any_element()
        } else {
            let id_for_inactive_left = item_id.clone();
            let id_for_inactive_right = item_id.clone();
            let block_index_for_inactive_left = block_index;
            let block_index_for_inactive_right = block_index;
            let item_pos_move = item_pos;
            let total_chars = text.chars().count();
            let bold_flags = crate::helpers::spans_to_bool_vec(&styles.bold, total_chars);
            let italic_flags = crate::helpers::spans_to_bool_vec(&styles.italic, total_chars);
            let underline_flags = crate::helpers::spans_to_bool_vec(&styles.underline, total_chars);
            let strike_flags = crate::helpers::spans_to_bool_vec(&styles.strike, total_chars);

            let mut inactive_line_rows = Vec::new();
            let mut global_offset = 0;
            for (line_idx, line_str) in text.split('\n').enumerate() {
                let line_len = line_str.chars().count();
                let line_start = global_offset;
                let line_end = global_offset + line_len;
                let line_str_owned = line_str.to_string();
                let id_for_line = item_id.clone();
                let block_index_for_line = block_index;

                let line_flags = crate::helpers::slice_flags(&bold_flags, line_start, line_end, line_len);
                let line_italic = crate::helpers::slice_flags(&italic_flags, line_start, line_end, line_len);
                let line_underline =
                    crate::helpers::slice_flags(&underline_flags, line_start, line_end, line_len);
                let line_strike = crate::helpers::slice_flags(&strike_flags, line_start, line_end, line_len);

                let runs = crate::helpers::split_text_into_full_runs(
                    line_str,
                    &line_flags,
                    &line_italic,
                    &line_underline,
                    &line_strike,
                );
                let mut line_runs_els = Vec::new();
                if runs.is_empty() {
                    line_runs_els.push(
                        div()
                            .text_color(rgb(TEXT_PRIMARY))
                            .child("\u{00A0}")
                            .into_any_element(),
                    );
                } else {
                    for run in &runs {
                        let color = if run.is_bold { 0xffffff } else { TEXT_PRIMARY };
                        line_runs_els.push(crate::canvas::text_editor::styled_run_element(
                            run,
                            self.canvas_body_font_size,
                            color,
                            true,
                        ));
                    }
                }

                let line_el = div()
                    .id(("inactive-line-row", id_num.wrapping_add(line_idx)))
                    .min_h(px(self.canvas_line_height()))
                    .flex()
                    .items_center()
                    .children(line_runs_els)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                            this.focus_handle.focus(window, cx);
                            this.is_panning = false;
                            this.pan_start_mouse = None;
                            this.pan_start_val = None;
                            this.sync_active_text_block();
                            this.active_text_block_id = Some(id_for_line.clone());
                            this.active_block_index = block_index_for_line;
                            this.active_field = ActiveField::Body;

                            let item_x = this
                                .capture_segment(&id_for_line, block_index_for_line)
                                .map(|(x, _)| x)
                                .unwrap_or(item_pos_move.0);

                            let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                            let rel_x = (event.position.x.as_f32() - sidebar_w - this.pan_x - item_x - 6.0).max(0.0);
                            let line_len = line_str_owned.chars().count();
                            let line_flags = if line_start < this.edit_body_bold.len() {
                                let end = (line_start + line_len).min(this.edit_body_bold.len());
                                Some(&this.edit_body_bold[line_start..end])
                            } else {
                                None
                            };
                            let local_idx = calculate_line_text_offset_with_bold_and_font(
                                rel_x,
                                &line_str_owned,
                                line_flags,
                                this.canvas_body_font_size,
                                this.font_type(),
                            );
                            let click_idx = (line_start + local_idx).min(this.edit_body.chars().count());

                            this.edit_body_cursor = click_idx;
                            this.edit_body_anchor = Some(click_idx);
                            this.is_selecting_body = true;
                            this.cursor_visible = true;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    );

                inactive_line_rows.push(line_el.into_any_element());
                global_offset += line_len + 1;
            }

            div()
                .id(("text-content", id_num))
                .p(px(5.0))
                .text_size(px(self.canvas_body_font_size))
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        this.sync_active_text_block();
                        this.active_text_block_id = Some(id_for_inactive_left.clone());
                        this.active_block_index = block_index_for_inactive_left;
                        this.active_field = ActiveField::Body;
                        let (item_x, item_y) = this
                            .capture_segment(&id_for_inactive_left, block_index_for_inactive_left)
                            .unwrap_or(item_pos);
                        let click_idx = calculate_canvas_text_offset_full(
                            event.position,
                            this.is_sidebar_open,
                            this.pan_x,
                            this.pan_y,
                            item_x,
                            item_y,
                            this.canvas_top_y,
                            &this.edit_body,
                            Some(&this.edit_body_bold),
                            6.0,
                            18.0,
                            textbox_width,
                            this.canvas_body_font_size,
                            this.font_type(),
                        );
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.cursor_visible = true;
                        cx.notify();
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
                        this.sync_active_text_block();
                        this.active_text_block_id = Some(id_for_inactive_right.clone());
                        this.active_block_index = block_index_for_inactive_right;
                        this.active_field = ActiveField::Body;
                        let (item_x, item_y) = this
                            .capture_segment(&id_for_inactive_right, block_index_for_inactive_right)
                            .unwrap_or(item_pos);
                        let click_idx = calculate_canvas_text_offset_full(
                            event.position,
                            this.is_sidebar_open,
                            this.pan_x,
                            this.pan_y,
                            item_x,
                            item_y,
                            this.canvas_top_y,
                            &this.edit_body,
                            Some(&this.edit_body_bold),
                            6.0,
                            18.0,
                            textbox_width,
                            this.canvas_body_font_size,
                            this.font_type(),
                        );
                        this.edit_body_cursor = click_idx;
                        this.edit_body_anchor = Some(click_idx);
                        this.is_selecting_body = true;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .w(px(textbox_width - 16.0))
                        .flex()
                        .flex_col()
                        .children(inactive_line_rows),
                )
                .into_any_element()
        }
    }

    /// Renders all editable canvas blocks such as text boxes, images, and combined
    /// image+text boxes.
    ///
    /// Each item receives drag/resize handlers, delete controls, and the active text editing
    /// behavior when it is selected as the currently focused body block.
    pub(crate) fn render_canvas_elements(
        &mut self,
        is_body_focused: bool,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut canvas_elements = Vec::new();

        for (_, item) in self.edit_canvas_items.iter().enumerate() {
            let item_id = match item {
                CanvasItem::Text(t) => t.id.clone(),
                CanvasItem::Image(img) => img.id.clone(),
                CanvasItem::Mixed(m) => m.id.clone(),
            };
            let drag_id = item_id.clone();
            let resize_id = item_id.clone();
            let delete_id = item_id.clone();

            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            item_id.hash(&mut hasher);
            let id_num = hasher.finish() as usize;

            let is_active = match item {
                CanvasItem::Text(t) => {
                    self.active_text_block_id == Some(t.id.clone())
                        && self.active_field == ActiveField::Body
                }
                CanvasItem::Mixed(m) => {
                    self.active_text_block_id == Some(m.id.clone())
                        && self.active_field == ActiveField::Body
                }
                CanvasItem::Image(_) => false,
            };

            match item {
                CanvasItem::Text(t) => {
                    let textbox_width = if is_active {
                        let max_line_len = self
                            .edit_body
                            .lines()
                            .map(|line| line.chars().count())
                            .max()
                            .unwrap_or(0);
                        let line_text_w = max_line_len as f32 * 6.2;
                        let needed_width = line_text_w + 20.0;
                        let base_w = t.width.unwrap_or(250.0);
                        let desired_w = needed_width.max(base_w).max(250.0);

                        let sidebar_w = if self.is_sidebar_open { 220.0 } else { 44.0 };
                        let page_sidebar_w = 180.0;
                        let canvas_visible_w =
                            (self.window_w - sidebar_w - page_sidebar_w - 30.0).max(300.0);
                        let canvas_max_right = canvas_visible_w - self.pan_x;
                        let max_allowed_width = (canvas_max_right - t.x).max(150.0);
                        desired_w.min(max_allowed_width)
                    } else {
                        t.width.unwrap_or(250.0)
                    };
                    let drag_id_clone = drag_id.clone();
                    let t_pos = (t.x, t.y);
                    let text = t.text.clone();
                    let styles = TextStyleSpans {
                        bold: t.bold_spans.clone(),
                        italic: t.italic_spans.clone(),
                        underline: t.underline_spans.clone(),
                        strike: t.strike_spans.clone(),
                    };

                    let mut inner_block = div()
                        .flex()
                        .flex_col()
                        .w(px(textbox_width))
                        .bg(rgb(0x1e1e1e))
                        .border_1()
                        .border_color(if is_active {
                            rgb(0x0078d4)
                        } else {
                            rgb(0x3d3d3d)
                        })
                        .rounded(px(6.0))
                        .overflow_hidden();

                    // Drag Handle header
                    inner_block = inner_block.child(
                        div()
                            .id(("drag-header", id_num))
                            .h(px(12.0))
                            .bg(rgb(0x2d2d2d))
                            .cursor_move()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.drag_item_id = Some(drag_id_clone.clone());
                                    this.drag_start_mouse = Some(event.position);
                                    this.drag_start_item_pos = Some(t_pos);
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            )
                            .flex()
                            .justify_between()
                            .items_center()
                            .px(px(6.0))
                            .child(div())
                            .child(
                                div()
                                    .id(("delete-btn", id_num))
                                    .text_size(px(SMALL_ICON_FONT_SIZE))
                                    .line_height(gpui::relative(1.0))
                                    .text_color(rgb(0xff6b6b))
                                    .hover(|s| s.text_color(rgb(0xff0000)))
                                    .cursor_pointer()
                                    .child("×")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.edit_canvas_items.retain(|item| match item {
                                                CanvasItem::Text(tx) => tx.id != delete_id,
                                                _ => true,
                                            });
                                            if this.active_text_block_id == Some(delete_id.clone())
                                            {
                                                this.active_text_block_id = None;
                                                this.active_block_index = None;
                                                this.edit_body = String::new();
                                                this.reset_body_styles();
                                                this.edit_body_cursor = 0;
                                            }
                                            cx.notify();
                                            cx.stop_propagation();
                                        }),
                                    ),
                            ),
                    );

                    let segment = self.render_text_segment(
                        id_num, item_id.clone(), None, text, styles, t_pos.0, t_pos.1,
                        textbox_width, is_active, is_body_focused, cx,
                    );
                    inner_block = inner_block.child(segment);

                    // corner resize handle container
                    let mut wrapper = div()
                        .absolute()
                        .left(px(t_pos.0 + self.pan_x))
                        .top(px(t_pos.1 + self.pan_y))
                        .child(inner_block);

                    // Resize Handle on active text box (corner resize)
                    if is_active {
                        wrapper = wrapper.child(
                            div()
                                .id(("resize-handle", id_num))
                                .absolute()
                                .right(px(-4.0))
                                .bottom(px(-4.0))
                                .w(px(10.0))
                                .h(px(10.0))
                                .bg(rgb(0x0078d4))
                                .cursor_e_resize()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(
                                        move |this, event: &gpui::MouseDownEvent, _, cx| {
                                            this.resize_item_id = Some(resize_id.clone());
                                            this.resize_block_index = None;
                                            this.resize_start_mouse = Some(event.position);
                                            this.resize_start_size = Some((textbox_width, 100.0));
                                            cx.notify();
                                            cx.stop_propagation();
                                        },
                                    ),
                                ),
                        );
                    }

                    canvas_elements.push(wrapper.into_any_element());
                }
                CanvasItem::Image(img_item) => {
                    if let Some(img_data) = self.decrypt_image(&img_item.path) {
                        let source = gpui::ImageSource::Image(Arc::new(img_data));
                        let drag_id_clone = drag_id.clone();
                        let img_pos = (img_item.x, img_item.y);
                        let img_size = (img_item.width, img_item.height);

                        let mut inner_block = div()
                            .flex()
                            .flex_col()
                            .w(px(img_item.width))
                            .h(px(img_item.height + 10.0))
                            .bg(rgb(0x1e1e1e))
                            .border_1()
                            .border_color(rgb(0x3d3d3d))
                            .rounded(px(6.0))
                            .overflow_hidden();

                        // Drag Handle header
                        inner_block = inner_block.child(
                            div()
                                .id(("drag-header", id_num))
                                .h(px(10.0))
                                .bg(rgb(0x2d2d2d))
                                .cursor_move()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(
                                        move |this, event: &gpui::MouseDownEvent, _, cx| {
                                            this.drag_item_id = Some(drag_id_clone.clone());
                                            this.drag_start_mouse = Some(event.position);
                                            this.drag_start_item_pos = Some(img_pos);
                                            cx.notify();
                                            cx.stop_propagation();
                                        },
                                    ),
                                )
                                .flex()
                                .justify_between()
                                .items_center()
                                .px(px(6.0))
                                .child(div())
                                .child(
                                    div()
                                        .id(("delete-btn", id_num))
                                        .text_size(px(SMALL_ICON_FONT_SIZE))
                                        .line_height(gpui::relative(1.0))
                                        .text_color(rgb(0xff6b6b))
                                        .hover(|s| s.text_color(rgb(0xff0000)))
                                        .cursor_pointer()
                                        .child("×")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.edit_canvas_items.retain(|item| match item {
                                                    CanvasItem::Image(im) => im.id != delete_id,
                                                    _ => true,
                                                });
                                                cx.notify();
                                                cx.stop_propagation();
                                            }),
                                        ),
                                ),
                        );

                        // Image element
                        inner_block = inner_block
                            .child(img(source).w(px(img_item.width)).h(px(img_item.height)));

                        let wrapper = div()
                            .absolute()
                            .left(px(img_item.x + self.pan_x))
                            .top(px(img_item.y + self.pan_y))
                            .child(inner_block)
                            // Corner resize handle
                            .child(
                                div()
                                    .id(("resize-handle", id_num))
                                    .absolute()
                                    .right(px(-4.0))
                                    .bottom(px(-4.0))
                                    .w(px(10.0))
                                    .h(px(10.0))
                                    .bg(rgb(0x0078d4))
                                    .cursor_e_resize()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(
                                            move |this, event: &gpui::MouseDownEvent, _, cx| {
                                                this.resize_item_id = Some(resize_id.clone());
                                                this.resize_block_index = None;
                                                this.resize_start_mouse = Some(event.position);
                                                this.resize_start_size = Some(img_size);
                                                cx.notify();
                                                cx.stop_propagation();
                                            },
                                        ),
                                    ),
                            );

                        canvas_elements.push(wrapper.into_any_element());
                    }
                }
                CanvasItem::Mixed(m) => {
                    let textbox_width = if is_active {
                        let max_line_len = self
                            .edit_body
                            .lines()
                            .map(|line| line.chars().count())
                            .max()
                            .unwrap_or(0);
                        let line_text_w = max_line_len as f32 * 6.2;
                        let needed_width = line_text_w + 20.0;
                        let base_w = m.width.unwrap_or(250.0);
                        let desired_w = needed_width.max(base_w).max(250.0);

                        let sidebar_w = if self.is_sidebar_open { 220.0 } else { 44.0 };
                        let page_sidebar_w = 180.0;
                        let canvas_visible_w =
                            (self.window_w - sidebar_w - page_sidebar_w - 30.0).max(300.0);
                        let canvas_max_right = canvas_visible_w - self.pan_x;
                        let max_allowed_width = (canvas_max_right - m.x).max(150.0);
                        desired_w.min(max_allowed_width)
                    } else {
                        m.width.unwrap_or(250.0)
                    };
                    let drag_id_clone = drag_id.clone();
                    let m_pos = (m.x, m.y);
                    let blocks = m.blocks.clone();

                    let mut inner_block = div()
                        .flex()
                        .flex_col()
                        .w(px(textbox_width))
                        .bg(rgb(0x1e1e1e))
                        .border_1()
                        .border_color(if is_active {
                            rgb(0x0078d4)
                        } else {
                            rgb(0x3d3d3d)
                        })
                        .rounded(px(6.0))
                        .overflow_hidden();

                    // Drag Handle header (drags/deletes the whole combined box)
                    inner_block = inner_block.child(
                        div()
                            .id(("drag-header", id_num))
                            .h(px(12.0))
                            .bg(rgb(0x2d2d2d))
                            .cursor_move()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.drag_item_id = Some(drag_id_clone.clone());
                                    this.drag_start_mouse = Some(event.position);
                                    this.drag_start_item_pos = Some(m_pos);
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            )
                            .flex()
                            .justify_between()
                            .items_center()
                            .px(px(6.0))
                            .child(div())
                            .child(
                                div()
                                    .id(("delete-btn", id_num))
                                    .text_size(px(SMALL_ICON_FONT_SIZE))
                                    .line_height(gpui::relative(1.0))
                                    .text_color(rgb(0xff6b6b))
                                    .hover(|s| s.text_color(rgb(0xff0000)))
                                    .cursor_pointer()
                                    .child("×")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.edit_canvas_items.retain(|item| match item {
                                                CanvasItem::Mixed(mx) => mx.id != delete_id,
                                                _ => true,
                                            });
                                            if this.active_text_block_id == Some(delete_id.clone())
                                            {
                                                this.active_text_block_id = None;
                                                this.active_block_index = None;
                                                this.edit_body = String::new();
                                                this.reset_body_styles();
                                                this.edit_body_cursor = 0;
                                            }
                                            cx.notify();
                                            cx.stop_propagation();
                                        }),
                                    ),
                            ),
                    );

                    for (block_idx, block) in blocks.iter().enumerate() {
                        match block {
                            ContentBlock::Image { path, width, height } => {
                                if let Some(img_data) = self.decrypt_image(path) {
                                    let source = gpui::ImageSource::Image(Arc::new(img_data));
                                    let resize_id_clone = resize_id.clone();
                                    let (img_w, img_h) = (*width, *height);

                                    let image_wrapper = div()
                                        .relative()
                                        .child(img(source).w(px(img_w)).h(px(img_h)))
                                        .child(
                                            div()
                                                .id(("img-resize-handle", id_num.wrapping_add(block_idx)))
                                                .absolute()
                                                .right(px(-4.0))
                                                .bottom(px(-4.0))
                                                .w(px(10.0))
                                                .h(px(10.0))
                                                .bg(rgb(0x0078d4))
                                                .cursor_e_resize()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                                        this.resize_item_id = Some(resize_id_clone.clone());
                                                        this.resize_block_index = Some(block_idx);
                                                        this.resize_start_mouse = Some(event.position);
                                                        this.resize_start_size = Some((img_w, img_h));
                                                        cx.notify();
                                                        cx.stop_propagation();
                                                    }),
                                                ),
                                        );

                                    inner_block = inner_block.child(image_wrapper);
                                }
                            }
                            ContentBlock::Text {
                                text,
                                bold_spans,
                                italic_spans,
                                underline_spans,
                                strike_spans,
                            } => {
                                let is_seg_active =
                                    is_active && self.active_block_index == Some(block_idx);
                                let seg_id_num = id_num.wrapping_add(block_idx * 7 + 1);
                                let styles = TextStyleSpans {
                                    bold: bold_spans.clone(),
                                    italic: italic_spans.clone(),
                                    underline: underline_spans.clone(),
                                    strike: strike_spans.clone(),
                                };
                                let seg = self.render_text_segment(
                                    seg_id_num,
                                    item_id.clone(),
                                    Some(block_idx),
                                    text.clone(),
                                    styles,
                                    m_pos.0,
                                    m_pos.1,
                                    textbox_width,
                                    is_seg_active,
                                    is_body_focused,
                                    cx,
                                );
                                inner_block = inner_block.child(seg);
                            }
                        }
                    }

                    let mut wrapper = div()
                        .absolute()
                        .left(px(m_pos.0 + self.pan_x))
                        .top(px(m_pos.1 + self.pan_y))
                        .child(inner_block);

                    // Width resize handle for the whole box (wraps the text under the image)
                    if is_active {
                        wrapper = wrapper.child(
                            div()
                                .id(("resize-handle", id_num))
                                .absolute()
                                .right(px(-4.0))
                                .bottom(px(-4.0))
                                .w(px(10.0))
                                .h(px(10.0))
                                .bg(rgb(0x0078d4))
                                .cursor_e_resize()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                        this.resize_item_id = Some(resize_id.clone());
                                        this.resize_block_index = None;
                                        this.resize_start_mouse = Some(event.position);
                                        this.resize_start_size = Some((textbox_width, 100.0));
                                        cx.notify();
                                        cx.stop_propagation();
                                    }),
                                ),
                        );
                    }

                    canvas_elements.push(wrapper.into_any_element());
                }
            }
        }

        canvas_elements
    }
}
