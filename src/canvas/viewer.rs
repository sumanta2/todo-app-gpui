use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, Context, IntoElement, MouseButton, Window,
};
use std::sync::Arc;

use crate::app::NotesApp;
use crate::helpers::hash_str;
use crate::models::{load_canvas_items, CanvasItem, Note, NoteContent};
use crate::text_selection::{
    calculate_canvas_text_offset_full, calculate_line_text_offset_with_bold_and_font,
};

impl NotesApp {
    /// Renders the read-only canvas viewer for a note after editing is finished.
    ///
    /// This method paints the saved canvas text blocks and images exactly as they were last
    /// serialized, while preserving the current viewer selection state.
    pub(crate) fn render_canvas_viewer(
        &mut self,
        _note: &Note,
        content: &NoteContent,
        page_sidebar: AnyElement,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut viewer_elements = Vec::new();

        let active_page_items: Vec<CanvasItem> = {
            let mut items = Vec::new();
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter().find(|s| s.id == *sec_id) {
                    if let Some(ref page_id) = self.active_page_id {
                        if let Some(page) = section.pages.iter().find(|p| p.id == *page_id) {
                            items = load_canvas_items(&page.body);
                        }
                    }
                }
            }
            items
        };

        for item in &active_page_items {
            match item {
                //  RENDER CANVAS-ITEM TEXT FOR VIEW =========================================================================
                CanvasItem::Text(t) => {
                    let t_id = t.id.clone();
                    let is_text_active = self.viewer_active_text_block_id.as_deref() == Some(&t_id);
                    let text_content = t.text.clone();
                    let item_x = t.x;
                    let item_y = t.y;
                    let item_w = t.width.unwrap_or(250.0);

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
                    let bold_flags = crate::helpers::spans_to_bool_vec(&t.bold_spans, total_chars);

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

                                let before_prefix: String =
                                    line_chars[..loc_start].iter().collect();
                                let sel_content: String =
                                    line_chars[loc_start..loc_end].iter().collect();

                                let before_flags =
                                    &line_bold_flags[..loc_start.min(line_bold_flags.len())];
                                let before_runs = crate::helpers::split_text_into_styled_runs(
                                    &before_prefix,
                                    before_flags,
                                );
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

                                let sel_flags = &line_bold_flags[loc_start
                                    .min(line_bold_flags.len())
                                    ..loc_end.min(line_bold_flags.len())];
                                let sel_runs = crate::helpers::split_text_into_styled_runs(
                                    &sel_content,
                                    sel_flags,
                                );
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

                        // Render full line text as a solid contiguous styled element
                        let runs =
                            crate::helpers::split_text_into_styled_runs(line, &line_bold_flags);
                        let mut line_elements = Vec::new();
                        if runs.is_empty() {
                            line_elements.push(
                                div()
                                    .text_color(rgb(0xd4d4d4))
                                    .child("\u{00A0}")
                                    .into_any_element(),
                            );
                        } else {
                            for run in &runs {
                                let disp = run.text.replace(' ', "\u{00A0}");
                                let el = div()
                                    .text_color(if run.is_bold {
                                        rgb(0xffffff)
                                    } else {
                                        rgb(0xd4d4d4)
                                    })
                                    .font_weight(if run.is_bold {
                                        gpui::FontWeight::BOLD
                                    } else {
                                        gpui::FontWeight::NORMAL
                                    })
                                    .child(disp);
                                line_elements.push(el.into_any_element());
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
                            let local_cursor =
                                cursor_idx.saturating_sub(line_global_start).min(line_len);
                            let line_chars: Vec<char> = line.chars().collect();
                            let before_prefix: String = line_chars[..local_cursor].iter().collect();
                            let before_flags =
                                &line_bold_flags[..local_cursor.min(line_bold_flags.len())];
                            let prefix_runs = crate::helpers::split_text_into_styled_runs(
                                &before_prefix,
                                before_flags,
                            );

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
                        let t_id_for_line = t_id.clone();
                        let line_str_owned = line.to_string();
                        let line_flags_for_click = line_bold_flags.clone();

                        let row = row
                            .id(("viewer-line-row", hash_str(&t_id).wrapping_add(line_idx)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(
                                    move |this, event: &gpui::MouseDownEvent, window, cx| {
                                        this.focus_handle.focus(window, cx);
                                        this.is_panning = false;
                                        this.pan_start_mouse = None;
                                        this.pan_start_val = None;
                                        this.viewer_active_text_block_id =
                                            Some(t_id_for_line.clone());

                                        let sidebar_w =
                                            if this.is_sidebar_open { 220.0 } else { 44.0 };
                                        let rel_x = (event.position.x.as_f32()
                                            - sidebar_w
                                            - this.pan_x
                                            - item_x
                                            - 5.0)
                                            .max(0.0);
                                        let local_idx =
                                        calculate_line_text_offset_with_bold_and_font(
                                            rel_x,
                                            &line_str_owned,
                                            Some(&line_flags_for_click),
                                            this.canvas_body_font_size,
                                            this.font_type(),
                                        );
                                        let click_idx =
                                            (line_global_start + local_idx).min(total_chars);
                                        this.viewer_text_cursor = click_idx;
                                        this.viewer_text_anchor = Some(click_idx);
                                        this.is_selecting_viewer_text = true;
                                        this.cursor_visible = true;
                                        cx.notify();
                                        cx.stop_propagation();
                                    },
                                ),
                            );

                        line_rows.push(row.into_any_element());
                        global_offset += line_len + 1;
                    }

                    let t_id_down = t_id.clone();
                    let text_for_down = text_content.clone();
                    let bold_flags_down = bold_flags.clone();
                    let bold_flags_move = bold_flags.clone();

                    let text_el = div()
                        .absolute()
                        .left(px(item_x + self.pan_x))
                        .top(px(item_y + self.pan_y))
                        .w(px(item_w))
                        .p(px(5.0))
                        .text_size(px(self.canvas_body_font_size))
                        .text_color(rgb(0xd4d4d4))
                        .cursor_text()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                                this.focus_handle.focus(window, cx);
                                this.is_panning = false;
                                this.pan_start_mouse = None;
                                this.pan_start_val = None;
                                this.viewer_active_text_block_id = Some(t_id_down.clone());

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
                        .on_mouse_move(cx.listener(
                            move |this, event: &gpui::MouseMoveEvent, _, cx| {
                                if this.is_selecting_viewer_text {
                                    if let Some(ref active_id) = this.viewer_active_text_block_id {
                                        if active_id == &t_id {
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
                            },
                        ))
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
                        .children(line_rows);

                    viewer_elements.push(text_el.into_any_element());
                }

                //  RENDER CANVAS-ITEM IMAGES FOR VIEW =========================================================================
                CanvasItem::Image(img_item) => {
                    if let Some(img_data) = self.decrypt_image(&img_item.path) {
                        let source = gpui::ImageSource::Image(Arc::new(img_data));
                        let image_el = div()
                            .absolute()
                            .left(px(img_item.x + self.pan_x))
                            .top(px(img_item.y + self.pan_y))
                            .w(px(img_item.width))
                            .h(px(img_item.height))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.is_panning = true;
                                    this.pan_start_mouse = Some(event.position);
                                    this.pan_start_val = Some((this.pan_x, this.pan_y));
                                    cx.notify();
                                }),
                            )
                            .on_mouse_move(cx.listener(
                                |this, event: &gpui::MouseMoveEvent, _, cx| {
                                    if this.is_panning {
                                        if let (Some(start_mouse), Some(start_pan)) =
                                            (this.pan_start_mouse, this.pan_start_val)
                                        {
                                            let dx =
                                                event.position.x.as_f32() - start_mouse.x.as_f32();
                                            let dy =
                                                event.position.y.as_f32() - start_mouse.y.as_f32();
                                            this.pan_x = (start_pan.0 + dx).min(0.0);
                                            this.pan_y = (start_pan.1 + dy).min(0.0);
                                            cx.notify();
                                        }
                                    }
                                },
                            ))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _event: &gpui::MouseUpEvent, _, cx| {
                                    this.is_panning = false;
                                    this.pan_start_mouse = None;
                                    this.pan_start_val = None;
                                    cx.notify();
                                }),
                            )
                            .child(
                                img(source)
                                    .w(px(img_item.width))
                                    .h(px(img_item.height))
                                    .rounded(px(6.0))
                                    .border_1()
                                    .border_color(rgb(0x3d3d3d)),
                            );
                        viewer_elements.push(image_el.into_any_element());
                    }
                }
            }
        }

        let page_name = {
            let mut name = "Untitled Page".to_string();
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter().find(|s| s.id == *sec_id) {
                    if let Some(ref page_id) = self.active_page_id {
                        if let Some(page) = section.pages.iter().find(|p| p.id == *page_id) {
                            name = page.name.clone();
                        }
                    }
                }
            }
            name
        };

        let active_items_for_drag = active_page_items.clone();

        let canvas_container = div()
            .id("note-viewer-canvas")
            .flex_1()
            .relative()
            .bg(rgb(0x141414))
            .border_1()
            .border_color(rgb(0x3d3d3d))
            .rounded(px(6.0))
            .overflow_hidden()
            .cursor_default()
            .on_scroll_wheel(cx.listener(|this, event: &gpui::ScrollWheelEvent, _, cx| {
                match event.delta {
                    gpui::ScrollDelta::Pixels(point) => {
                        this.pan_x = (this.pan_x + point.x.as_f32()).min(0.0);
                        this.pan_y = (this.pan_y + point.y.as_f32()).min(0.0);
                        cx.notify();
                    }
                    gpui::ScrollDelta::Lines(point) => {
                        this.pan_x = (this.pan_x + point.x * 20.0).min(0.0);
                        this.pan_y = (this.pan_y + point.y * 20.0).min(0.0);
                        cx.notify();
                    }
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                    this.viewer_text_anchor = None;
                    this.is_selecting_viewer_text = false;
                    this.is_panning = true;
                    this.pan_start_mouse = Some(event.position);
                    this.pan_start_val = Some((this.pan_x, this.pan_y));
                    cx.notify();
                }),
            )
            .on_mouse_move(
                cx.listener(move |this, event: &gpui::MouseMoveEvent, _, cx| {
                    if this.is_selecting_viewer_text {
                        if let Some(ref active_id) = this.viewer_active_text_block_id.clone() {
                            if let Some(CanvasItem::Text(tx)) =
                                active_items_for_drag.iter().find(|i| match i {
                                    CanvasItem::Text(t) => t.id == *active_id,
                                    _ => false,
                                })
                            {
                                let item_x = tx.x;
                                let item_y = tx.y;
                                let item_w = tx.width.unwrap_or(250.0);
                                let tx_total_chars = tx.text.chars().count();
                                let bold_flags = crate::helpers::spans_to_bool_vec(
                                    &tx.bold_spans,
                                    tx_total_chars,
                                );
                                let drag_idx = calculate_canvas_text_offset_full(
                                    event.position,
                                    this.is_sidebar_open,
                                    this.pan_x,
                                    this.pan_y,
                                    item_x,
                                    item_y,
                                    this.canvas_top_y,
                                    &tx.text,
                                    Some(&bold_flags),
                                    5.0,
                                    5.0,
                                    item_w,
                                    this.canvas_body_font_size,
                                    this.font_type(),
                                );
                                this.viewer_text_cursor = drag_idx;
                                cx.notify();
                            }
                        }
                    } else if this.is_panning {
                        if let (Some(start_mouse), Some(start_pan)) =
                            (this.pan_start_mouse, this.pan_start_val)
                        {
                            let dx = event.position.x.as_f32() - start_mouse.x.as_f32();
                            let dy = event.position.y.as_f32() - start_mouse.y.as_f32();
                            this.pan_x = (start_pan.0 + dx).min(0.0);
                            this.pan_y = (start_pan.1 + dy).min(0.0);
                            cx.notify();
                        }
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseUpEvent, _, cx| {
                    if this.is_selecting_viewer_text {
                        if this.viewer_text_anchor == Some(this.viewer_text_cursor) {
                            this.viewer_text_anchor = None;
                        }
                        this.is_selecting_viewer_text = false;
                        cx.notify();
                    }
                    if this.is_panning {
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        cx.notify();
                    }
                }),
            )
            .child(
                div()
                    .absolute()
                    .top(px(12.0))
                    .left(px(12.0))
                    .text_size(px(11.0))
                    .text_color(rgb(0x606060))
                    .child("💡 Drag the background to pan the canvas"),
            )
            .children(viewer_elements);

        let section_tabs = {
            let mut tabs = Vec::new();
            for sec in &content.sections {
                let sec_id = sec.id.clone();
                let is_active = Some(&sec_id) == self.active_section_id.as_ref();
                let click_id = sec_id.clone();

                let mut tab_el = div()
                    .id(("sec-tab", hash_str(&sec_id)))
                    .px(px(6.0))
                    .py(px(2.0))
                    .text_size(px(11.0))
                    .bg(if is_active {
                        rgb(0x1e1e1e)
                    } else {
                        rgb(0x2d2d2d)
                    });
                if is_active {
                    tab_el = tab_el.border_t_2().border_color(rgb(0x0078d4));
                }
                tab_el = tab_el
                    .text_color(if is_active {
                        rgb(0xffffff)
                    } else {
                        rgb(0x808080)
                    })
                    .font_weight(if is_active {
                        gpui::FontWeight::BOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.switch_to_section(click_id.clone(), cx);
                    }))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(sec.name.clone());

                tabs.push(tab_el.into_any_element());
            }

            let left_side = div().flex().flex_row().gap(px(4.0)).children(tabs);

            let right_side = div()
                .flex()
                .gap(px(6.0))
                .child(
                    div()
                        .id("edit-note-header-btn")
                        .px(px(5.0))
                        .py(px(2.0))
                        .bg(rgb(0x2d2d2d))
                        .hover(|s| s.bg(rgb(0x3d3d3d)))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(rgb(0xd4d4d4))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.start_edit(cx);
                        }))
                        .child("Edit Note"),
                )
                .child(
                    div()
                        .id("delete-note-header-btn")
                        .px(px(5.0))
                        .py(px(2.0))
                        .bg(rgb(0x5a2a2a))
                        .hover(|s| s.bg(rgb(0x6a3a3a)))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(rgb(0xff6b6b))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(ref id) = this.selected_note_id {
                                this.delete_note(id.clone(), cx);
                            }
                        }))
                        .child("Delete Note"),
                );

            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .child(left_side)
                .child(right_side)
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(0x1e1e1e))
            .gap(px(6.0))
            .child(section_tabs)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .h_full()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .h_full()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .px(px(5.0))
                                    .py(px(2.0))
                                    .border_b_1()
                                    .border_color(rgb(0x2d2d2d))
                                    .child(
                                        div()
                                            .text_size(px(20.0))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0xffffff))
                                            .child(page_name),
                                    ),
                            )
                            .child(canvas_container),
                    )
                    .child(page_sidebar),
            )
    }
}
