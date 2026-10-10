//! Inline editor for the name of the active section tab.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, MouseButton};

use crate::app::NotesApp;
use crate::constants::typography::WEIGHT_BOLD;
use crate::models::ActiveField;
use crate::text::selection::calculate_line_text_offset_with_font;

impl NotesApp {
    pub(super) fn build_section_name_editor(
        &self,
        is_focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let font_size = self.section_name_font_size;
        if self.edit_section_name.is_empty() {
            div()
                .id("section-name-placeholder")
                .relative()
                .flex()
                .items_center()
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, window, cx| {
                        this.active_field = ActiveField::SectionName;
                        this.focus_handle.focus(window, cx);
                        this.edit_section_name_cursor = 0;
                        this.edit_section_name_anchor = None;
                        this.is_selecting_section_name = false;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .text_size(px(font_size))
                        .font_family("Calibri")
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(crate::constants::colors::note_hint()))
                        .child("Section Name..."),
                )
                .child(if is_focused {
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(1.0))
                        .w(px(1.5))
                        .h(px(font_size + 1.0))
                        .bg(if self.cursor_visible {
                            rgb(crate::constants::colors::cursor())
                        } else {
                            rgba(0x00000000)
                        })
                } else {
                    div()
                })
                .into_any_element()
        } else {
            let sel_start = self
                .edit_section_name_anchor
                .map(|a| a.min(self.edit_section_name_cursor))
                .unwrap_or(self.edit_section_name_cursor);
            let sel_end = self
                .edit_section_name_anchor
                .map(|a| a.max(self.edit_section_name_cursor))
                .unwrap_or(self.edit_section_name_cursor);
            let has_selection = sel_start < sel_end;
            let text = &self.edit_section_name;
            let cursor = self.edit_section_name_cursor;
            let chars: Vec<char> = text.chars().collect();

            let mut els: Vec<AnyElement> = Vec::new();
            if has_selection {
                let prefix: String = chars[..sel_start.min(chars.len())].iter().collect();
                let sel_chunk: String = chars[sel_start.min(chars.len())..sel_end.min(chars.len())]
                    .iter()
                    .collect();
                let before_disp = prefix.replace(' ', "\u{00A0}");
                let sel_disp = sel_chunk.replace(' ', "\u{00A0}");
                els.push(
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(0.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(font_size))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgba(0x00000000))
                                .child(before_disp),
                        )
                        .child(
                            div()
                                .bg(rgb(0x004c87))
                                .rounded(px(2.0))
                                .h(px(font_size + 3.0))
                                .flex()
                                .items_center()
                                .child(
                                    div()
                                        .text_size(px(font_size))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgba(0x00000000))
                                        .child(sel_disp),
                                ),
                        )
                        .into_any_element(),
                );
            }

            els.push(
                div()
                    .text_size(px(font_size))
                    .font_family("Calibri")
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(crate::constants::colors::onenote_ink()))
                    .child(if text.is_empty() {
                        "\u{00A0}".to_string()
                    } else {
                        text.replace(' ', "\u{00A0}")
                    })
                    .into_any_element(),
            );

            if is_focused && !has_selection {
                let prefix: String = chars[..cursor.min(chars.len())].iter().collect();
                let before_disp = prefix.replace(' ', "\u{00A0}");
                els.push(
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
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgba(0x00000000))
                                .child(before_disp),
                        )
                        .child(
                            div()
                                .w(px(1.5))
                                .h(px(font_size + 1.0))
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

            div()
                .id("section-name-editor")
                .relative()
                .flex()
                .items_center()
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                        this.active_field = ActiveField::SectionName;
                        this.focus_handle.focus(window, cx);
                        let rel_x =
                            (event.position.x.as_f32() - this.active_section_tab_x).max(0.0);
                        let click_idx = calculate_line_text_offset_with_font(
                            rel_x,
                            &this.edit_section_name,
                            this.section_name_font_size,
                            WEIGHT_BOLD,
                            this.font_type(),
                        );
                        this.edit_section_name_cursor = click_idx;
                        this.edit_section_name_anchor = Some(click_idx);
                        this.is_selecting_section_name = true;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                    if this.is_selecting_section_name {
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
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        if this.is_selecting_section_name {
                            if this.edit_section_name_anchor == Some(this.edit_section_name_cursor)
                            {
                                this.edit_section_name_anchor = None;
                            }
                            this.is_selecting_section_name = false;
                            cx.notify();
                            cx.stop_propagation();
                        }
                    }),
                )
                .children(els)
                .into_any_element()
        }
    }
}
