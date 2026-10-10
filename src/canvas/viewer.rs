//! Read-only canvas for the page that is currently open.

use gpui::{div, img, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::canvas::canvas_top_tracker;
use crate::constants::typography::BUTTON_FONT_SIZE;
use crate::helpers::hash_str;
use crate::models::{load_canvas_items, CanvasItem, ContentBlock, NoteContent};

impl NotesApp {
    /// Parses a page body once, then reuses that result until the page changes.
    fn cached_page_items(&mut self, page_id: &str, body: &str) -> Vec<CanvasItem> {
        if let Some((cached_id, items)) = &self.page_items_cache {
            if cached_id == page_id {
                return items.clone();
            }
        }
        let items = load_canvas_items(body);
        self.page_items_cache = Some((page_id.to_string(), items.clone()));
        items
    }

    /// Renders the read-only canvas viewer for a note after editing is finished.
    ///
    /// This method paints the saved canvas text blocks, images, and combined image+text boxes
    /// exactly as they were last serialized, while preserving the current viewer selection state.
    pub(crate) fn render_canvas_viewer(
        &mut self,
        content: &NoteContent,
        page_sidebar: AnyElement,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut viewer_elements = Vec::new();

        let open_page = {
            let sec_id = self.active_section_id.as_deref();
            let page_id = self.active_page_id.as_deref();
            sec_id
                .and_then(|sec_id| content.sections.iter().find(|section| section.id == sec_id))
                .and_then(|section| {
                    page_id.and_then(|page_id| section.pages.iter().find(|page| page.id == page_id))
                })
                .map(|page| (page.id.clone(), page.body.clone()))
        };
        let active_page_items = if let Some((page_id, body)) = open_page {
            self.cached_page_items(&page_id, &body)
        } else {
            Vec::new()
        };
        self.prefetch_canvas_images(&active_page_items);

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
                            font_runs: t.font_runs.clone(),
                            line_layouts: t.line_layouts.clone(),
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
                    if let Some(img_data) = self.canvas_image(&img_item.path) {
                        let source = gpui::ImageSource::Image(img_data);
                        let image_el = div()
                            .absolute()
                            .left(px(self.place_x(img_item.x)))
                            .top(px(self.place_y(img_item.y)))
                            .w(px(self.scaled(img_item.width)))
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
                                    .w(px(self.scaled(img_item.width)))
                                    .h(px(self.scaled(img_item.height)))
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
                            ContentBlock::Image {
                                path,
                                width,
                                height,
                                offset_x,
                            } => {
                                if let Some(img_data) = self.canvas_image(path) {
                                    let source = gpui::ImageSource::Image(img_data);
                                    let image_el = div()
                                        .absolute()
                                        .left(px(self.place_x(m.x + offset_x)))
                                        .top(px(self.place_y(y_cursor)))
                                        .w(px(self.scaled(*width)))
                                        .h(px(self.scaled(*height)))
                                        .child(
                                            img(source)
                                                .w(px(self.scaled(*width)))
                                                .h(px(self.scaled(*height)))
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
                                font_runs,
                                line_layouts,
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
                                        font_runs: font_runs.clone(),
                                        line_layouts: line_layouts.clone(),
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

        let entity = cx.entity();

        let canvas_container = div()
            .id("note-viewer-canvas")
            .flex_1()
            .relative()
            .bg(rgb(crate::constants::colors::note_page()))
            .overflow_hidden()
            .cursor_default()
            .on_scroll_wheel(cx.listener(|this, event: &gpui::ScrollWheelEvent, _, cx| {
                this.handle_canvas_scroll(event, cx);
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
                cx.listener(move |this, event: &gpui::MouseMoveEvent, window, cx| {
                    if this.drag_page_sidebar(event.position.x.as_f32()) {
                        cx.notify();
                    } else if this.is_selecting_viewer_text {
                        this.queue_selection_drag(
                            crate::canvas::selection_overlay::HighlightKind::Viewer,
                            event.position,
                            window,
                            cx,
                        );
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
                        this.end_viewer_pointer(cx);
                    }
                    if this.is_panning {
                        this.is_panning = false;
                        this.pan_start_mouse = None;
                        this.pan_start_val = None;
                        cx.notify();
                    }
                    this.page_sidebar_resizing = false;
                }),
            )
            // Pin instruction, hidden for now.
            // .child(
            //     div()
            //         .absolute()
            //         .top(px(
            //             crate::constants::layout::HEADING_PADDING_TOP
            //                 + self.page_heading_font_size
            //                 + 14.0,
            //         ))
            //         .left(px(crate::constants::layout::HEADING_PADDING_LEFT))
            //         .text_size(px(HINT_FONT_SIZE))
            //         .text_color(rgb(TEXT_HINT))
            //         .child("💡 Drag the background to pan the canvas"),
            // )
            .child(canvas_top_tracker(entity))
            .children(viewer_elements)
            .child(
                div()
                    .absolute()
                    .top(px(
                        self.place_y(crate::constants::layout::HEADING_PADDING_TOP)
                    ))
                    .left(px(
                        self.place_x(crate::constants::layout::HEADING_PADDING_LEFT)
                    ))
                    .child(
                        self.page_title_with_rule(
                            div()
                                .text_size(px(self.scaled(self.page_heading_font_size)))
                                .whitespace_nowrap()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(crate::constants::colors::note_ink()))
                                .child(page_name.clone())
                                .into_any_element(),
                            &page_name,
                        ),
                    ),
            );

        let section_tabs = {
            let mut tabs = Vec::new();
            for sec in &content.sections {
                let sec_id = sec.id.clone();
                let is_active = Some(&sec_id) == self.active_section_id.as_ref();
                let click_id = sec_id.clone();

                let mut tab_el = div()
                    .id(("sec-tab", hash_str(&sec_id)))
                    .px(px(10.0))
                    .py(px(4.0))
                    .font_family("Calibri")
                    .text_size(px(self.section_name_font_size))
                    .bg(if is_active {
                        rgb(crate::constants::colors::onenote_page_selected())
                    } else {
                        rgb(crate::constants::colors::onenote_tab_idle())
                    });
                tab_el = tab_el.border_t_2().border_color(if is_active {
                    rgb(crate::constants::colors::onenote_accent())
                } else {
                    rgb(crate::constants::colors::onenote_tab_idle())
                });
                tab_el = tab_el
                    .text_color(if is_active {
                        rgb(crate::constants::colors::onenote_ink())
                    } else {
                        rgb(crate::constants::colors::onenote_ink_muted())
                    })
                    .font_weight(if is_active {
                        gpui::FontWeight::BOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .cursor_pointer()
                    .hover(|style| {
                        if is_active {
                            style
                        } else {
                            style.bg(rgb(crate::constants::colors::onenote_tab_hover()))
                        }
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.switch_to_section(click_id.clone(), cx);
                    }))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(sec.name.clone());

                tabs.push(tab_el.into_any_element());
            }

            let mut left_side = div().flex().flex_row().items_center().gap(px(4.0));
            if !self.is_sidebar_open {
                left_side = left_side.child(self.render_note_switcher(cx));
            }
            let left_side = left_side.children(tabs);

            let right_side = div().flex().gap(px(6.0)).child(
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

        let canvas_container = canvas_container.child(self.render_canvas_view_toggle(cx));

        let mut page = div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(rgb(crate::constants::colors::onenote_bar()))
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
