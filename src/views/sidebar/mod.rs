//! Notebook list pinned on the left.

mod heading;
mod switcher;

use gpui::{div, prelude::*, px, rgb, Context, IntoElement, Window};

use crate::app::NotesApp;
use crate::constants::colors::{
    onenote_bar_line, sidebar_bg, sidebar_create, sidebar_create_hover, sidebar_hover, sidebar_item,
    sidebar_item_text, sidebar_on_selected, sidebar_selected,
};
use crate::models::ActiveField;

impl NotesApp {
    pub(crate) fn render_sidebar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_note_heading_focused =
            self.focus_handle.is_focused(window) && self.active_field == ActiveField::NoteHeading;

        let expanded = self.sidebar_expanded();
        let mut sidebar = div()
            .w(if expanded { px(220.0) } else { px(44.0) })
            .h_full()
            .bg(rgb(sidebar_bg()))
            .border_r_1()
            .border_color(rgb(onenote_bar_line()))
            .flex()
            .flex_col()
            .pt(px(if expanded { 4.0 } else { 8.0 }))
            .pb(px(if expanded { 4.0 } else { 8.0 }))
            .pr(px(if expanded { 4.0 } else { 0.0 }))
            .pl(px(0.0))
            .gap(px(4.0));

        if expanded {
            sidebar = sidebar
                .child(
                    div()
                        .id("new-note-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(rgb(sidebar_create()))
                        .hover(|s| s.bg(rgb(sidebar_create_hover())))
                        .text_color(rgb(sidebar_on_selected()))
                        .font_family("Lato")
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.create_note(cx);
                        }))
                        .child("+ Create New Note"),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .pl(px(4.0))
                        .child(
                            div()
                                .font_family("Lato")
                                .text_size(px(16.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(sidebar_item_text()))
                                .child("All Notes"),
                        )
                        .child(self.render_sidebar_pin(cx)),
                )
                .child(
                    div()
                        .id("notes-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .gap(px(0.0))
                        .children(
                            self.notes
                                .iter()
                                .enumerate()
                                .filter(|(_, note)| {
                                    let query = self.title_search.trim();
                                    query.is_empty()
                                        || note
                                            .heading
                                            .to_lowercase()
                                            .contains(&query.to_lowercase())
                                })
                                .map(|(index, note)| {
                                    let note_id = note.id.clone();
                                    let is_selected =
                                        self.selected_note_id == Some(note_id.clone());
                                    let item_click_id = note_id.clone();

                                    let mut item = div()
                                        .id(("note-item", index))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(8.0))
                                        .py(px(8.0))
                                        .min_h(px(48.0))
                                        .cursor_pointer()
                                        .bg(if is_selected {
                                            rgb(sidebar_selected())
                                        } else {
                                            rgb(sidebar_item())
                                        });
                                    item = if is_selected {
                                        item.border_1().border_color(rgb(onenote_bar_line()))
                                    } else {
                                        item.border_b_1().border_color(rgb(onenote_bar_line()))
                                    };
                                    item.hover(|s| {
                                        if is_selected {
                                            s
                                        } else {
                                            s.bg(rgb(sidebar_hover()))
                                        }
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.select_note(item_click_id.clone(), cx);
                                    }))
                                    .child(
                                        div().flex().flex_col().items_start().child(
                                            div()
                                                .text_size(px(self.note_heading_font_size))
                                                .font_family("Lato")
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(rgb(if is_selected {
                                                    sidebar_on_selected()
                                                } else {
                                                    sidebar_item_text()
                                                }))
                                                .child(self.render_notebook_heading(
                                                    &note.heading,
                                                    is_selected,
                                                    is_note_heading_focused,
                                                    cx,
                                                )),
                                        ),
                                    )
                                }),
                        ),
                );
        } else {
            sidebar = sidebar.child(self.render_sidebar_pin(cx));
        }

        sidebar
    }

    /// Unpin control. On the All Notes row when the list is open, and alone when it is collapsed.
    fn render_sidebar_pin(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        div()
            .id("toggle-sidebar-btn")
            .flex()
            .items_center()
            .justify_center()
            .w(px(28.0))
            .h(px(28.0))
            .rounded(px(4.0))
            .hover(|s| s.bg(rgb(sidebar_hover())))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.toggle_sidebar(cx);
            }))
            .child(
                div()
                    .font_family("Segoe MDL2 Assets")
                    .text_size(px(14.0))
                    .text_color(rgb(sidebar_item_text()))
                    .child("\u{E77A}"),
            )
            .into_any_element()
    }
}
