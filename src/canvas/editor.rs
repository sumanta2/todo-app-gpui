use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::models::{ActiveField, CanvasItem, Note, NoteContent, TextItem};
use crate::text_selection::{calculate_canvas_drag_offset, calculate_line_text_offset};

impl NotesApp {
    /// Renders the editable note canvas and its surrounding controls.
    ///
    /// This function assembles the section tabs, the editable heading, the canvas body, and
    /// the page sidebar into one notebook editor surface.
    pub(crate) fn render_canvas_editor(
        &mut self,
        _note: &Note,
        content: &NoteContent,
        page_sidebar: AnyElement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.window_w = window.bounds().size.width.as_f32();

        let is_section_name_focused =
            self.focus_handle.is_focused(window) && self.active_field == ActiveField::SectionName;
        let is_heading_focused =
            self.focus_handle.is_focused(window) && self.active_field == ActiveField::Heading;
        let is_body_focused =
            self.focus_handle.is_focused(window) && self.active_field == ActiveField::Body;

        let section_tabs = self.build_section_tabs(content, is_section_name_focused, cx);
        let heading_content = self.build_heading_editor(is_heading_focused, cx);
        let canvas_elements = self.render_canvas_elements(is_body_focused, cx);

        let canvas_container = div()
            .id("note-body-canvas")
            .flex_1()
            .relative()
            .bg(rgb(0x141414))
            .border_1()
            .border_color(if is_body_focused {
                rgb(0x0078d4)
            } else {
                rgb(0x3d3d3d)
            })
            .rounded(px(6.0))
            .overflow_hidden()
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
                    this.is_panning = true;
                    this.pan_has_dragged = false;
                    this.pan_start_mouse = Some(event.position);
                    this.pan_start_val = Some((this.pan_x, this.pan_y));
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                let mut changed = false;
                if let Some(ref item_id) = this.drag_item_id {
                    if let (Some(start_mouse), Some(start_pos)) =
                        (this.drag_start_mouse, this.drag_start_item_pos)
                    {
                        let dx = event.position.x.as_f32() - start_mouse.x.as_f32();
                        let dy = event.position.y.as_f32() - start_mouse.y.as_f32();
                        let new_x = (start_pos.0 + dx).max(0.0);
                        let new_y = (start_pos.1 + dy).max(0.0);

                        if let Some(item) = this.edit_canvas_items.iter_mut().find(|i| match i {
                            CanvasItem::Text(t) => t.id == *item_id,
                            CanvasItem::Image(img) => img.id == *item_id,
                        }) {
                            match item {
                                CanvasItem::Text(t) => {
                                    t.x = new_x;
                                    t.y = new_y;
                                }
                                CanvasItem::Image(img) => {
                                    img.x = new_x;
                                    img.y = new_y;
                                }
                            }
                            changed = true;
                        }
                    }
                } else if let Some(ref resize_id) = this.resize_item_id {
                    if let (Some(start_mouse), Some(start_size)) =
                        (this.resize_start_mouse, this.resize_start_size)
                    {
                        let dx = event.position.x.as_f32() - start_mouse.x.as_f32();
                        let dy = event.position.y.as_f32() - start_mouse.y.as_f32();
                        let new_width = (start_size.0 + dx).max(100.0);
                        let new_height = (start_size.1 + dy).max(50.0);

                        if let Some(item) = this.edit_canvas_items.iter_mut().find(|i| match i {
                            CanvasItem::Text(t) => t.id == *resize_id,
                            CanvasItem::Image(img) => img.id == *resize_id,
                        }) {
                            match item {
                                CanvasItem::Text(t) => {
                                    t.width = Some(new_width);
                                }
                                CanvasItem::Image(img) => {
                                    img.width = new_width;
                                    img.height = new_height;
                                }
                            }
                            changed = true;
                        }
                    }
                } else if this.is_selecting_body {
                    if let Some(ref active_id) = this.active_text_block_id.clone() {
                        if let Some(CanvasItem::Text(tx)) = this.edit_canvas_items.iter().find(|i| match i {
                            CanvasItem::Text(t) => t.id == *active_id,
                            _ => false,
                        }) {
                            let item_x = tx.x;
                            let item_y = tx.y;
                            let item_w = tx.width.unwrap_or(250.0);
                            let anchor = this.edit_body_anchor.unwrap_or(this.edit_body_cursor);
                            let drag_idx = calculate_canvas_drag_offset(
                                event.position,
                                this.is_sidebar_open,
                                this.pan_x,
                                this.pan_y,
                                item_x,
                                item_y,
                                this.canvas_top_y,
                                &this.edit_body,
                                anchor,
                                item_w,
                            );
                            this.edit_body_cursor = drag_idx;
                            changed = true;
                        }
                    }
                } else if this.is_selecting_heading {
                    let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                    let rel_x = (event.position.x.as_f32() - sidebar_w - 10.0).max(0.0);
                    let drag_idx = calculate_line_text_offset(rel_x, &this.edit_heading, 20.0);
                    this.edit_heading_cursor = drag_idx;
                    changed = true;
                } else if this.is_selecting_section_name {
                    let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                    let rel_x = (event.position.x.as_f32() - sidebar_w - 20.0).max(0.0);
                    let drag_idx = calculate_line_text_offset(rel_x, &this.edit_section_name, 11.0);
                    this.edit_section_name_cursor = drag_idx;
                    changed = true;
                } else if this.is_panning {
                    if let (Some(start_mouse), Some(start_pan)) =
                        (this.pan_start_mouse, this.pan_start_val)
                    {
                        let dx = event.position.x.as_f32() - start_mouse.x.as_f32();
                        let dy = event.position.y.as_f32() - start_mouse.y.as_f32();
                        if dx.abs() > 3.0 || dy.abs() > 3.0 {
                            this.pan_has_dragged = true;
                        }
                        this.pan_x = (start_pan.0 + dx).min(0.0);
                        this.pan_y = (start_pan.1 + dy).min(0.0);
                        changed = true;
                    }
                }

                if changed {
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseUpEvent, _, cx| {
                    if this.is_selecting_body {
                        if this.edit_body_anchor == Some(this.edit_body_cursor) {
                            this.edit_body_anchor = None;
                        }
                        this.is_selecting_body = false;
                        cx.notify();
                    }
                    if this.is_selecting_heading {
                        if this.edit_heading_anchor == Some(this.edit_heading_cursor) {
                            this.edit_heading_anchor = None;
                        }
                        this.is_selecting_heading = false;
                        cx.notify();
                    }
                    if this.is_selecting_section_name {
                        if this.edit_section_name_anchor == Some(this.edit_section_name_cursor) {
                            this.edit_section_name_anchor = None;
                        }
                        this.is_selecting_section_name = false;
                        cx.notify();
                    }
                    if this.is_panning {
                        if !this.pan_has_dragged {
                            if let Some(start_mouse) = this.pan_start_mouse {
                                let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                                let click_x =
                                    (start_mouse.x.as_f32() - sidebar_w - this.pan_x).max(0.0);
                                let click_y =
                                    (start_mouse.y.as_f32() - this.canvas_top_y - this.pan_y)
                                        .max(0.0);

                                this.sync_active_text_block();

                                if let Some(ref active_id) = this.active_text_block_id {
                                    let is_empty = this.edit_canvas_items.iter().any(|item| {
                                        if let CanvasItem::Text(t) = item {
                                            t.id == *active_id && t.text.trim().is_empty()
                                        } else {
                                            false
                                        }
                                    });
                                    if is_empty {
                                        this.edit_canvas_items.retain(|item| {
                                            if let CanvasItem::Text(t) = item {
                                                t.id != *active_id
                                            } else {
                                                true
                                            }
                                        });
                                    }
                                }

                                let new_id = chrono::Local::now().timestamp_millis().to_string();
                                let new_text_item = TextItem {
                                    id: new_id.clone(),
                                    x: click_x,
                                    y: click_y,
                                    text: String::new(),
                                    width: Some(250.0),
                                    bold_spans: Vec::new(),
                                };
                                this.edit_canvas_items.push(CanvasItem::Text(new_text_item));
                                this.active_text_block_id = Some(new_id);
                                this.edit_body = String::new();
                                this.edit_body_bold = Vec::new();
                                this.edit_body_cursor = 0;
                                this.edit_body_anchor = None;
                                this.active_field = ActiveField::Body;
                            }
                        }
                    }
                    this.drag_item_id = None;
                    this.drag_start_mouse = None;
                    this.drag_start_item_pos = None;
                    this.resize_item_id = None;
                    this.resize_start_mouse = None;
                    this.resize_start_size = None;
                    this.is_panning = false;
                    this.pan_start_mouse = None;
                    this.pan_start_val = None;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .absolute()
                    .top(px(12.0))
                    .left(px(12.0))
                    .text_size(px(11.0))
                    .text_color(rgb(0x606060))
                    .child("💡 Click canvas to type | Ctrl+B to bold | Drag headers to move | Ctrl+V to paste | Drag background to pan"),
            )
            .children(canvas_elements);

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
                                    .border_color(if is_heading_focused {
                                        rgb(0x0078d4)
                                    } else {
                                        rgb(0x2d2d2d)
                                    })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.active_field = ActiveField::Heading;
                                            this.focus_handle.focus(window, cx);
                                            this.edit_heading_cursor =
                                                this.edit_heading.chars().count();
                                            this.edit_heading_anchor = None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(heading_content),
                            )
                            .child(canvas_container),
                    )
                    .child(page_sidebar),
            )
    }
}
