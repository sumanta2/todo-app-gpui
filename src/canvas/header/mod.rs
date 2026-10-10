//! Section tab strip above the open page.

mod heading;
mod page_title;
mod section_name;

use gpui::{div, prelude::*, px, rgb, Context, IntoElement};

use crate::app::NotesApp;
use crate::constants::typography::{
    BUTTON_FONT_SIZE, SMALL_ICON_FONT_SIZE, WEIGHT_BOLD, WEIGHT_NORMAL,
};
use crate::helpers::hash_str;
use crate::models::NoteContent;
use crate::text::selection::calculate_text_width_for_font;

impl NotesApp {
    pub(crate) fn build_section_tabs(
        &mut self,
        content: &NoteContent,
        is_section_name_focused: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut current_tab_x = self.layout_sidebar_w();
        let mut active_tab_x = current_tab_x + 6.0;
        let mut tabs = Vec::new();

        for sec in &content.sections {
            let sec_id = sec.id.clone();
            let is_active = Some(&sec_id) == self.active_section_id.as_ref();
            let click_id = sec_id.clone();
            let delete_id = sec_id.clone();

            if is_active {
                active_tab_x = current_tab_x + 6.0;
            }

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
                });
            tab_el = if is_active {
                tab_el.cursor_text()
            } else {
                tab_el
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(crate::constants::colors::onenote_tab_hover())))
            };
            tab_el = tab_el
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.switch_to_section(click_id.clone(), cx);
                }))
                .flex()
                .items_center()
                .gap(px(6.0));

            if is_active {
                tab_el = tab_el.child(self.build_section_name_editor(is_section_name_focused, cx));
            } else {
                tab_el = tab_el.child(sec.name.clone());
            }

            if content.sections.len() > 1 {
                tab_el = tab_el.child(
                    div()
                        .id(("delete-sec", hash_str(&sec_id)))
                        .text_color(rgb(0xff6b6b))
                        .hover(|s| s.text_color(rgb(0xff0000)))
                        .cursor_pointer()
                        .text_size(px(SMALL_ICON_FONT_SIZE))
                        .child("×")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.delete_section(delete_id.clone(), cx);
                            cx.stop_propagation();
                        })),
                );
            }

            tabs.push(tab_el.into_any_element());

            // Track horizontal offset for following tabs
            let tab_text = if is_active {
                if self.edit_section_name.is_empty() {
                    "Section Name..."
                } else {
                    &self.edit_section_name
                }
            } else {
                &sec.name
            };
            let weight = if is_active {
                WEIGHT_BOLD
            } else {
                WEIGHT_NORMAL
            };
            let text_w = calculate_text_width_for_font(
                tab_text,
                self.section_name_font_size,
                weight,
                self.font_type(),
            );
            let mut tab_w = 12.0 + text_w;
            if content.sections.len() > 1 {
                tab_w += 18.0;
            }
            current_tab_x += tab_w + 4.0;
        }

        self.active_section_tab_x = active_tab_x;

        tabs.push(
            div()
                .id("add-section-btn")
                .px(px(8.0))
                .py(px(4.0))
                .font_family("Calibri")
                .text_size(px(self.section_name_font_size + 3.0))
                .bg(rgb(crate::constants::colors::onenote_tab_idle()))
                .hover(|s| s.bg(rgb(crate::constants::colors::onenote_tab_hover())))
                .text_color(rgb(0x8a805a))
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.add_section(cx);
                }))
                .child("+")
                .into_any_element(),
        );

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
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(id) = this.selected_note_id.clone() {
                        this.delete_note(id, cx);
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
    }
}
