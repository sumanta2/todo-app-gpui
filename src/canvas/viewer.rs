//! Read-only canvas for the page that is currently open.

use gpui::{
    div, img, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window,
};
use std::sync::Arc;

use crate::app::NotesApp;
use crate::canvas::canvas_top_tracker;
use crate::constants::{
    colors::{TEXT_HINT, TEXT_PRIMARY, TEXT_SECONDARY},
    typography::{BUTTON_FONT_SIZE, HINT_FONT_SIZE},
};
use crate::helpers::hash_str;
use crate::models::{load_canvas_items, CanvasItem, ContentBlock, Note, NoteContent};
use crate::text::selection::calculate_canvas_text_offset_full;

use super::viewer_text::resolve_viewer_segment;

impl NotesApp {
    /// Renders the read-only canvas viewer for a note after editing is finished.
    ///
    /// This method paints the saved canvas text blocks, images, and combined image+text boxes
    /// exactly as they were last serialized, while preserving the current viewer selection state.
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
                    let seg = self.render_viewer_text_segment(
                        t.id.clone(),
                        t.text.clone(),
                        crate::models::TextStyleSpans {
                            bold: t.bold_spans.clone(),
                            italic: t.italic_spans.clone(),
                            underline: t.underline_spans.clone(),
                            strike: t.strike_spans.clone(),
                        },
                        t.x,
                        t.y,
                        t.width.unwrap_or(250.0),
                        cx,
                    );
                    viewer_elements.push(seg);
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

                //  RENDER CANVAS-ITEM COMBINED IMAGE+TEXT BOXES FOR VIEW =====================================================
                CanvasItem::Mixed(m) => {
                    let mut y_cursor = m.y;
                    for (block_idx, block) in m.blocks.iter().enumerate() {
                        match block {
                            ContentBlock::Image { path, width, height } => {
                                if let Some(img_data) = self.decrypt_image(path) {
                                    let source = gpui::ImageSource::Image(Arc::new(img_data));
                                    let image_el = div()
                                        .absolute()
                                        .left(px(m.x + self.pan_x))
                                        .top(px(y_cursor + self.pan_y))
                                        .w(px(*width))
                                        .h(px(*height))
                                        .child(
                                            img(source)
                                                .w(px(*width))
                                                .h(px(*height))
                                                .rounded(px(6.0))
                                                .border_1()
                                                .border_color(rgb(0x3d3d3d)),
                                        );
                                    viewer_elements.push(image_el.into_any_element());
                                    y_cursor += *height + 4.0;
                                }
                            }
                            ContentBlock::Text {
                                text,
                                bold_spans,
                                italic_spans,
                                underline_spans,
                                strike_spans,
                            } => {
                                let seg_id = format!("{}::{}", m.id, block_idx);
                                let line_count = text.split('\n').count().max(1) as f32;
                                let seg = self.render_viewer_text_segment(
                                    seg_id,
                                    text.clone(),
                                    crate::models::TextStyleSpans {
                                        bold: bold_spans.clone(),
                                        italic: italic_spans.clone(),
                                        underline: underline_spans.clone(),
                                        strike: strike_spans.clone(),
                                    },
                                    m.x,
                                    y_cursor,
                                    m.width.unwrap_or(250.0),
                                    cx,
                                );
                                viewer_elements.push(seg);
                                y_cursor += line_count * self.canvas_line_height() + 10.0;
                            }
                        }
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
        let entity = cx.entity();

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
                            if let Some((text, bold_spans, item_x, item_y, item_w)) =
                                resolve_viewer_segment(&active_items_for_drag, active_id)
                            {
                                let total_chars = text.chars().count();
                                let bold_flags =
                                    crate::text::styles::spans_to_bool_vec(&bold_spans, total_chars);
                                let drag_idx = calculate_canvas_text_offset_full(
                                    event.position,
                                    this.is_sidebar_open,
                                    this.pan_x,
                                    this.pan_y,
                                    item_x,
                                    item_y,
                                    this.canvas_top_y,
                                    &text,
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
                    .text_size(px(HINT_FONT_SIZE))
                    .text_color(rgb(TEXT_HINT))
                    .child("💡 Drag the background to pan the canvas"),
            )
            .child(canvas_top_tracker(entity))
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
                    .text_size(px(self.section_name_font_size))
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
                        rgb(TEXT_SECONDARY)
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
                        .text_size(px(BUTTON_FONT_SIZE))
                        .text_color(rgb(TEXT_PRIMARY))
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
                        .text_size(px(BUTTON_FONT_SIZE))
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
                                            .text_size(px(self.page_heading_font_size))
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
