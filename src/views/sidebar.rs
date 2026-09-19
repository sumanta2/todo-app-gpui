use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::constants::{layout::NOTE_ITEM_TEXT_OFFSET_X, typography::WEIGHT_SEMIBOLD};
use crate::models::ActiveField;
use crate::text_selection::{calculate_line_text_offset_weighted, get_selection_range};

impl NotesApp {
    /// Renders the left sidebar containing note navigation and note-creation actions.
    ///
    /// The sidebar keeps the list of notes and the current notebook title editor in sync with
    /// the app state while allowing the user to switch between notes quickly.
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
                                                .text_size(px(self.note_heading_font_size))
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(rgb(0xe0e0e0))
                                                .child(if self.is_editing && is_selected {
                                                    let is_focused = is_note_heading_focused;
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
                                                                    this.active_field =
                                                                        ActiveField::NoteHeading;
                                                                    this.focus_handle.focus(window, cx);
                                                                    this.edit_note_heading_cursor = 0;
                                                                    this.edit_note_heading_anchor =
                                                                        None;
                                                                    this.cursor_visible = true;
                                                                    cx.notify();
                                                                    cx.stop_propagation();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(rgb(0x808080))
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
                                                                        rgb(0x0078d4)
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
                                                        ).unwrap_or((self.edit_note_heading_cursor, self.edit_note_heading_cursor));

                                                        let has_selection = sel_start < sel_end;
                                                        let text = &self.edit_note_heading;
                                                        let chars: Vec<char> = text.chars().collect();

                                                        let mut els: Vec<AnyElement> = Vec::new();
                                                        if has_selection {
                                                            let prefix: String = chars[..sel_start.min(chars.len())].iter().collect();
                                                            let sel_chunk: String = chars[sel_start.min(chars.len())..sel_end.min(chars.len())].iter().collect();
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
                                                                    .child(
                                                                        div()
                                                                            .text_color(rgba(0x00000000))
                                                                            .child(before_disp),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .bg(rgb(0x264f78))
                                                                            .rounded(px(2.0))
                                                                            .h(px(font_size + 2.0))
                                                                            .flex()
                                                                            .items_center()
                                                                            .child(
                                                                                div()
                                                                                    .text_color(rgba(0x00000000))
                                                                                    .child(sel_disp),
                                                                            ),
                                                                    )
                                                                    .into_any_element(),
                                                            );
                                                        }

                                                        els.push(
                                                            div()
                                                                .child(if text.is_empty() { "\u{00A0}".to_string() } else { text.replace(' ', "\u{00A0}") })
                                                                .into_any_element(),
                                                        );

                                                        if is_focused && !has_selection {
                                                            let prefix: String = chars[..self.edit_note_heading_cursor.min(chars.len())].iter().collect();
                                                            let before_disp = prefix.replace(' ', "\u{00A0}");
                                                            els.push(
                                                                div()
                                                                    .absolute()
                                                                    .left(px(0.0))
                                                                    .top(px(2.0))
                                                                    .flex()
                                                                    .flex_row()
                                                                    .items_center()
                                                                    .child(
                                                                        div()
                                                                            .text_color(rgba(0x00000000))
                                                                            .child(before_disp),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .w(px(1.5))
                                                                            .h(px(font_size))
                                                                            .bg(if self.cursor_visible {
                                                                                rgb(0x0078d4)
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
                                                                    this.active_field =
                                                                        ActiveField::NoteHeading;
                                                                    this.focus_handle.focus(window, cx);
                                                                    let rel_x = (event.position.x.as_f32() - NOTE_ITEM_TEXT_OFFSET_X).max(0.0);
                                                                    let click_idx = calculate_line_text_offset_weighted(
                                                                        rel_x,
                                                                        &this.edit_note_heading,
                                                                        this.note_heading_font_size,
                                                                        WEIGHT_SEMIBOLD,
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
                                                                    let rel_x = (event.position.x.as_f32() - NOTE_ITEM_TEXT_OFFSET_X).max(0.0);
                                                                    let drag_idx = calculate_line_text_offset_weighted(
                                                                        rel_x,
                                                                        &this.edit_note_heading,
                                                                        this.note_heading_font_size,
                                                                        WEIGHT_SEMIBOLD,
                                                                    );
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
