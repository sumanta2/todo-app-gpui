//! Notebook name field inside the selected row of the left list.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, IntoElement, MouseButton};

use crate::app::NotesApp;
use crate::constants::{
    colors::{cursor, sidebar_item_text, sidebar_on_selected},
    layout::NOTE_ITEM_TEXT_OFFSET_X,
    typography::WEIGHT_SEMIBOLD,
};
use crate::models::ActiveField;
use crate::text::selection::{calculate_line_text_offset_with_font, get_selection_range};

impl NotesApp {
    /// Name editor for the selected notebook, or the plain title when that row is not being renamed.
    pub(super) fn render_notebook_heading(
        &self,
        heading: &str,
        is_selected: bool,
        is_focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.is_editing && is_selected {
            let font_size = self.note_heading_font_size;
            let editor = if self.edit_note_heading.is_empty() {
                div()
                    .id("note-heading-placeholder")
                    .relative()
                    .flex()
                    .items_center()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.active_field = ActiveField::NoteHeading;
                            this.focus_handle.focus(window, cx);
                            this.edit_note_heading_cursor = 0;
                            this.edit_note_heading_anchor = None;
                            this.cursor_visible = true;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child(
                        div()
                            .font_family("Lato")
                            .text_color(rgb(if is_selected {
                                sidebar_on_selected()
                            } else {
                                sidebar_item_text()
                            }))
                            .child("Notebook Name..."),
                    )
                    .child(if is_focused {
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(2.0))
                            .w(px(1.5))
                            .h(px(font_size))
                            .bg(if self.cursor_visible {
                                rgb(cursor())
                            } else {
                                rgba(0x00000000)
                            })
                    } else {
                        div()
                    })
            } else {
                let (sel_start, sel_end) = get_selection_range(
                    self.edit_note_heading_cursor,
                    self.edit_note_heading_anchor,
                )
                .unwrap_or((self.edit_note_heading_cursor, self.edit_note_heading_cursor));

                let has_selection = sel_start < sel_end;
                let text = &self.edit_note_heading;
                let chars: Vec<char> = text.chars().collect();

                let mut els: Vec<AnyElement> = Vec::new();
                if has_selection {
                    let prefix: String = chars[..sel_start.min(chars.len())].iter().collect();
                    let sel_chunk: String = chars
                        [sel_start.min(chars.len())..sel_end.min(chars.len())]
                        .iter()
                        .collect();
                    let before_disp = prefix.replace(' ', "\u{00A0}");
                    let sel_disp = sel_chunk.replace(' ', "\u{00A0}");
                    els.push(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(1.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(div().text_color(rgba(0x00000000)).child(before_disp))
                            .child(
                                div()
                                    .bg(rgb(crate::constants::colors::note_selection()))
                                    .rounded(px(2.0))
                                    .h(px(font_size + 2.0))
                                    .flex()
                                    .items_center()
                                    .child(div().text_color(rgba(0x00000000)).child(sel_disp)),
                            )
                            .into_any_element(),
                    );
                }

                els.push(
                    div()
                        .child(if text.is_empty() {
                            "\u{00A0}".to_string()
                        } else {
                            text.replace(' ', "\u{00A0}")
                        })
                        .into_any_element(),
                );

                if is_focused && !has_selection {
                    let prefix: String = chars[..self.edit_note_heading_cursor.min(chars.len())]
                        .iter()
                        .collect();
                    let before_disp = prefix.replace(' ', "\u{00A0}");
                    els.push(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(2.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(div().text_color(rgba(0x00000000)).child(before_disp))
                            .child(
                                div()
                                    .w(px(1.5))
                                    .h(px(font_size))
                                    .bg(if self.cursor_visible {
                                        rgb(cursor())
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
                    .id("note-heading-editor")
                    .relative()
                    .flex()
                    .items_center()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                            this.active_field = ActiveField::NoteHeading;
                            this.focus_handle.focus(window, cx);
                            let rel_x =
                                (event.position.x.as_f32() - NOTE_ITEM_TEXT_OFFSET_X).max(0.0);
                            let click_idx = calculate_line_text_offset_with_font(
                                rel_x,
                                &this.edit_note_heading,
                                this.note_heading_font_size,
                                WEIGHT_SEMIBOLD,
                                this.font_type(),
                            );
                            this.edit_note_heading_cursor = click_idx;
                            this.edit_note_heading_anchor = Some(click_idx);
                            this.is_selecting_note_heading = true;
                            this.cursor_visible = true;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                        if this.is_selecting_note_heading {
                            let rel_x =
                                (event.position.x.as_f32() - NOTE_ITEM_TEXT_OFFSET_X).max(0.0);
                            let drag_idx = calculate_line_text_offset_with_font(
                                rel_x,
                                &this.edit_note_heading,
                                this.note_heading_font_size,
                                WEIGHT_SEMIBOLD,
                                this.font_type(),
                            );
                            if crate::text::selection::assign_if_changed(
                                &mut this.edit_note_heading_cursor,
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
                            if this.is_selecting_note_heading {
                                if this.edit_note_heading_anchor
                                    == Some(this.edit_note_heading_cursor)
                                {
                                    this.edit_note_heading_anchor = None;
                                }
                                this.is_selecting_note_heading = false;
                                cx.notify();
                                cx.stop_propagation();
                            }
                        }),
                    )
                    .children(els)
            };
            editor.into_any_element()
        } else {
            div()
                .child(if heading.trim().is_empty() {
                    "Untitled Note".to_owned()
                } else {
                    heading.to_owned()
                })
                .into_any_element()
        }
    }
}
