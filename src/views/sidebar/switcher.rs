//! Notebook switcher shown beside the section tabs when the left list is unpinned.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, MouseButton};

use crate::app::NotesApp;
use crate::constants::colors::{
    SIDEBAR_BG, SIDEBAR_HOVER, SIDEBAR_ITEM, SIDEBAR_ITEM_TEXT, SIDEBAR_SELECTED,
};
use crate::models::ActiveField;

impl NotesApp {
    /// Note name, list, and pin control shown before the section tabs when the left bar is unpinned.
    pub(crate) fn render_note_switcher(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.is_sidebar_open {
            return div().into_any_element();
        }
        let open = self.note_menu_open;
        let label = self.current_note_label();
        let editing_name = self.is_editing && self.active_field == ActiveField::NoteHeading;
        div()
            .id("note-switcher")
            .relative()
            .h(px(28.0))
            .px(px(6.0))
            .rounded(px(4.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .bg(if open {
                rgb(SIDEBAR_HOVER)
            } else {
                rgb(SIDEBAR_ITEM)
            })
            .child(
                div()
                    .font_family("Segoe MDL2 Assets")
                    .text_size(px(14.0))
                    .text_color(rgb(SIDEBAR_ITEM_TEXT))
                    .child("\u{E8F1}"),
            )
            .child(
                div()
                    .id("note-switcher-name")
                    .max_w(px(160.0))
                    .cursor_text()
                    .text_size(px(self.section_name_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(SIDEBAR_ITEM_TEXT))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            if !this.is_editing {
                                this.start_edit(cx);
                            }
                            this.active_field = ActiveField::NoteHeading;
                            this.focus_handle.focus(window, cx);
                            this.cursor_visible = true;
                            this.note_menu_open = false;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child(if editing_name {
                        format!("{label}|")
                    } else {
                        label
                    }),
            )
            .child(
                div()
                    .id("note-switcher-chevron")
                    .cursor_pointer()
                    .px(px(2.0))
                    .text_size(px(10.0))
                    .text_color(rgb(SIDEBAR_ITEM_TEXT))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.note_menu_open = !this.note_menu_open;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child(if open { "▴" } else { "▾" }),
            )
            .child(
                div()
                    .id("note-switcher-pin")
                    .cursor_pointer()
                    .w(px(22.0))
                    .h(px(22.0))
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|style| style.bg(rgb(SIDEBAR_HOVER)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.toggle_sidebar(cx);
                            cx.stop_propagation();
                        }),
                    )
                    .child(
                        div()
                            .font_family("Segoe MDL2 Assets")
                            .text_size(px(14.0))
                            .text_color(rgb(SIDEBAR_ITEM_TEXT))
                            .child("\u{E718}"),
                    ),
            )
            .into_any_element()
    }

    /// Note list painted over the canvas when the unpinned switcher is open.
    pub(crate) fn render_note_menu_layers(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        if self.is_sidebar_open || self.full_page_view || !self.note_menu_open {
            return Vec::new();
        }
        let mut list = div()
            .absolute()
            .top(px(72.0))
            .left(px(8.0))
            .w(px(220.0))
            .p(px(4.0))
            .bg(rgb(SIDEBAR_BG))
            .border_1()
            .border_color(rgb(SIDEBAR_ITEM_TEXT))
            .rounded(px(6.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            });
        for (index, note) in self.notes.iter().enumerate() {
            let note_id = note.id.clone();
            let selected = self.selected_note_id.as_ref() == Some(&note.id);
            let name = if note.heading.trim().is_empty() {
                "Untitled Note".to_owned()
            } else {
                note.heading.clone()
            };
            list = list.child(
                div()
                    .id(("note-menu-item", index))
                    .h(px(28.0))
                    .px(px(8.0))
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .bg(if selected {
                        rgb(SIDEBAR_SELECTED)
                    } else {
                        rgb(SIDEBAR_ITEM)
                    })
                    .hover(|style| {
                        style.bg(rgb(if selected {
                            SIDEBAR_SELECTED
                        } else {
                            SIDEBAR_HOVER
                        }))
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.select_note(note_id.clone(), cx);
                            cx.stop_propagation();
                        }),
                    )
                    .child(
                        div()
                            .text_size(px(self.section_name_font_size))
                            .font_family("Lato")
                            .text_color(rgb(SIDEBAR_ITEM_TEXT))
                            .child(name),
                    ),
            );
        }
        vec![
            div()
                .absolute()
                .top(px(36.0))
                .left(px(0.0))
                .right(px(0.0))
                .bottom(px(0.0))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.note_menu_open = false;
                        cx.notify();
                    }),
                )
                .into_any_element(),
            list.into_any_element(),
        ]
    }

    fn current_note_label(&self) -> String {
        if self.is_editing {
            if self.edit_note_heading.trim().is_empty() {
                "Untitled Note".to_owned()
            } else {
                self.edit_note_heading.clone()
            }
        } else {
            self.selected_note()
                .map(|note| {
                    if note.heading.trim().is_empty() {
                        "Untitled Note".to_owned()
                    } else {
                        note.heading.clone()
                    }
                })
                .unwrap_or_else(|| "No note".to_owned())
        }
    }
}
