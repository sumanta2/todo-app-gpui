//! Editable canvas shell: pan, drag, resize, and pointer routing.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::canvas::canvas_top_tracker;
use crate::constants::{
    layout::{HEADING_PADDING_LEFT, HEADING_PADDING_TOP},
    typography::{WEIGHT_BOLD, WEIGHT_NORMAL},
};
use crate::models::{ActiveField, CanvasItem, NoteContent};
use crate::text::selection::calculate_line_text_offset_with_font;

impl NotesApp {
    /// Renders the editable note canvas and its surrounding controls.
    ///
    /// This function assembles the section tabs, the editable heading, the canvas body, and
    /// the page sidebar into one notebook editor surface.
    pub(crate) fn render_canvas_editor(
        &mut self,
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

        let mut canvas_container = div()
            .id("note-body-canvas")
            .flex_1()
            .relative()
            .bg(rgb(crate::constants::colors::NOTE_PAGE))
            .overflow_hidden()
            .on_scroll_wheel(cx.listener(|this, event: &gpui::ScrollWheelEvent, _, cx| {
                this.handle_canvas_scroll(event, cx);
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
            .on_mouse_move(
                cx.listener(|this, event: &gpui::MouseMoveEvent, window, cx| {
                    let mut changed = this.drag_page_sidebar(event.position.x.as_f32());
                    if this.update_inline_image_gesture(event.position) {
                        changed = true;
                    } else if let Some(ref item_id) = this.drag_item_id {
                        if let (Some(start_mouse), Some(start_pos)) =
                            (this.drag_start_mouse, this.drag_start_item_pos)
                        {
                            let zoom = this.canvas_zoom.max(0.25);
                            let dx = (event.position.x.as_f32() - start_mouse.x.as_f32()) / zoom;
                            let dy = (event.position.y.as_f32() - start_mouse.y.as_f32()) / zoom;
                            let new_x = (start_pos.0 + dx).max(0.0);
                            let new_y = (start_pos.1 + dy).max(0.0);

                            if let Some(item) =
                                this.edit_canvas_items.iter_mut().find(|i| match i {
                                    CanvasItem::Text(t) => t.id == *item_id,
                                    CanvasItem::Image(img) => img.id == *item_id,
                                    CanvasItem::Mixed(m) => m.id == *item_id,
                                })
                            {
                                match item {
                                    CanvasItem::Text(t) => {
                                        t.x = new_x;
                                        t.y = new_y;
                                    }
                                    CanvasItem::Image(img) => {
                                        img.x = new_x;
                                        img.y = new_y;
                                    }
                                    CanvasItem::Mixed(m) => {
                                        m.x = new_x;
                                        m.y = new_y;
                                    }
                                }
                                changed = true;
                            }
                        }
                    } else if let Some(ref resize_id) = this.resize_item_id {
                        if let (Some(start_mouse), Some(start_size)) =
                            (this.resize_start_mouse, this.resize_start_size)
                        {
                            let zoom = this.canvas_zoom.max(0.25);
                            let dx = (event.position.x.as_f32() - start_mouse.x.as_f32()) / zoom;
                            let dy = (event.position.y.as_f32() - start_mouse.y.as_f32()) / zoom;
                            let new_width = (start_size.0 + dx).max(100.0);
                            let new_height = (start_size.1 + dy).max(50.0);
                            let resize_block_index = this.resize_block_index;

                            if let Some(item) =
                                this.edit_canvas_items.iter_mut().find(|i| match i {
                                    CanvasItem::Text(t) => t.id == *resize_id,
                                    CanvasItem::Image(img) => img.id == *resize_id,
                                    CanvasItem::Mixed(m) => m.id == *resize_id,
                                })
                            {
                                match item {
                                    CanvasItem::Text(t) => {
                                        t.width = Some(new_width);
                                    }
                                    CanvasItem::Image(img) => {
                                        img.width = new_width;
                                        img.height = new_height;
                                    }
                                    CanvasItem::Mixed(m) => {
                                        if let Some(idx) = resize_block_index {
                                            // Resizing the image inside the box: zoom the image only.
                                            if let Some(crate::models::ContentBlock::Image {
                                                width,
                                                height,
                                                ..
                                            }) = m.blocks.get_mut(idx)
                                            {
                                                *width = new_width;
                                                *height = new_height;
                                            }
                                        } else {
                                            // Resizing the whole box: adjust the text wrap width.
                                            m.width = Some(new_width);
                                        }
                                    }
                                }
                                changed = true;
                            }
                        }
                    } else if this.is_selecting_body {
                        this.queue_selection_drag(
                            crate::canvas::selection_overlay::HighlightKind::Body,
                            event.position,
                            window,
                            cx,
                        );
                    } else if this.is_selecting_heading {
                        let sidebar_w = this.layout_sidebar_w();
                        let rel_x = ((event.position.x.as_f32() - sidebar_w - this.pan_x)
                            / this.canvas_zoom.max(0.25)
                            - HEADING_PADDING_LEFT)
                            .max(0.0);
                        let drag_idx = calculate_line_text_offset_with_font(
                            rel_x,
                            &this.edit_heading,
                            this.page_heading_font_size,
                            WEIGHT_NORMAL,
                            this.font_type(),
                        );
                        if crate::text::selection::assign_if_changed(
                            &mut this.edit_heading_cursor,
                            drag_idx,
                        ) {
                            changed = true;
                        }
                    } else if this.is_selecting_section_name {
                        let rel_x =
                            (event.position.x.as_f32() - this.active_section_tab_x).max(0.0);
                        let drag_idx = calculate_line_text_offset_with_font(
                            rel_x,
                            &this.edit_section_name,
                            this.section_name_font_size,
                            WEIGHT_BOLD,
                            this.font_type(),
                        );
                        if crate::text::selection::assign_if_changed(
                            &mut this.edit_section_name_cursor,
                            drag_idx,
                        ) {
                            changed = true;
                        }
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
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseUpEvent, _, cx| {
                    if this.is_selecting_body {
                        this.end_body_pointer(cx);
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
                                let sidebar_w = this.layout_sidebar_w();
                                let click_x =
                                    (start_mouse.x.as_f32() - sidebar_w - this.pan_x).max(0.0);
                                let click_y =
                                    (start_mouse.y.as_f32() - this.canvas_top_y - this.pan_y)
                                        .max(0.0);

                                this.sync_active_text_block();
                                this.discard_empty_text_items();

                                // A click inside an existing box joins that box.
                                // Empty canvas, including space beside a box, only shows a caret.
                                if let Some((id, block_index, text_top)) =
                                    this.text_target_at(click_x, click_y)
                                {
                                    this.focus_text_line_at_click(
                                        &id,
                                        block_index,
                                        click_y,
                                        text_top,
                                    );
                                } else {
                                    this.selected_inline_image = None;
                                    this.arm_pending_caret(click_x, click_y);
                                }
                            }
                        }
                    }
                    let canvas_edited = this.drag_item_id.is_some()
                        || this.resize_item_id.is_some()
                        || this.inline_image_gesture.is_some();
                    this.drag_item_id = None;
                    this.drag_start_mouse = None;
                    this.drag_start_item_pos = None;
                    this.resize_item_id = None;
                    this.resize_block_index = None;
                    this.resize_start_mouse = None;
                    this.resize_start_size = None;
                    this.is_panning = false;
                    this.pan_start_mouse = None;
                    this.pan_start_val = None;
                    this.inline_image_gesture = None;
                    this.page_sidebar_resizing = false;
                    if canvas_edited {
                        this.end_canvas_gesture(cx);
                    }
                    cx.notify();
                }),
            )
            // Pin instruction, hidden for now.
            // .child(
            //     div()
            //         .absolute()
            //         .top(px(HEADING_PADDING_TOP + self.page_heading_font_size + 14.0))
            //         .left(px(HEADING_PADDING_LEFT))
            //         .text_size(px(HINT_FONT_SIZE))
            //         .text_color(rgb(TEXT_HINT))
            //         .child("💡 Click to place the cursor, then type | Home: B I U S | Ctrl+B / Ctrl+I / Ctrl+U | Drag headers to move | Ctrl+V to paste"),
            // )
            .child(canvas_top_tracker(cx.entity()))
            .children(canvas_elements);

        if self.is_editing
            && self.active_field == ActiveField::Body
            && self.active_text_block_id.is_none()
        {
            if let Some((caret_x, caret_y)) = self.pending_caret {
                canvas_container = canvas_container.child(
                    div()
                        .absolute()
                        .left(px(self.place_x(caret_x)))
                        .top(px(self.place_y(caret_y)))
                        .w(px(crate::constants::typography::CURSOR_WIDTH))
                        .h(px(self.canvas_cursor_height()))
                        .bg(if self.cursor_visible {
                            rgb(crate::constants::colors::NOTE_INK)
                        } else {
                            gpui::rgba(0x00000000)
                        }),
                );
            }
        }

        canvas_container = canvas_container.child(
            div()
                .absolute()
                .top(px(self.place_y(HEADING_PADDING_TOP)))
                .left(px(self.place_x(HEADING_PADDING_LEFT)))
                .child(heading_content),
        );
        canvas_container = canvas_container.child(self.render_canvas_view_toggle(cx));

        let mut page = div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(crate::constants::colors::ONENOTE_BAR))
            .gap(px(0.0));
        if !self.full_page_view {
            page = page.child(section_tabs);
        }
        let mut body = div()
            .flex()
            .flex_row()
            .flex_1()
            .h_full()
            .child(canvas_container);
        if !self.full_page_view {
            body = body.child(page_sidebar);
        }
        page.child(body)
    }
}
