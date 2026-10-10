//! Editable page heading on the canvas.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, MouseButton};

use crate::app::NotesApp;
use crate::constants::layout::HEADING_PADDING_LEFT;
use crate::models::ActiveField;

impl NotesApp {
    pub(crate) fn build_heading_editor(
        &self,
        is_heading_focused: bool,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let font_size = self.page_heading_font_size * self.canvas_zoom;
        if !self.page_renaming {
            let label = if self.edit_heading.trim().is_empty() {
                "Heading..."
            } else {
                self.edit_heading.as_str()
            };
            let row = div()
                .id("heading-menu-target")
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                        this.begin_page_rename_at(window, event.position.x.as_f32(), cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                        if this.is_editing {
                            this.section_menu_at = None;
                            this.section_renaming = false;
                            this.page_renaming = false;
                            this.page_menu_at =
                                Some((event.position.x.as_f32(), event.position.y.as_f32()));
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .text_size(px(font_size))
                        .whitespace_nowrap()
                        .text_color(rgb(if self.edit_heading.trim().is_empty() {
                            crate::constants::colors::note_hint()
                        } else {
                            crate::constants::colors::note_ink()
                        }))
                        .child(label.to_string()),
                );
            return self.page_title_with_rule(
                window,
                row.into_any_element(),
                label,
                gpui::FontWeight::NORMAL,
            );
        }
        if self.edit_heading.is_empty() {
            let row = div()
                .id("heading-placeholder")
                .relative()
                .flex()
                .items_center()
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, window, cx| {
                        this.active_field = ActiveField::Heading;
                        this.focus_handle.focus(window, cx);
                        this.edit_heading_cursor = 0;
                        this.edit_heading_anchor = None;
                        this.is_selecting_heading = false;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .text_size(px(font_size))
                        .whitespace_nowrap()
                        .text_color(rgb(crate::constants::colors::note_hint()))
                        .child("Heading..."),
                )
                .child(if is_heading_focused {
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(2.0))
                        .w(px(crate::constants::typography::CURSOR_WIDTH))
                        .h(px(font_size))
                        .bg(if self.cursor_visible {
                            rgb(crate::constants::colors::cursor())
                        } else {
                            rgba(0x00000000)
                        })
                } else {
                    div()
                });
            return self.page_title_with_rule(
                window,
                row.into_any_element(),
                "Heading...",
                gpui::FontWeight::NORMAL,
            );
        } else {
            let sel_start = self
                .edit_heading_anchor
                .map(|a| a.min(self.edit_heading_cursor))
                .unwrap_or(self.edit_heading_cursor);
            let sel_end = self
                .edit_heading_anchor
                .map(|a| a.max(self.edit_heading_cursor))
                .unwrap_or(self.edit_heading_cursor);
            let has_selection = sel_start < sel_end;
            let text = &self.edit_heading;
            let cursor = self.edit_heading_cursor;
            let chars: Vec<char> = text.chars().collect();

            let mut elements: Vec<AnyElement> = Vec::new();
            if has_selection {
                let prefix: String = chars[..sel_start.min(chars.len())].iter().collect();
                let sel_chunk: String = chars[sel_start.min(chars.len())..sel_end.min(chars.len())]
                    .iter()
                    .collect();
                let before_disp = prefix.replace(' ', "\u{00A0}");
                let sel_disp = sel_chunk.replace(' ', "\u{00A0}");
                elements.push(
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(1.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(font_size))
                                .text_color(rgba(0x00000000))
                                .child(before_disp),
                        )
                        .child(
                            div()
                                .bg(rgb(0x004c87))
                                .rounded(px(2.0))
                                .h(px(font_size + 2.0))
                                .flex()
                                .items_center()
                                .child(
                                    div()
                                        .text_size(px(font_size))
                                        .text_color(rgba(0x00000000))
                                        .child(sel_disp),
                                ),
                        )
                        .into_any_element(),
                );
            }

            elements.push(
                div()
                    .text_size(px(font_size))
                    .whitespace_nowrap()
                    .text_color(rgb(crate::constants::colors::note_ink()))
                    .child(if text.is_empty() {
                        "\u{00A0}".to_string()
                    } else {
                        text.replace(' ', "\u{00A0}")
                    })
                    .into_any_element(),
            );

            if is_heading_focused && !has_selection {
                let prefix: String = chars[..cursor.min(chars.len())].iter().collect();
                let before_disp = prefix.replace(' ', "\u{00A0}");
                elements.push(
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(2.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(font_size))
                                .text_color(rgba(0x00000000))
                                .child(before_disp),
                        )
                        .child(
                            div()
                                .w(px(crate::constants::typography::CURSOR_WIDTH))
                                .h(px(font_size))
                                .bg(if self.cursor_visible {
                                    rgb(crate::constants::colors::cursor())
                                } else {
                                    rgba(0x00000000)
                                })
                                .flex_shrink_0()
                                .ml(px(-1.0)),
                        )
                        .into_any_element(),
                );
            }

            let row = div()
                .id("heading-editor")
                .relative()
                .flex()
                .items_center()
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                        this.active_field = ActiveField::Heading;
                        this.focus_handle.focus(window, cx);
                        let sidebar_w = this.layout_sidebar_w();
                        let rel_x = ((event.position.x.as_f32() - sidebar_w - this.pan_x)
                            / this.canvas_zoom.max(0.25)
                            - HEADING_PADDING_LEFT)
                            .max(0.0);
                        let click_idx = this.heading_index_at_x(window, rel_x);
                        this.edit_heading_cursor = click_idx;
                        this.edit_heading_anchor = Some(click_idx);
                        this.is_selecting_heading = true;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, window, cx| {
                    if this.is_selecting_heading {
                        let sidebar_w = this.layout_sidebar_w();
                        let rel_x = ((event.position.x.as_f32() - sidebar_w - this.pan_x)
                            / this.canvas_zoom.max(0.25)
                            - HEADING_PADDING_LEFT)
                            .max(0.0);
                        let drag_idx = this.heading_index_at_x(window, rel_x);
                        if crate::text::selection::assign_if_changed(
                            &mut this.edit_heading_cursor,
                            drag_idx,
                        ) {
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        if this.is_selecting_heading {
                            if this.edit_heading_anchor == Some(this.edit_heading_cursor) {
                                this.edit_heading_anchor = None;
                            }
                            this.is_selecting_heading = false;
                            cx.notify();
                            cx.stop_propagation();
                        }
                    }),
                )
                .children(elements);
            self.page_title_with_rule(
                window,
                row.into_any_element(),
                text,
                gpui::FontWeight::NORMAL,
            )
        }
    }

    /// Opens the page-name editor and puts the caret at the click.
    ///
    /// Clicks to the left of the title land on the first character. Clicks to the right land at the end.
    pub(crate) fn begin_page_rename_at(
        &mut self,
        window: &mut gpui::Window,
        mouse_x: f32,
        cx: &mut Context<Self>,
    ) {
        if !self.is_editing {
            return;
        }
        self.section_menu_at = None;
        self.page_menu_at = None;
        self.section_renaming = false;
        self.page_renaming = true;
        self.active_field = ActiveField::Heading;
        self.focus_handle.focus(window, cx);
        let rel_x = ((mouse_x - self.layout_sidebar_w() - self.pan_x) / self.canvas_zoom.max(0.25)
            - HEADING_PADDING_LEFT)
            .max(0.0);
        let click_idx = self.heading_index_at_x(window, rel_x);
        self.edit_heading_cursor = click_idx;
        self.edit_heading_anchor = Some(click_idx);
        self.is_selecting_heading = true;
        self.cursor_visible = true;
        cx.notify();
    }
}
