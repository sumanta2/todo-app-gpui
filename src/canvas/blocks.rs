//! Text, image, and mixed blocks on the editable canvas, including drag, resize, and delete.

use gpui::{div, img, prelude::*, px, rgb, AnyElement, Context, MouseButton};
use std::sync::Arc;

use crate::app::NotesApp;
use crate::constants::typography::SMALL_ICON_FONT_SIZE;
use crate::models::{ActiveField, CanvasItem, ContentBlock, TextStyleSpans};

impl NotesApp {
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
        if self.active_field == ActiveField::Body {
            self.ensure_body_hit_cache();
        }

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
                        let line_text_w = crate::text::selection::max_text_advance(
                            &self.edit_body,
                            Some(&self.edit_body_bold),
                            self.canvas_body_font_size,
                            self.font_type(),
                        );
                        let needed_width = line_text_w + 24.0;
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

                    let show_chrome = self.text_box_chrome_visible(&item_id, is_active);
                    let mut inner_block = div()
                        .flex()
                        .flex_col()
                        .w(px(textbox_width))
                        .rounded(px(6.0));
                    if show_chrome {
                        inner_block = inner_block
                            .bg(rgb(0x1e1e1e))
                            .border_1()
                            .border_color(rgb(0x3d3d3d))
                            .overflow_hidden();
                    }

                    // Drag handle. A blank slot keeps the text in place when the bar is hidden.
                    let header = div()
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
                            );
                    if show_chrome {
                        inner_block = inner_block.child(header);
                    } else {
                        inner_block = inner_block.child(div().h(px(12.0)));
                    }

                    let segment = self.render_text_segment(
                        id_num, item_id.clone(), None, text, styles, t_pos.0, t_pos.1,
                        textbox_width, is_active, is_body_focused, cx,
                    );
                    inner_block = inner_block.child(segment);

                    // corner resize handle container
                    let hover_id = item_id.clone();
                    let mut wrapper = div()
                        .id(("canvas-text", id_num))
                        .absolute()
                        .left(px(t_pos.0 + self.pan_x))
                        .top(px(t_pos.1 + self.pan_y))
                        .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                            this.set_canvas_item_hover(&hover_id, *hovered, cx);
                        }))
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
                                .bg(rgb(0x3d3d3d))
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
                        let line_text_w = crate::text::selection::max_text_advance(
                            &self.edit_body,
                            Some(&self.edit_body_bold),
                            self.canvas_body_font_size,
                            self.font_type(),
                        );
                        let needed_width = line_text_w + 24.0;
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

                    let show_chrome = self.text_box_chrome_visible(&item_id, is_active);
                    let mut inner_block = div()
                        .flex()
                        .flex_col()
                        .w(px(textbox_width))
                        .rounded(px(6.0));
                    if show_chrome {
                        inner_block = inner_block
                            .bg(rgb(0x1e1e1e))
                            .border_1()
                            .border_color(rgb(0x3d3d3d))
                            .overflow_hidden();
                    }

                    // Drag handle. A blank slot keeps the content in place when the bar is hidden.
                    let header = div()
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
                            );
                    if show_chrome {
                        inner_block = inner_block.child(header);
                    } else {
                        inner_block = inner_block.child(div().h(px(12.0)));
                    }

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
                                                .bg(rgb(0x3d3d3d))
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

                    let hover_id = item_id.clone();
                    let mut wrapper = div()
                        .id(("canvas-mixed", id_num))
                        .absolute()
                        .left(px(m_pos.0 + self.pan_x))
                        .top(px(m_pos.1 + self.pan_y))
                        .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                            this.set_canvas_item_hover(&hover_id, *hovered, cx);
                        }))
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
                                .bg(rgb(0x3d3d3d))
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

    /// Header and border are visible while the box is hovered, active, or being dragged.
    fn text_box_chrome_visible(&self, item_id: &str, is_active: bool) -> bool {
        is_active
            || self.hovered_canvas_item_id.as_deref() == Some(item_id)
            || self.drag_item_id.as_deref() == Some(item_id)
            || self.resize_item_id.as_deref() == Some(item_id)
    }

    fn set_canvas_item_hover(&mut self, item_id: &str, hovered: bool, cx: &mut Context<Self>) {
        if hovered {
            if self.hovered_canvas_item_id.as_deref() != Some(item_id) {
                self.hovered_canvas_item_id = Some(item_id.to_string());
                cx.notify();
            }
        } else if self.hovered_canvas_item_id.as_deref() == Some(item_id) {
            self.hovered_canvas_item_id = None;
            cx.notify();
        }
    }
}
