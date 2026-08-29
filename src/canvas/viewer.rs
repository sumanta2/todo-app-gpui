use gpui::{div, img, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window};
use std::sync::Arc;

use crate::app::NotesApp;
use crate::helpers::hash_str;
use crate::models::{load_canvas_items, CanvasItem, Note, NoteContent};
use crate::text_selection::{
    calculate_canvas_drag_offset_with_header, calculate_canvas_text_offset_with_header,
};

impl NotesApp {
    pub(crate) fn render_canvas_viewer(
        &mut self,
        _note: &Note,
        content: &NoteContent,
        page_sidebar: AnyElement,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut viewer_elements = Vec::new();

        let active_page_items = {
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

                    let logical_lines: Vec<&str> = text_content.split('\n').collect();
                    let mut line_rows: Vec<AnyElement> = Vec::new();
                    let mut global_offset = 0;

                    for line in logical_lines {
                        let line_len = line.chars().count();
                        let line_global_start = global_offset;
                        let line_global_end = global_offset + line_len;

                        let mut row = div().flex().flex_row().items_center().min_h(px(20.0));

                        if has_sel {
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

                                let before_str: String = line_chars[..loc_start].iter().collect();
                                let sel_str: String =
                                    line_chars[loc_start..loc_end].iter().collect();
                                let after_str: String = line_chars[loc_end..].iter().collect();

                                if !before_str.is_empty() {
                                    row = row.child(
                                        div()
                                            .flex_shrink_0()
                                            .text_color(rgb(0xd4d4d4))
                                            .child(before_str.replace(' ', "\u{00A0}")),
                                    );
                                }
                                 if !sel_str.is_empty() {
                                    row = row.child(
                                        div()
                                            .flex_shrink_0()
                                            .text_color(rgb(0xffffff))
                                            .bg(rgb(0x0078d4))
                                            .rounded(px(2.0))
                                            .child(sel_str.replace(' ', "\u{00A0}")),
                                    );
                                }
                                if !after_str.is_empty() {
                                    row = row.child(
                                        div()
                                            .flex_shrink_0()
                                            .text_color(rgb(0xd4d4d4))
                                            .child(after_str.replace(' ', "\u{00A0}")),
                                    );
                                }
                            } else {
                                if line.is_empty() && sel_start <= line_global_start && sel_end > line_global_start {
                                    row = row.child(
                                        div()
                                            .w(px(6.0))
                                            .h(px(14.0))
                                            .bg(rgb(0x0078d4))
                                            .rounded(px(2.0)),
                                    );
                                } else if !line.is_empty() {
                                    row = row.child(
                                        div()
                                            .text_color(rgb(0xd4d4d4))
                                            .child(line.replace(' ', "\u{00A0}")),
                                    );
                                }
                            }
                        } else {
                            if !line.is_empty() {
                                row = row.child(
                                    div()
                                        .text_color(rgb(0xd4d4d4))
                                        .child(line.replace(' ', "\u{00A0}")),
                                );
                            }
                        }

                        line_rows.push(row.into_any_element());
                        global_offset += line_len + 1;
                    }

                    let t_id_down = t_id.clone();
                    let text_for_down = text_content.clone();

                    let text_el = div()
                        .absolute()
                        .left(px(item_x + self.pan_x))
                        .top(px(item_y + self.pan_y))
                        .w(px(item_w))
                        .p(px(5.0))
                        .text_size(px(12.0))
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

                                let click_idx = calculate_canvas_text_offset_with_header(
                                    event.position,
                                    this.is_sidebar_open,
                                    this.pan_x,
                                    this.pan_y,
                                    item_x,
                                    item_y,
                                    this.canvas_top_y,
                                    &text_for_down,
                                    6.0,
                                    6.0,
                                    item_w,
                                );
                                this.viewer_text_cursor = click_idx;
                                this.viewer_text_anchor = Some(click_idx);
                                this.is_selecting_viewer_text = true;
                                cx.notify();
                                cx.stop_propagation();
                            }),
                        )
                        .on_mouse_move(cx.listener(
                            move |this, event: &gpui::MouseMoveEvent, _, cx| {
                                if this.is_selecting_viewer_text {
                                    if let Some(ref active_id) = this.viewer_active_text_block_id {
                                        if active_id == &t_id {
                                            let anchor = this
                                                .viewer_text_anchor
                                                .unwrap_or(this.viewer_text_cursor);
                                            let drag_idx = calculate_canvas_drag_offset_with_header(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                item_x,
                                                item_y,
                                                this.canvas_top_y,
                                                &text_content,
                                                anchor,
                                                6.0,
                                                6.0,
                                                item_w,
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
                                let anchor =
                                    this.viewer_text_anchor.unwrap_or(this.viewer_text_cursor);
                                let drag_idx = calculate_canvas_drag_offset_with_header(
                                    event.position,
                                    this.is_sidebar_open,
                                    this.pan_x,
                                    this.pan_y,
                                    item_x,
                                    item_y,
                                    this.canvas_top_y,
                                    &tx.text,
                                    anchor,
                                    6.0,
                                    6.0,
                                    item_w,
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
