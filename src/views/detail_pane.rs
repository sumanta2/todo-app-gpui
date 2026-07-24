use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, Window};

use crate::app::NotesApp;
use crate::helpers::hash_str;
use crate::models::load_note_content;

impl NotesApp {
    pub(crate) fn render_detail_pane(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected_note = self
            .notes
            .iter()
            .find(|n| Some(n.id.clone()) == self.selected_note_id)
            .cloned();

        if let Some(note) = selected_note {
            let content = if self.is_editing {
                self.edit_content
                    .clone()
                    .unwrap_or_else(|| load_note_content(&note.body, &note.images))
            } else {
                load_note_content(&note.body, &note.images)
            };

            // Vertical Page sidebar (on the right)
            let page_sidebar = {
                let active_sec = content
                    .sections
                    .iter()
                    .find(|s| Some(&s.id) == self.active_section_id.as_ref());
                let pages = active_sec.map(|s| &s.pages).cloned().unwrap_or_default();

                let mut page_elements = Vec::new();
                for page in &pages {
                    let page_id = page.id.clone();
                    let is_active = Some(&page_id) == self.active_page_id.as_ref();
                    let click_id = page_id.clone();
                    let delete_id = page_id.clone();
                    let active_sec_id = self.active_section_id.clone().unwrap_or_default();

                    let mut page_row = div()
                        .id(("page-row", hash_str(&page_id)))
                        .px(px(4.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .cursor_pointer();
                    if is_active {
                        page_row = page_row.bg(rgb(0x2d2d2d));
                    }
                    page_row = page_row
                        .hover(|s| s.bg(rgb(0x242424)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.switch_to_page(active_sec_id.clone(), click_id.clone(), cx);
                        }))
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(if is_active {
                                    rgb(0xffffff)
                                } else {
                                    rgb(0xd4d4d4)
                                })
                                .font_weight(if is_active {
                                    gpui::FontWeight::BOLD
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .child(if page.name.trim().is_empty() {
                                    "Untitled Page".to_owned()
                                } else {
                                    page.name.clone()
                                }),
                        );

                    if self.is_editing && pages.len() > 1 {
                        page_row = page_row.child(
                            div()
                                .id(("delete-page", hash_str(&page_id)))
                                .text_color(rgb(0xff6b6b))
                                .hover(|s| s.text_color(rgb(0xff0000)))
                                .child("×")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.delete_page(delete_id.clone(), cx);
                                    cx.stop_propagation();
                                })),
                        );
                    }

                    page_elements.push(page_row.into_any_element());
                }

                let add_page_btn = if self.is_editing {
                    Some(
                        div()
                            .id("add-page-btn")
                            .mb(px(6.0))
                            .px(px(8.0))
                            .py(px(4.0))
                            .bg(rgb(0x0078d4))
                            .hover(|s| s.bg(rgb(0x106ebe)))
                            .text_color(rgb(0xffffff))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.add_page(cx);
                            }))
                            .child("+ Add Page"),
                    )
                } else {
                    None
                };

                div()
                    .w(px(180.0))
                    .h_full()
                    .bg(rgb(0x1a1a1a))
                    .border_l_1()
                    .border_color(rgb(0x2d2d2d))
                    .p(px(6.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(0x808080))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("PAGES"),
                    )
                    .children(add_page_btn)
                    .child(
                        div()
                            .id("pages-list-scroller")
                            .flex_1()
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .children(page_elements),
                    )
            };

            if self.is_editing {
                self.render_canvas_editor(
                    &note,
                    &content,
                    page_sidebar.into_any_element(),
                    window,
                    cx,
                )
                .into_any_element()
            } else {
                self.render_canvas_viewer(
                    &note,
                    &content,
                    page_sidebar.into_any_element(),
                    window,
                    cx,
                )
                .into_any_element()
            }
        } else {
            // NO NOTE SELECTED PLACEHOLDER
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .flex_1()
                .h_full()
                .bg(rgb(0x1e1e1e))
                .child(
                    div()
                        .text_size(px(16.0))
                        .text_color(rgb(0x808080))
                        .child("Select a note or create a new one to begin"),
                )
                .into_any_element()
        }
    }
}
