//! Text, image, and mixed blocks on the editable canvas, including drag, resize, and delete.

use gpui::{div, img, prelude::*, px, rgb, AnyElement, Context, MouseButton};

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
        let image_paths: Vec<String> = self
            .edit_canvas_items
            .iter()
            .flat_map(|item| match item {
                crate::models::CanvasItem::Image(image) => vec![image.path.clone()],
                crate::models::CanvasItem::Mixed(mixed) => mixed
                    .blocks
                    .iter()
                    .filter_map(|block| match block {
                        crate::models::ContentBlock::Image { path, .. } => Some(path.clone()),
                        _ => None,
                    })
                    .collect(),
                crate::models::CanvasItem::Text(_) => Vec::new(),
            })
            .collect();
        self.prefetch_image_paths(&image_paths);
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
                        self.fitted_text_box_width(t.width.unwrap_or(250.0), t.x)
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
                        font_runs: t.font_runs.clone(),
                        line_layouts: t.line_layouts.clone(),
                    };

                    let show_chrome = self.text_box_chrome_visible(&item_id, is_active);
                    let mut inner_block = div().flex().flex_col().w(px(self.scaled(textbox_width)));
                    if show_chrome {
                        inner_block = inner_block
                            .bg(rgb(crate::constants::colors::note_page()))
                            .border_1()
                            .border_color(rgb(crate::constants::colors::note_chrome_border()))
                            .overflow_hidden();
                    }

                    // Drag handle. A blank slot keeps the text in place when the bar is hidden.
                    let header = div()
                        .id(("drag-header", id_num))
                        .h(px(12.0))
                        .bg(rgb(crate::constants::colors::note_chrome_bar()))
                        .cursor_move()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                this.begin_canvas_gesture();
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
                                        this.record_edit(crate::app::history::EditKind::Canvas);
                                        this.edit_canvas_items.retain(|item| match item {
                                            CanvasItem::Text(tx) => tx.id != delete_id,
                                            _ => true,
                                        });
                                        if this.active_text_block_id == Some(delete_id.clone()) {
                                            this.active_text_block_id = None;
                                            this.active_block_index = None;
                                            this.edit_body = String::new();
                                            this.reset_body_styles();
                                            this.edit_body_cursor = 0;
                                        }
                                        this.schedule_autosave(cx);
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
                        id_num,
                        item_id.clone(),
                        None,
                        text,
                        styles,
                        t_pos.0,
                        t_pos.1,
                        textbox_width,
                        is_active,
                        is_body_focused,
                        cx,
                    );
                    inner_block = inner_block.child(segment);

                    // corner resize handle container
                    let hover_id = item_id.clone();
                    let mut wrapper = div()
                        .id(("canvas-text", id_num))
                        .absolute()
                        .left(px(self.place_x(t_pos.0)))
                        .top(px(self.place_y(t_pos.1)))
                        .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                            this.set_canvas_item_hover(&hover_id, *hovered, cx);
                        }))
                        .child(inner_block);

                    // Right edge is always hittable, so an unselected box still shows its border
                    // while the pointer is there and while the box is being resized.
                    let edge_hover_id = item_id.clone();
                    wrapper = wrapper.child(
                        div()
                            .id(("resize-handle", id_num))
                            .absolute()
                            .right(px(-3.0))
                            .top(px(0.0))
                            .bottom(px(0.0))
                            .w(px(8.0))
                            .cursor_col_resize()
                            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                this.set_canvas_edge_hover(&edge_hover_id, *hovered, cx);
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.begin_canvas_gesture();
                                    this.resize_item_id = Some(resize_id.clone());
                                    this.resize_block_index = None;
                                    this.resize_start_mouse = Some(event.position);
                                    this.resize_start_size = Some((textbox_width, 100.0));
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            ),
                    );

                    canvas_elements.push(wrapper.into_any_element());
                }
                CanvasItem::Image(img_item) => {
                    if let Some(img_data) = self.canvas_image(&img_item.path) {
                        let source = gpui::ImageSource::Image(img_data);
                        let drag_id_clone = drag_id.clone();
                        let img_pos = (img_item.x, img_item.y);
                        let img_size = (img_item.width, img_item.height);

                        let mut inner_block = div()
                            .flex()
                            .flex_col()
                            .w(px(self.scaled(img_item.width)))
                            .h(px(self.scaled(img_item.height) + 10.0))
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
                                            this.begin_canvas_gesture();
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
                                                this.record_edit(crate::app::history::EditKind::Canvas);
                                                this.edit_canvas_items.retain(|item| match item {
                                                    CanvasItem::Image(im) => im.id != delete_id,
                                                    _ => true,
                                                });
                                                this.schedule_autosave(cx);
                                                cx.notify();
                                                cx.stop_propagation();
                                            }),
                                        ),
                                ),
                        );

                        // Image element
                        inner_block = inner_block.child(
                            img(source)
                                .w(px(self.scaled(img_item.width)))
                                .h(px(self.scaled(img_item.height))),
                        );

                        let wrapper = div()
                            .absolute()
                            .left(px(self.place_x(img_item.x)))
                            .top(px(self.place_y(img_item.y)))
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
                                                this.begin_canvas_gesture();
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
                        self.fitted_text_box_width(m.width.unwrap_or(250.0), m.x)
                    } else {
                        m.width.unwrap_or(250.0)
                    };
                    let drag_id_clone = drag_id.clone();
                    let m_pos = (m.x, m.y);
                    let blocks = m.blocks.clone();

                    let show_chrome = self.text_box_chrome_visible(&item_id, is_active);
                    let image_frame_open = self
                        .selected_inline_image
                        .as_ref()
                        .is_some_and(|(id, _)| id == &item_id);
                    let mut inner_block = div().flex().flex_col().w(px(self.scaled(textbox_width)));
                    if show_chrome {
                        inner_block = inner_block
                            .bg(rgb(crate::constants::colors::note_page()))
                            .border_1()
                            .border_color(rgb(crate::constants::colors::note_chrome_border()));
                        if !image_frame_open {
                            inner_block = inner_block.overflow_hidden();
                        }
                    }

                    // Drag handle. A blank slot keeps the content in place when the bar is hidden.
                    let header = div()
                        .id(("drag-header", id_num))
                        .h(px(12.0))
                        .bg(rgb(crate::constants::colors::note_chrome_bar()))
                        .cursor_move()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                this.begin_canvas_gesture();
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
                                        this.record_edit(crate::app::history::EditKind::Canvas);
                                        this.edit_canvas_items.retain(|item| match item {
                                            CanvasItem::Mixed(mx) => mx.id != delete_id,
                                            _ => true,
                                        });
                                        if this.active_text_block_id == Some(delete_id.clone()) {
                                            this.active_text_block_id = None;
                                            this.active_block_index = None;
                                            this.edit_body = String::new();
                                            this.reset_body_styles();
                                            this.edit_body_cursor = 0;
                                        }
                                        this.schedule_autosave(cx);
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
                            ContentBlock::Image {
                                path,
                                width,
                                height,
                                offset_x,
                            } => {
                                if let Some(img_data) = self.canvas_image(path) {
                                    let source = gpui::ImageSource::Image(img_data);
                                    let image_row = self.render_inline_image(
                                        id_num,
                                        block_idx,
                                        item_id.clone(),
                                        source,
                                        *width,
                                        *height,
                                        *offset_x,
                                        textbox_width,
                                        cx,
                                    );
                                    inner_block = inner_block.child(image_row);
                                }
                            }
                            ContentBlock::Text {
                                text,
                                bold_spans,
                                italic_spans,
                                underline_spans,
                                strike_spans,
                                font_runs,
                                line_layouts,
                            } => {
                                let is_seg_active =
                                    is_active && self.active_block_index == Some(block_idx);
                                let seg_id_num = id_num.wrapping_add(block_idx * 7 + 1);
                                let styles = TextStyleSpans {
                                    bold: bold_spans.clone(),
                                    italic: italic_spans.clone(),
                                    underline: underline_spans.clone(),
                                    strike: strike_spans.clone(),
                                    font_runs: font_runs.clone(),
                                    line_layouts: line_layouts.clone(),
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
                        .left(px(self.place_x(m_pos.0)))
                        .top(px(self.place_y(m_pos.1)))
                        .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                            this.set_canvas_item_hover(&hover_id, *hovered, cx);
                        }))
                        .child(inner_block);

                    // Right edge is always hittable, so an unselected box still shows its border
                    // while the pointer is there and while the box is being resized.
                    let edge_hover_id = item_id.clone();
                    wrapper = wrapper.child(
                        div()
                            .id(("resize-handle", id_num))
                            .absolute()
                            .right(px(-3.0))
                            .top(px(0.0))
                            .bottom(px(0.0))
                            .w(px(8.0))
                            .cursor_col_resize()
                            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                this.set_canvas_edge_hover(&edge_hover_id, *hovered, cx);
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.begin_canvas_gesture();
                                    this.resize_item_id = Some(resize_id.clone());
                                    this.resize_block_index = None;
                                    this.resize_start_mouse = Some(event.position);
                                    this.resize_start_size = Some((textbox_width, 100.0));
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            ),
                    );

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
            || self.hovered_canvas_edge_id.as_deref() == Some(item_id)
            || self.drag_item_id.as_deref() == Some(item_id)
            || self.resize_item_id.as_deref() == Some(item_id)
            || self
                .selected_inline_image
                .as_ref()
                .is_some_and(|(id, _)| id == item_id)
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

    /// Tracks the pointer on the right edge only. Leaving that edge does not clear a hover
    /// that still belongs to the rest of the box.
    fn set_canvas_edge_hover(&mut self, item_id: &str, hovered: bool, cx: &mut Context<Self>) {
        if hovered {
            if self.hovered_canvas_edge_id.as_deref() != Some(item_id) {
                self.hovered_canvas_edge_id = Some(item_id.to_string());
                cx.notify();
            }
        } else if self.hovered_canvas_edge_id.as_deref() == Some(item_id) {
            self.hovered_canvas_edge_id = None;
            cx.notify();
        }
    }

    /// Image inside a text box: click to select, drag sideways, or zoom from the frame handles.
    #[allow(clippy::too_many_arguments)]
    fn render_inline_image(
        &self,
        id_num: usize,
        block_idx: usize,
        item_id: String,
        source: gpui::ImageSource,
        img_w: f32,
        img_h: f32,
        offset_x: f32,
        box_w: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let max_off = (box_w - img_w).max(0.0);
        let left = offset_x.clamp(0.0, max_off);
        let selected = self
            .selected_inline_image
            .as_ref()
            .is_some_and(|(id, idx)| id == &item_id && *idx == block_idx);
        let move_id = item_id.clone();
        let mut image = div()
            .id(("inline-image", id_num.wrapping_add(block_idx)))
            .absolute()
            .left(px(self.scaled(left)))
            .top(px(0.0))
            .w(px(self.scaled(img_w)))
            .h(px(self.scaled(img_h)))
            .cursor_move()
            .child(
                img(source)
                    .w(px(self.scaled(img_w)))
                    .h(px(self.scaled(img_h))),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                    this.begin_canvas_gesture();
                    this.selected_inline_image = Some((move_id.clone(), block_idx));
                    this.inline_image_gesture = Some(crate::app::InlineImageGesture {
                        item_id: move_id.clone(),
                        block_index: block_idx,
                        box_width: box_w,
                        kind: crate::app::InlineGestureKind::Move {
                            start_x: event.position.x.as_f32(),
                            start_offset: left,
                        },
                    });
                    this.is_panning = false;
                    cx.notify();
                    cx.stop_propagation();
                }),
            );
        if selected {
            image = image.border_1().border_color(rgb(0xc8c8c8));
            for handle in [
                crate::app::ImageHandle::Nw,
                crate::app::ImageHandle::N,
                crate::app::ImageHandle::Ne,
                crate::app::ImageHandle::E,
                crate::app::ImageHandle::Se,
                crate::app::ImageHandle::S,
                crate::app::ImageHandle::Sw,
                crate::app::ImageHandle::W,
            ] {
                image = image.child(self.inline_zoom_handle(
                    handle,
                    item_id.clone(),
                    block_idx,
                    img_w,
                    img_h,
                    left,
                    box_w,
                    id_num,
                    cx,
                ));
            }
        }
        div()
            .relative()
            .w(px(self.scaled(box_w)))
            .h(px(self.scaled(img_h)))
            .child(image)
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn inline_zoom_handle(
        &self,
        handle: crate::app::ImageHandle,
        item_id: String,
        block_idx: usize,
        img_w: f32,
        img_h: f32,
        offset: f32,
        box_w: f32,
        id_num: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (x, y) = handle_origin(handle, self.scaled(img_w), self.scaled(img_h));
        let handle_n = handle as usize;
        let mut knob = div()
            .id(("img-zoom", id_num.wrapping_add(block_idx * 8 + handle_n)))
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(8.0))
            .h(px(8.0))
            .bg(rgb(0x1e1e1e))
            .border_1()
            .border_color(rgb(0xc8c8c8));
        knob = match handle {
            crate::app::ImageHandle::N | crate::app::ImageHandle::S => knob.cursor_n_resize(),
            crate::app::ImageHandle::E | crate::app::ImageHandle::W => knob.cursor_e_resize(),
            crate::app::ImageHandle::Nw | crate::app::ImageHandle::Se => knob.cursor_nwse_resize(),
            crate::app::ImageHandle::Ne | crate::app::ImageHandle::Sw => knob.cursor_nesw_resize(),
        };
        knob.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                this.begin_canvas_gesture();
                this.selected_inline_image = Some((item_id.clone(), block_idx));
                this.inline_image_gesture = Some(crate::app::InlineImageGesture {
                    item_id: item_id.clone(),
                    block_index: block_idx,
                    box_width: box_w,
                    kind: crate::app::InlineGestureKind::Zoom {
                        handle,
                        start_x: event.position.x.as_f32(),
                        start_y: event.position.y.as_f32(),
                        start_w: img_w,
                        start_h: img_h,
                        start_offset: offset,
                    },
                });
                this.is_panning = false;
                cx.notify();
                cx.stop_propagation();
            }),
        )
        .into_any_element()
    }

    /// Applies a horizontal move or a uniform zoom to the image under the pointer.
    pub(crate) fn update_inline_image_gesture(&mut self, pos: gpui::Point<gpui::Pixels>) -> bool {
        let Some(gesture) = self.inline_image_gesture.clone() else {
            return false;
        };
        let zoom = self.canvas_zoom.max(0.25);
        let Some(crate::models::CanvasItem::Mixed(item)) =
            self.edit_canvas_items.iter_mut().find(|item| match item {
                crate::models::CanvasItem::Mixed(m) => m.id == gesture.item_id,
                _ => false,
            })
        else {
            return false;
        };
        let Some(crate::models::ContentBlock::Image {
            width,
            height,
            offset_x,
            ..
        }) = item.blocks.get_mut(gesture.block_index)
        else {
            return false;
        };
        match gesture.kind {
            crate::app::InlineGestureKind::Move {
                start_x,
                start_offset,
            } => {
                let dx = (pos.x.as_f32() - start_x) / zoom;
                let max_off = (gesture.box_width - *width).max(0.0);
                *offset_x = (start_offset + dx).clamp(0.0, max_off);
            }
            crate::app::InlineGestureKind::Zoom {
                handle,
                start_x,
                start_y,
                start_w,
                start_h,
                start_offset,
            } => {
                let dx = (pos.x.as_f32() - start_x) / zoom;
                let dy = (pos.y.as_f32() - start_y) / zoom;
                let delta = zoom_delta(handle, dx, dy);
                let aspect = if start_w > 1.0 {
                    start_h / start_w
                } else {
                    1.0
                };
                let max_w = gesture.box_width.max(48.0);
                let new_w = (start_w + delta).clamp(48.0, max_w);
                let new_h = (new_w * aspect).clamp(48.0, 1600.0);
                let mut new_off = start_offset;
                if matches!(
                    handle,
                    crate::app::ImageHandle::W
                        | crate::app::ImageHandle::Nw
                        | crate::app::ImageHandle::Sw
                ) {
                    new_off = start_offset - (new_w - start_w);
                }
                let max_off = (gesture.box_width - new_w).max(0.0);
                *width = new_w;
                *height = new_h;
                *offset_x = new_off.clamp(0.0, max_off);
            }
        }
        true
    }

    pub(crate) fn align_inline_image(
        &mut self,
        item_id: &str,
        block_index: usize,
        which: i32,
        box_w: f32,
    ) {
        let Some(crate::models::CanvasItem::Mixed(item)) =
            self.edit_canvas_items.iter_mut().find(|item| match item {
                crate::models::CanvasItem::Mixed(m) => m.id == item_id,
                _ => false,
            })
        else {
            return;
        };
        let Some(crate::models::ContentBlock::Image {
            width, offset_x, ..
        }) = item.blocks.get_mut(block_index)
        else {
            return;
        };
        let max_off = (box_w - *width).max(0.0);
        *offset_x = match which {
            1 => max_off / 2.0,
            2 => max_off,
            _ => 0.0,
        };
    }
}

fn handle_origin(handle: crate::app::ImageHandle, img_w: f32, img_h: f32) -> (f32, f32) {
    let edge = -4.0;
    match handle {
        crate::app::ImageHandle::Nw => (edge, edge),
        crate::app::ImageHandle::N => (img_w / 2.0 - 4.0, edge),
        crate::app::ImageHandle::Ne => (img_w - 4.0, edge),
        crate::app::ImageHandle::E => (img_w - 4.0, img_h / 2.0 - 4.0),
        crate::app::ImageHandle::Se => (img_w - 4.0, img_h - 4.0),
        crate::app::ImageHandle::S => (img_w / 2.0 - 4.0, img_h - 4.0),
        crate::app::ImageHandle::Sw => (edge, img_h - 4.0),
        crate::app::ImageHandle::W => (edge, img_h / 2.0 - 4.0),
    }
}

fn zoom_delta(handle: crate::app::ImageHandle, dx: f32, dy: f32) -> f32 {
    match handle {
        crate::app::ImageHandle::E => dx,
        crate::app::ImageHandle::W => -dx,
        crate::app::ImageHandle::S => dy,
        crate::app::ImageHandle::N => -dy,
        crate::app::ImageHandle::Se => {
            if dx.abs() >= dy.abs() {
                dx
            } else {
                dy
            }
        }
        crate::app::ImageHandle::Nw => {
            if dx.abs() >= dy.abs() {
                -dx
            } else {
                -dy
            }
        }
        crate::app::ImageHandle::Ne => {
            if dx.abs() >= dy.abs() {
                dx
            } else {
                -dy
            }
        }
        crate::app::ImageHandle::Sw => {
            if dx.abs() >= dy.abs() {
                -dx
            } else {
                dy
            }
        }
    }
}
