use gpui::{div, img, prelude::*, px, rgb, AnyElement, Context, MouseButton};
use std::sync::Arc;

use crate::app::NotesApp;
use crate::helpers::{calculate_canvas_drag_offset, calculate_canvas_text_offset};
use crate::models::{ActiveField, CanvasItem};

impl NotesApp {
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
                    let t_id_clone = t.id.clone();
                    let drag_id_clone = drag_id.clone();
                    let t_pos = (t.x, t.y);

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
                                    .text_size(px(10.0))
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
                                                this.edit_body = String::new();
                                                this.edit_body_cursor = 0;
                                            }
                                            cx.notify();
                                            cx.stop_propagation();
                                        }),
                                    ),
                            ),
                    );

                    // Text Editor content or static presentation
                    if is_active {
                        let body_text = self.edit_body.clone();
                        let cursor_idx = self.edit_body_cursor;
                        let anchor_idx = self.edit_body_anchor;

                        let build_body_content = {
                            if body_text.is_empty() {
                                div()
                                    .flex()
                                    .items_center()
                                    .child(div().text_color(rgb(0x606060)).child("Type note..."))
                                    .child(if is_body_focused {
                                        div()
                                            .w(px(2.0))
                                            .h(px(16.0))
                                            .bg(rgb(0x0078d4))
                                            .flex_shrink_0()
                                    } else {
                                        div()
                                    })
                            } else {
                                let sel_start =
                                    anchor_idx.map(|a| a.min(cursor_idx)).unwrap_or(cursor_idx);
                                let sel_end =
                                    anchor_idx.map(|a| a.max(cursor_idx)).unwrap_or(cursor_idx);
                                let has_sel = sel_start < sel_end;

                                let mut line_rows: Vec<AnyElement> = Vec::new();
                                let logical_lines: Vec<&str> = body_text.split('\n').collect();
                                let mut global_offset = 0;
                                for (line_idx, line) in logical_lines.iter().enumerate() {
                                    let line_len = line.chars().count();
                                    let is_last_logical_line = line_idx == logical_lines.len() - 1;
                                    let line_global_start = global_offset;
                                    let line_global_end = global_offset + line_len;

                                    let mut row = div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .min_h(px(20.0));

                                    if is_body_focused && !has_sel {
                                        let is_cursor_in_line = if is_last_logical_line {
                                            cursor_idx >= line_global_start
                                                && cursor_idx <= line_global_end
                                        } else {
                                            cursor_idx >= line_global_start
                                                && cursor_idx <= line_global_end
                                        };

                                        if is_cursor_in_line {
                                            let local_cursor = cursor_idx
                                                .saturating_sub(line_global_start)
                                                .min(line_len);
                                            let line_chars: Vec<char> = line.chars().collect();
                                            let before_str: String =
                                                line_chars[..local_cursor].iter().collect();
                                            let after_str: String =
                                                line_chars[local_cursor..].iter().collect();

                                            if !before_str.is_empty() {
                                                row = row.child(
                                                    div()
                                                        .text_color(rgb(0xd4d4d4))
                                                        .child(before_str.replace(' ', "\u{00A0}")),
                                                );
                                            }
                                            row = row.child(
                                                div()
                                                    .w(px(2.0))
                                                    .h(px(16.0))
                                                    .bg(rgb(0x0078d4))
                                                    .flex_shrink_0(),
                                            );
                                            if !after_str.is_empty() {
                                                row = row.child(
                                                    div()
                                                        .text_color(rgb(0xd4d4d4))
                                                        .child(after_str.replace(' ', "\u{00A0}")),
                                                );
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
                                    } else if has_sel {
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

                                            let before_str: String =
                                                line_chars[..loc_start].iter().collect();
                                            let sel_str: String =
                                                line_chars[loc_start..loc_end].iter().collect();
                                            let after_str: String =
                                                line_chars[loc_end..].iter().collect();

                                            if !before_str.is_empty() {
                                                row = row.child(
                                                    div()
                                                        .flex_shrink_0()
                                                        .text_color(rgb(0xd4d4d4))
                                                        .child(before_str.replace(' ', "\u{00A0}")),
                                                );
                                            }                                             if !sel_str.is_empty() {
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

                                div().flex().flex_col().children(line_rows)
                            }
                        };

                        let t_id_for_down_left = t_id_clone.clone();
                        let t_id_for_down_right = t_id_clone.clone();
                        let t_pos_move = t_pos;

                        inner_block = inner_block.child(
                            div()
                                .id(("text-content", id_num))
                                .p(px(5.0))
                                .text_size(px(12.0))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(
                                        move |this, event: &gpui::MouseDownEvent, window, cx| {
                                            this.focus_handle.focus(window, cx);
                                            this.is_panning = false;
                                            this.pan_start_mouse = None;
                                            this.pan_start_val = None;
                                            this.sync_active_text_block();
                                            this.active_text_block_id =
                                                Some(t_id_for_down_left.clone());
                                            this.active_field = ActiveField::Body;
                                            if let Some(CanvasItem::Text(tx)) = this
                                                .edit_canvas_items
                                                .iter()
                                                .find(|item| match item {
                                                    CanvasItem::Text(blk) => {
                                                        blk.id == t_id_for_down_left
                                                    }
                                                    _ => false,
                                                })
                                            {
                                                this.edit_body = tx.text.clone();
                                            }
                                            let click_idx = calculate_canvas_text_offset(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                t_pos.0,
                                                t_pos.1,
                                                this.canvas_top_y,
                                                &this.edit_body,
                                                textbox_width,
                                            );
                                            this.edit_body_cursor = click_idx;
                                            this.edit_body_anchor = Some(click_idx);
                                            this.is_selecting_body = true;
                                            cx.notify();
                                            cx.stop_propagation();
                                        },
                                    ),
                                )
                                .on_mouse_down(
                                    MouseButton::Right,
                                    cx.listener(
                                        move |this, event: &gpui::MouseDownEvent, window, cx| {
                                            this.focus_handle.focus(window, cx);
                                            this.is_panning = false;
                                            this.pan_start_mouse = None;
                                            this.pan_start_val = None;
                                            this.sync_active_text_block();
                                            this.active_text_block_id =
                                                Some(t_id_for_down_right.clone());
                                            this.active_field = ActiveField::Body;
                                            if let Some(CanvasItem::Text(tx)) = this
                                                .edit_canvas_items
                                                .iter()
                                                .find(|item| match item {
                                                    CanvasItem::Text(blk) => {
                                                        blk.id == t_id_for_down_right
                                                    }
                                                    _ => false,
                                                })
                                            {
                                                this.edit_body = tx.text.clone();
                                            }
                                            let click_idx = calculate_canvas_text_offset(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                t_pos.0,
                                                t_pos.1,
                                                this.canvas_top_y,
                                                &this.edit_body,
                                                textbox_width,
                                            );
                                            this.edit_body_cursor = click_idx;
                                            this.edit_body_anchor = Some(click_idx);
                                            this.is_selecting_body = true;
                                            cx.notify();
                                            cx.stop_propagation();
                                        },
                                    ),
                                )
                                .on_mouse_move(cx.listener(
                                    move |this, event: &gpui::MouseMoveEvent, _, cx| {
                                        if this.is_selecting_body {
                                            let anchor = this
                                                .edit_body_anchor
                                                .unwrap_or(this.edit_body_cursor);
                                            let drag_idx = calculate_canvas_drag_offset(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                t_pos_move.0,
                                                t_pos_move.1,
                                                this.canvas_top_y,
                                                &this.edit_body,
                                                anchor,
                                                textbox_width,
                                            );
                                            this.edit_body_cursor = drag_idx;
                                            cx.notify();
                                            cx.stop_propagation();
                                        }
                                    },
                                ))
                                .on_mouse_up(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if this.is_selecting_body {
                                            if this.edit_body_anchor == Some(this.edit_body_cursor)
                                            {
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
                                            if this.edit_body_anchor == Some(this.edit_body_cursor)
                                            {
                                                this.edit_body_anchor = None;
                                            }
                                            this.is_selecting_body = false;
                                            cx.notify();
                                            cx.stop_propagation();
                                        }
                                    }),
                                )
                                .child(div().w(px(textbox_width - 16.0)).child(build_body_content)),
                        );
                    } else {
                        let t_id_for_inactive_left = t_id_clone.clone();
                        let t_id_for_inactive_right = t_id_clone.clone();
                        let t_pos_move = t_pos;

                        inner_block = inner_block.child(
                            div()
                                .id(("text-content", id_num))
                                .p(px(5.0))
                                .text_size(px(12.0))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(
                                        move |this, event: &gpui::MouseDownEvent, window, cx| {
                                            this.focus_handle.focus(window, cx);
                                            this.is_panning = false;
                                            this.pan_start_mouse = None;
                                            this.pan_start_val = None;
                                            this.sync_active_text_block();
                                            this.active_text_block_id =
                                                Some(t_id_for_inactive_left.clone());
                                            this.active_field = ActiveField::Body;
                                            if let Some(CanvasItem::Text(tx)) = this
                                                .edit_canvas_items
                                                .iter()
                                                .find(|item| match item {
                                                    CanvasItem::Text(blk) => {
                                                        blk.id == t_id_for_inactive_left
                                                    }
                                                    _ => false,
                                                })
                                            {
                                                this.edit_body = tx.text.clone();
                                            }
                                            let click_idx = calculate_canvas_text_offset(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                t_pos.0,
                                                t_pos.1,
                                                this.canvas_top_y,
                                                &this.edit_body,
                                                textbox_width,
                                            );
                                            this.edit_body_cursor = click_idx;
                                            this.edit_body_anchor = Some(click_idx);
                                            this.is_selecting_body = true;
                                            cx.notify();
                                            cx.stop_propagation();
                                        },
                                    ),
                                )
                                .on_mouse_down(
                                    MouseButton::Right,
                                    cx.listener(
                                        move |this, event: &gpui::MouseDownEvent, window, cx| {
                                            this.focus_handle.focus(window, cx);
                                            this.is_panning = false;
                                            this.pan_start_mouse = None;
                                            this.pan_start_val = None;
                                            this.sync_active_text_block();
                                            this.active_text_block_id =
                                                Some(t_id_for_inactive_right.clone());
                                            this.active_field = ActiveField::Body;
                                            if let Some(CanvasItem::Text(tx)) = this
                                                .edit_canvas_items
                                                .iter()
                                                .find(|item| match item {
                                                    CanvasItem::Text(blk) => {
                                                        blk.id == t_id_for_inactive_right
                                                    }
                                                    _ => false,
                                                })
                                            {
                                                this.edit_body = tx.text.clone();
                                            }
                                            let click_idx = calculate_canvas_text_offset(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                t_pos.0,
                                                t_pos.1,
                                                this.canvas_top_y,
                                                &this.edit_body,
                                                textbox_width,
                                            );
                                            this.edit_body_cursor = click_idx;
                                            this.edit_body_anchor = Some(click_idx);
                                            this.is_selecting_body = true;
                                            cx.notify();
                                            cx.stop_propagation();
                                        },
                                    ),
                                )
                                .on_mouse_move(cx.listener(
                                    move |this, event: &gpui::MouseMoveEvent, _, cx| {
                                        if this.is_selecting_body {
                                            let anchor = this
                                                .edit_body_anchor
                                                .unwrap_or(this.edit_body_cursor);
                                            let drag_idx = calculate_canvas_drag_offset(
                                                event.position,
                                                this.is_sidebar_open,
                                                this.pan_x,
                                                this.pan_y,
                                                t_pos_move.0,
                                                t_pos_move.1,
                                                this.canvas_top_y,
                                                &this.edit_body,
                                                anchor,
                                                textbox_width,
                                            );
                                            this.edit_body_cursor = drag_idx;
                                            cx.notify();
                                            cx.stop_propagation();
                                        }
                                    },
                                ))
                                .on_mouse_up(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if this.is_selecting_body {
                                            if this.edit_body_anchor == Some(this.edit_body_cursor)
                                            {
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
                                            if this.edit_body_anchor == Some(this.edit_body_cursor)
                                            {
                                                this.edit_body_anchor = None;
                                            }
                                            this.is_selecting_body = false;
                                            cx.notify();
                                            cx.stop_propagation();
                                        }
                                    }),
                                )
                                .child(
                                    div()
                                        .w(px(textbox_width - 16.0))
                                        .flex()
                                        .flex_row()
                                        .flex_wrap()
                                        .children(t.text.lines().map(|line| {
                                            div().w(px(textbox_width - 16.0)).child(line.to_owned())
                                        })),
                                ),
                        );
                    }

                    // corner resize handle container
                    let mut wrapper = div()
                        .absolute()
                        .left(px(t.x + self.pan_x))
                        .top(px(t.y + self.pan_y))
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
                                        .text_size(px(10.0))
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
            }
        }

        canvas_elements
    }
}
