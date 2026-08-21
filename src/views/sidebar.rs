use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::helpers::{calculate_line_text_offset, get_selection_range};
use crate::models::ActiveField;

impl NotesApp {
    pub(crate) fn render_sidebar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_note_heading_focused =
            self.focus_handle.is_focused(window) && self.active_field == ActiveField::NoteHeading;

        let mut sidebar = div()
            .w(if self.is_sidebar_open {
                px(220.0)
            } else {
                px(44.0)
            })
            .h_full()
            .bg(rgb(0x181818))
            .border_r_1()
            .border_color(rgb(0x2d2d2d))
            .flex()
            .flex_col()
            .p(px(8.0))
            .gap(px(10.0));

        // Top Header: Toggle Icon Button (No "My Notes" text)
        sidebar = sidebar.child(
            div().flex().items_center().justify_between().child(
                div()
                    .id("toggle-sidebar-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded(px(4.0))
                    .hover(|s| s.bg(rgb(0x2d2d2d)))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_sidebar(cx);
                    }))
                    .child("☰"),
            ),
        );

        if self.is_sidebar_open {
            sidebar = sidebar
                .child(
                    div()
                        .id("new-note-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(32.0))
                        .bg(rgb(0x0078d4))
                        .hover(|s| s.bg(rgb(0x106ebe)))
                        .text_color(rgb(0xffffff))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.create_note(cx);
                        }))
                        .child("+ New Note"),
                )
                .child(
                    div()
                        .id("notes-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .children(self.notes.iter().enumerate().map(|(index, note)| {
                            let note_id = note.id.clone();
                            let is_selected = self.selected_note_id == Some(note_id.clone());
                            let item_click_id = note_id.clone();

                            div()
                                .id(("note-item", index))
                                .flex()
                                .items_center()
                                .justify_between()
                                .p(px(8.0))
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .bg(if is_selected {
                                    rgb(0x2d2d2d)
                                } else {
                                    rgb(0x181818)
                                })
                                .border_l_2()
                                .border_color(if is_selected {
                                    rgb(0x0078d4)
                                } else {
                                    rgb(0x181818)
                                })
                                .hover(|s| s.bg(rgb(0x242424)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.selected_note_id == Some(item_click_id.clone()) {
                                        return;
                                    }
                                    this.selected_note_id = Some(item_click_id.clone());
                                    this.is_editing = false;
                                    this.active_section_id = None;
                                    this.active_page_id = None;
                                    this.initialize_active_section_page();
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_start()
                                        .child(
                                            div()
                                                .text_size(px(14.0))
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(rgb(0xe0e0e0))
                                                .child(if self.is_editing && is_selected {
                                                    let is_focused = is_note_heading_focused;
                                                    let editor = if self.edit_note_heading.is_empty() {
                                                        div()
                                                            .id("note-heading-placeholder")
                                                            .flex()
                                                            .items_center()
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, window, cx| {
                                                                    this.active_field =
                                                                        ActiveField::NoteHeading;
                                                                    this.focus_handle.focus(window, cx);
                                                                    this.edit_note_heading_cursor = 0;
                                                                    this.edit_note_heading_anchor =
                                                                        None;
                                                                    cx.notify();
                                                                    cx.stop_propagation();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(rgb(0x808080))
                                                                    .child("Notebook Name..."),
                                                            )
                                                    } else {
                                                        let mut els: Vec<AnyElement> = Vec::new();
                                                        let char_count =
                                                            self.edit_note_heading.chars().count();
                                                        let (sel_start, sel_end) = get_selection_range(
                                                            self.edit_note_heading_cursor,
                                                            self.edit_note_heading_anchor,
                                                        ).unwrap_or((self.edit_note_heading_cursor, self.edit_note_heading_cursor));

                                                        for i in 0..=char_count {
                                                            if is_focused
                                                                && ((sel_start == sel_end && i == sel_start)
                                                                    || (sel_start < sel_end
                                                                        && i >= sel_start
                                                                        && i < sel_end))
                                                            {
                                                                let char_str = self
                                                                    .edit_note_heading
                                                                    .chars()
                                                                    .nth(i)
                                                                    .map(|c| c.to_string())
                                                                    .unwrap_or_else(|| {
                                                                        " ".to_string()
                                                                    });
                                                                els.push(
                                                                    div()
                                                                        .bg(if sel_start < sel_end {
                                                                            rgb(0x264f78)
                                                                        } else {
                                                                            rgb(0x0078d4)
                                                                        })
                                                                        .text_color(rgb(0xffffff))
                                                                        .child(char_str)
                                                                        .into_any_element(),
                                                                );
                                                            } else if i < char_count {
                                                                let char_str = self
                                                                    .edit_note_heading
                                                                    .chars()
                                                                    .nth(i)
                                                                    .unwrap()
                                                                    .to_string();
                                                                els.push(
                                                                    div()
                                                                        .child(char_str)
                                                                        .into_any_element(),
                                                                );
                                                            }
                                                        }

                                                        div()
                                                            .id("note-heading-editor")
                                                            .flex()
                                                            .items_center()
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                                                                    this.active_field =
                                                                        ActiveField::NoteHeading;
                                                                    this.focus_handle.focus(window, cx);
                                                                    let rel_x = (event.position.x.as_f32() - 16.0).max(0.0);
                                                                    let click_idx = calculate_line_text_offset(rel_x, &this.edit_note_heading, 13.0);
                                                                    this.edit_note_heading_cursor = click_idx;
                                                                    this.edit_note_heading_anchor = Some(click_idx);
                                                                    this.is_selecting_note_heading = true;
                                                                    cx.notify();
                                                                    cx.stop_propagation();
                                                                }),
                                                            )
                                                            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                                                                if this.is_selecting_note_heading {
                                                                    let rel_x = (event.position.x.as_f32() - 16.0).max(0.0);
                                                                    let drag_idx = calculate_line_text_offset(rel_x, &this.edit_note_heading, 13.0);
                                                                    this.edit_note_heading_cursor = drag_idx;
                                                                    cx.notify();
                                                                    cx.stop_propagation();
                                                                }
                                                            }))
                                                            .on_mouse_up(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, _, cx| {
                                                                    if this.is_selecting_note_heading {
                                                                        if this.edit_note_heading_anchor == Some(this.edit_note_heading_cursor) {
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
                                                        .child(if note.heading.trim().is_empty() {
                                                            "Untitled Note".to_owned()
                                                        } else {
                                                            note.heading.clone()
                                                        })
                                                        .into_any_element()
                                                }),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .text_color(rgb(0x808080))
                                                .child(note.created_at.clone()),
                                        ),
                                )
                        })),
                );
        }

        sidebar
    }
}
