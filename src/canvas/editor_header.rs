use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, IntoElement, MouseButton};

use crate::app::NotesApp;
use crate::constants::{
    layout::HEADING_PADDING_LEFT,
    typography::{WEIGHT_BOLD, WEIGHT_NORMAL},
};
use crate::helpers::hash_str;
use crate::models::{ActiveField, NoteContent};
use crate::text_selection::{calculate_line_text_offset_with_font, calculate_text_width_for_font};

impl NotesApp {
    /// Renders the section-name input inline in the active tab.
    ///
    /// When the field is empty, it shows a placeholder; when it has focus, it draws the
    /// live cursor and selection highlight to match the editing experience used elsewhere.
    fn build_section_name_editor(&self, is_focused: bool, cx: &mut Context<Self>) -> AnyElement {
        let font_size = self.section_name_font_size;
        if self.edit_section_name.is_empty() {
            div()
                .id("section-name-placeholder")
                .relative()
                .flex()
                .items_center()
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
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(0x808080))
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
                            rgb(0x0078d4)
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
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
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
                .id("section-name-editor")
                .relative()
                .flex()
                .items_center()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                        this.active_field = ActiveField::SectionName;
                        this.focus_handle.focus(window, cx);
                        let rel_x = (event.position.x.as_f32() - this.active_section_tab_x).max(0.0);
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
                        let rel_x = (event.position.x.as_f32() - this.active_section_tab_x).max(0.0);
                        let drag_idx = calculate_line_text_offset_with_font(
                            rel_x,
                            &this.edit_section_name,
                            this.section_name_font_size,
                            WEIGHT_BOLD,
                            this.font_type(),
                        );
                        this.edit_section_name_cursor = drag_idx;
                        cx.notify();
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

    /// Builds the horizontal section tab strip and editing action buttons.
    ///
    /// Each tab represents a section of the note, while the toolbar exposes actions such as
    /// bold formatting, save, and cancel for the current edit session.
    pub(crate) fn build_section_tabs(
        &mut self,
        content: &NoteContent,
        is_section_name_focused: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut current_tab_x = if self.is_sidebar_open { 220.0 } else { 44.0 };
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
                .px(px(6.0))
                .py(px(2.0))
                .text_size(px(self.section_name_font_size))
                .bg(if is_active {
                    rgb(0x1e1e1e)
                } else {
                    rgb(0x2d2d2d)
                });
            if is_active {
                tab_el = tab_el.border_t_2().border_color(rgb(0x0078d4));
            }
            tab_el = tab_el
                .text_color(if is_active {
                    rgb(0xffffff)
                } else {
                    rgb(0x808080)
                })
                .font_weight(if is_active {
                    gpui::FontWeight::BOLD
                } else {
                    gpui::FontWeight::NORMAL
                })
                .cursor_pointer()
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
                if self.edit_section_name.is_empty() { "Section Name..." } else { &self.edit_section_name }
            } else {
                &sec.name
            };
            let weight = if is_active { WEIGHT_BOLD } else { WEIGHT_NORMAL };
            let text_w = calculate_text_width_for_font(tab_text, self.section_name_font_size, weight, self.font_type());
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
                .px(px(6.0))
                .py(px(2.0))
                .text_size(px(self.section_name_font_size))
                .bg(rgb(0x252525))
                .hover(|s| s.bg(rgb(0x353535)))
                .text_color(rgb(0x0078d4))
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.add_section(cx);
                }))
                .child("+")
                .into_any_element(),
        );

        let left_side = div().flex().flex_row().gap(px(4.0)).children(tabs);

        let right_side = div()
            .flex()
            .gap(px(6.0))
            .child(
                div()
                    .id("bold-btn")
                    .px(px(7.0))
                    .py(px(2.0))
                    .bg(rgb(0x2d2d2d))
                    .hover(|s| s.bg(rgb(0x3d3d3d)))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_bold(cx);
                    }))
                    .child("B"),
            )
            .child(
                div()
                    .id("save-btn")
                    .px(px(5.0))
                    .py(px(2.0))
                    .bg(rgb(0x0078d4))
                    .hover(|s| s.bg(rgb(0x106ebe)))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .text_color(rgb(0xffffff))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.save_edit(cx);
                    }))
                    .child("Save"),
            )
            .child(
                div()
                    .id("cancel-btn")
                    .px(px(5.0))
                    .py(px(2.0))
                    .bg(rgb(0x404040))
                    .hover(|s| s.bg(rgb(0x505050)))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .text_color(rgb(0xd4d4d4))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.cancel_edit(cx);
                    }))
                    .child("Cancel"),
            );

        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_center()
            .child(left_side)
            .child(right_side)
    }

    /// Renders the page heading field used to title the currently selected note page.
    pub(crate) fn build_heading_editor(
        &self,
        is_heading_focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let font_size = self.page_heading_font_size;
        if self.edit_heading.is_empty() {
            div()
                .id("heading-placeholder")
                .relative()
                .flex()
                .items_center()
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
                        .text_color(rgb(0x808080))
                        .child("Heading..."),
                )
                .child(if is_heading_focused {
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(2.0))
                        .w(px(2.0))
                        .h(px(font_size))
                        .bg(if self.cursor_visible {
                            rgb(0x0078d4)
                        } else {
                            rgba(0x00000000)
                        })
                } else {
                    div()
                })
                .into_any_element()
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
                    .text_color(rgb(0xffffff))
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
                                .w(px(2.0))
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
                .id("heading-editor")
                .relative()
                .flex()
                .items_center()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                        this.active_field = ActiveField::Heading;
                        this.focus_handle.focus(window, cx);
                        let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                        let rel_x = (event.position.x.as_f32() - sidebar_w - HEADING_PADDING_LEFT).max(0.0);
                        let click_idx = calculate_line_text_offset_with_font(
                            rel_x,
                            &this.edit_heading,
                            this.page_heading_font_size,
                            WEIGHT_NORMAL,
                            this.font_type(),
                        );
                        this.edit_heading_cursor = click_idx;
                        this.edit_heading_anchor = Some(click_idx);
                        this.is_selecting_heading = true;
                        this.cursor_visible = true;
                        cx.notify();
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                    if this.is_selecting_heading {
                        let sidebar_w = if this.is_sidebar_open { 220.0 } else { 44.0 };
                        let rel_x = (event.position.x.as_f32() - sidebar_w - HEADING_PADDING_LEFT).max(0.0);
                        let drag_idx = calculate_line_text_offset_with_font(
                            rel_x,
                            &this.edit_heading,
                            this.page_heading_font_size,
                            WEIGHT_NORMAL,
                            this.font_type(),
                        );
                        this.edit_heading_cursor = drag_idx;
                        cx.notify();
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
                .children(elements)
                .into_any_element()
        }
    }
}
