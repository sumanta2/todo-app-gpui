//! Section tab strip above the open page.

mod heading;
mod page_title;
mod section_name;

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, FontWeight, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::constants::typography::BUTTON_FONT_SIZE;
use crate::helpers::hash_str;
use crate::models::NoteContent;
use crate::text::shaping::{shaped_uniform_prefix, UniformFace};

impl NotesApp {
    pub(crate) fn build_section_tabs(
        &mut self,
        content: &NoteContent,
        is_section_name_focused: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut current_tab_x = self.layout_sidebar_w();
        let mut active_tab_x = current_tab_x + 6.0;
        let mut tabs = Vec::new();

        for sec in &content.sections {
            let sec_id = sec.id.clone();
            let is_active = Some(&sec_id) == self.active_section_id.as_ref();
            let click_id = sec_id.clone();

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
            let menu_id = click_id.clone();
            tab_el = tab_el
                .cursor_pointer()
                .hover(|style| style.bg(rgb(crate::constants::colors::onenote_tab_hover())))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if !this.section_renaming
                            && this.active_section_id.as_deref() != Some(click_id.as_str())
                        {
                            this.switch_to_section(click_id.clone(), cx);
                        }
                        this.section_menu_at = None;
                        this.page_menu_at = None;
                        cx.stop_propagation();
                        cx.notify();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                        if this.active_section_id.as_deref() != Some(menu_id.as_str()) {
                            this.switch_to_section(menu_id.clone(), cx);
                        }
                        this.page_menu_at = None;
                        if this.is_editing {
                            this.page_renaming = false;
                            this.section_renaming = false;
                            this.section_menu_at =
                                Some((event.position.x.as_f32(), event.position.y.as_f32()));
                        }
                        cx.stop_propagation();
                        cx.notify();
                    }),
                )
                .flex()
                .items_center()
                .gap(px(6.0));

            if is_active && self.section_renaming {
                tab_el = tab_el.child(self.build_section_name_editor(is_section_name_focused, cx));
            } else {
                tab_el = tab_el.child(sec.name.clone());
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
            let text_w = shaped_uniform_prefix(
                window,
                tab_text,
                UniformFace {
                    family: "Calibri",
                    size: self.section_name_font_size,
                    weight: if is_active {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    },
                    italic: false,
                },
                1.0,
            )
            .last()
            .copied()
            .unwrap_or(0.0);
            current_tab_x += 12.0 + text_w + 4.0;
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

    /// Rename and Delete menus for the section tab and the page name.
    pub(crate) fn render_name_action_menus(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut layers = Vec::new();
        let section_count = self
            .edit_content
            .as_ref()
            .map(|content| content.sections.len())
            .unwrap_or(0);
        let page_count = self
            .edit_content
            .as_ref()
            .and_then(|content| {
                content
                    .sections
                    .iter()
                    .find(|section| self.active_section_id.as_ref() == Some(&section.id))
            })
            .map(|section| section.pages.len())
            .unwrap_or(0);
        if let Some((x, y)) = self.section_menu_at {
            let section_id = self.active_section_id.clone().unwrap_or_default();
            layers.push(name_action_menu(
                "section-name-menu",
                x,
                y,
                section_count > 1,
                cx,
                move |this, cx| {
                    this.section_menu_at = None;
                    this.section_renaming = true;
                    this.active_field = crate::models::ActiveField::SectionName;
                    this.edit_section_name_cursor = this.edit_section_name.chars().count();
                    this.edit_section_name_anchor = None;
                    this.cursor_visible = true;
                    cx.notify();
                },
                move |this, cx| {
                    this.section_menu_at = None;
                    this.delete_section(section_id.clone(), cx);
                },
            ));
        }
        if let Some((x, y)) = self.page_menu_at {
            let page_id = self.active_page_id.clone().unwrap_or_default();
            layers.push(name_action_menu(
                "page-name-menu",
                x,
                y,
                page_count > 1,
                cx,
                move |this, cx| {
                    this.page_menu_at = None;
                    this.page_renaming = true;
                    this.active_field = crate::models::ActiveField::Heading;
                    this.edit_heading_cursor = this.edit_heading.chars().count();
                    this.edit_heading_anchor = None;
                    this.cursor_visible = true;
                    cx.notify();
                },
                move |this, cx| {
                    this.page_menu_at = None;
                    this.delete_page(page_id.clone(), cx);
                },
            ));
        }
        layers
    }
}

fn name_action_menu(
    id: &'static str,
    x: f32,
    y: f32,
    delete_enabled: bool,
    cx: &mut Context<NotesApp>,
    rename: impl Fn(&mut NotesApp, &mut Context<NotesApp>) + 'static,
    delete: impl Fn(&mut NotesApp, &mut Context<NotesApp>) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .absolute()
        .left(px(x))
        .top(px(y + 4.0))
        .w(px(132.0))
        .p(px(4.0))
        .bg(rgb(crate::constants::colors::note_page()))
        .border_1()
        .border_color(rgb(crate::constants::colors::onenote_bar_line()))
        .rounded(px(2.0))
        .flex()
        .flex_col()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(name_action_row(
            if id.starts_with("section") {
                "section-rename"
            } else {
                "page-rename"
            },
            "\u{E70F}",
            "Rename",
            true,
            cx,
            rename,
        ))
        .child(name_action_row(
            if id.starts_with("section") {
                "section-delete"
            } else {
                "page-delete"
            },
            "\u{E711}",
            "Delete",
            delete_enabled,
            cx,
            delete,
        ))
        .into_any_element()
}

fn name_action_row(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    enabled: bool,
    cx: &mut Context<NotesApp>,
    action: impl Fn(&mut NotesApp, &mut Context<NotesApp>) + 'static,
) -> AnyElement {
    let mut chars = label.chars();
    let first = chars.next().map(|ch| ch.to_string()).unwrap_or_default();
    let rest: String = chars.collect();
    let ink = if enabled {
        crate::constants::colors::onenote_ink()
    } else {
        crate::constants::colors::onenote_ink_muted()
    };
    let mut row = div()
        .id(id)
        .h(px(28.0))
        .px(px(8.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0));
    if enabled {
        row = row
            .cursor_pointer()
            .hover(|style| style.bg(rgb(crate::constants::colors::onenote_tab_idle())))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    action(this, cx);
                    cx.stop_propagation();
                }),
            );
    }
    row.child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(14.0))
                .text_color(rgb(ink))
                .child(icon),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .text_size(px(14.0))
                .font_family("Calibri")
                .text_color(rgb(ink))
                .child(
                    div()
                        .border_b_1()
                        .border_color(rgb(ink))
                        .child(first),
                )
                .child(rest),
        )
        .into_any_element()
}
