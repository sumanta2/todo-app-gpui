//! Main pane: page list and either the editor or the viewer. The ribbon is drawn above this pane.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Window};

use crate::app::NotesApp;
use crate::constants::{
    colors::onenote_ink_muted,
    typography::EMPTY_STATE_FONT_SIZE,
};
use crate::helpers::hash_str;
use crate::models::load_note_content;

impl NotesApp {
    /// Width of the notebook list, or zero when the canvas fills the window.
    pub(crate) fn layout_sidebar_w(&self) -> f32 {
        if self.full_page_view {
            0.0
        } else if self.is_sidebar_open {
            220.0
        } else {
            0.0
        }
    }

    /// Width of the pages list, or zero when the canvas fills the window.
    pub(crate) fn layout_page_sidebar_w(&self) -> f32 {
        if self.full_page_view {
            0.0
        } else {
            self.page_sidebar_width
        }
    }

    /// Two-way arrow pinned to the canvas. It switches normal view and full page view.
    pub(crate) fn render_canvas_view_toggle(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id("canvas-view-toggle")
            .absolute()
            .top(px(8.0))
            .right(px(8.0))
            .w(px(32.0))
            .h(px(28.0))
            .rounded(px(4.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(1.0))
            .cursor_pointer()
            .bg(rgb(crate::constants::colors::onenote_tab_idle()))
            .border_1()
            .border_color(rgb(crate::constants::colors::onenote_bar()))
            .hover(|style| style.bg(rgb(crate::constants::colors::onenote_page_selected())))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.full_page_view = !this.full_page_view;
                    this.home_menu_open = false;
                    this.view_menu_open = false;
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .child(
                div()
                    .font_family("Segoe MDL2 Assets")
                    .text_size(px(12.0))
                    .text_color(rgb(crate::constants::colors::onenote_ink()))
                    .child("\u{E76B}"),
            )
            .child(
                div()
                    .font_family("Segoe MDL2 Assets")
                    .text_size(px(12.0))
                    .text_color(rgb(crate::constants::colors::onenote_ink()))
                    .child("\u{E76C}"),
            )
            .into_any_element()
    }

    /// Drags the pages list from its left edge. Moving left widens it; moving right narrows it.
    pub(crate) fn drag_page_sidebar(&mut self, mouse_x: f32) -> bool {
        if !self.page_sidebar_resizing {
            return false;
        }
        let dx = mouse_x - self.page_sidebar_resize_start_x;
        let next = (self.page_sidebar_resize_start_w - dx).clamp(100.0, 420.0);
        if (next - self.page_sidebar_width).abs() < 0.5 {
            return false;
        }
        self.page_sidebar_width = next;
        true
    }

    /// Renders the main detail pane for the selected note.
    ///
    /// Depending on whether the note is in edit mode, this either shows the editable canvas or
    /// the read-only viewer, along with the page sidebar for navigation.
    pub(crate) fn render_detail_pane(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected_id = self.selected_note_id.clone();
        let has_note = selected_id
            .as_ref()
            .is_some_and(|id| self.notes.iter().any(|n| n.id == *id));

        if has_note {
            let editing = self.is_editing;
            let content: crate::models::NoteContent = if editing {
                if let Some(content) = self.edit_content.clone() {
                    content
                } else {
                    let note = self
                        .notes
                        .iter()
                        .find(|n| self.selected_note_id.as_ref() == Some(&n.id))
                        .unwrap();
                    load_note_content(&note.body, &note.images)
                }
            } else if let Some(content) = self.view_content.take() {
                content
            } else {
                let note = self
                    .notes
                    .iter()
                    .find(|n| self.selected_note_id.as_ref() == Some(&n.id))
                    .unwrap();
                load_note_content(&note.body, &note.images)
            };

            // ===============================================================================================================
            // Vertical Page sidebar showing all page list (on the right) ====================================================
            let page_sidebar = {
                let active_sec = content
                    .sections
                    .iter()
                    .find(|s: &&crate::models::NoteSection| {
                        Some(&s.id) == self.active_section_id.as_ref()
                    });
                let pages: Vec<crate::models::NotePage> = active_sec
                    .map(|s: &crate::models::NoteSection| &s.pages)
                    .cloned()
                    .unwrap_or_default();

                let mut page_elements: Vec<AnyElement> = Vec::new();
                for page in &pages {
                    let page_id = page.id.clone();
                    let is_active = Some(&page_id) == self.active_page_id.as_ref();
                    let click_id = page_id.clone();
                    let active_sec_id = self.active_section_id.clone().unwrap_or_default();
                    let editing = self.is_editing;

                    let mut page_row = div()
                        .id(("page-row", hash_str(&page_id)))
                        .px(px(4.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .cursor_pointer();
                    if is_active {
                        page_row =
                            page_row.bg(rgb(crate::constants::colors::onenote_page_selected()));
                    }
                    let menu_page_id = click_id.clone();
                    let menu_sec_id = active_sec_id.clone();
                    page_row = page_row
                        .hover(|s| s.bg(rgb(crate::constants::colors::onenote_tab_idle())))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                if this.active_page_id.as_deref() != Some(click_id.as_str()) {
                                    this.switch_to_page(
                                        active_sec_id.clone(),
                                        click_id.clone(),
                                        cx,
                                    );
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
                                if this.active_page_id.as_deref() != Some(menu_page_id.as_str()) {
                                    this.switch_to_page(
                                        menu_sec_id.clone(),
                                        menu_page_id.clone(),
                                        cx,
                                    );
                                }
                                if editing {
                                    this.section_menu_at = None;
                                    this.section_renaming = false;
                                    this.page_renaming = false;
                                    this.page_menu_at = Some((
                                        event.position.x.as_f32(),
                                        event.position.y.as_f32(),
                                    ));
                                }
                                cx.stop_propagation();
                                cx.notify();
                            }),
                        )
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family("Calibri")
                                .text_size(px(self.page_list_font_size))
                                .text_color(rgb(crate::constants::colors::onenote_ink()))
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

                    page_elements.push(page_row.into_any_element());
                }

                let add_page_btn = if self.is_editing {
                    Some(
                        div()
                            .id("add-page-btn")
                            .mb(px(6.0))
                            .px(px(8.0))
                            .py(px(4.0))
                            .bg(rgb(crate::constants::colors::onenote_page_list()))
                            .hover(|s| s.bg(rgb(crate::constants::colors::onenote_tab_idle())))
                            .text_color(rgb(crate::constants::colors::onenote_accent()))
                            .font_family("Calibri")
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_size(px(self.page_list_font_size))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.add_page(cx);
                            }))
                            .child("+ Add Page"),
                    )
                } else {
                    None
                };

                div()
                    .id("page-sidebar")
                    .relative()
                    .w(px(self.page_sidebar_width))
                    .h_full()
                    .bg(rgb(crate::constants::colors::onenote_page_list()))
                    .border_l_1()
                    .border_color(rgb(crate::constants::colors::onenote_bar_line()))
                    .p(px(6.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .id("page-sidebar-resize")
                            .absolute()
                            .left(px(-3.0))
                            .top(px(0.0))
                            .bottom(px(0.0))
                            .w(px(6.0))
                            .cursor_col_resize()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.page_sidebar_resizing = true;
                                    this.page_sidebar_resize_start_x = event.position.x.as_f32();
                                    this.page_sidebar_resize_start_w = this.page_sidebar_width;
                                    cx.stop_propagation();
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .font_family("Calibri")
                            .text_size(px(self.page_list_font_size))
                            .text_color(rgb(crate::constants::colors::onenote_ink_muted()))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("Pages"),
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

            // ====================================================================================================================
            // show the Canvas section where notes and image will show and user use to create different notes  ====================
            let page = if editing {
                self.render_canvas_editor(&content, page_sidebar.into_any_element(), window, cx)
                    .into_any_element()
            } else {
                self.render_canvas_viewer(&content, page_sidebar.into_any_element(), window, cx)
                    .into_any_element()
            };
            if !editing {
                self.view_content = Some(content);
            }

            div()
                .flex()
                .flex_col()
                .flex_1()
                .h_full()
                .min_w(px(0.0))
                .bg(rgb(crate::constants::colors::onenote_bar()))
                .child(page)
                .into_any_element()
        } else {
            // NO NOTE SELECTED PLACEHOLDER
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .flex_1()
                .h_full()
                .bg(rgb(crate::constants::colors::onenote_bar()))
                .child(
                    div()
                        .text_size(px(EMPTY_STATE_FONT_SIZE))
                        .text_color(rgb(onenote_ink_muted()))
                        .child("Select a note or create a new one to begin"),
                )
                .into_any_element()
        }
    }
}
