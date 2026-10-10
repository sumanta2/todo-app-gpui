//! Ribbon tab strip. Home and View each open their own menu.

mod controls;
mod dropdown;
mod home;
mod view;

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, MouseButton};

use crate::app::{NotesApp, RibbonPane};
use crate::constants::colors::{
    header_ribbon_bg, header_ribbon_bg_hover, header_ribbon_text, note_page, onenote_ink,
};
use crate::constants::typography::{RIBBON_PAD_H, RIBBON_TEXT};

impl NotesApp {
    pub(crate) fn render_home_ribbon(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.home_menu_open
            || (self.ribbon_pinned && self.ribbon_pane == RibbonPane::Home);

        div()
            .w_full()
            .h(px(36.0))
            .flex_shrink_0()
            .bg(rgb(header_ribbon_bg()))
            .flex()
            .flex_row()
            .items_end()
            .pl(px(8.0))
            .gap(px(2.0))
            .child(
                div()
                    .id("home-ribbon-tab")
                    .h(px(32.0))
                    .px(px(RIBBON_PAD_H))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .text_size(px(RIBBON_TEXT))
                    .font_family("Calibri")
                    .font_weight(if open {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .bg(rgb(if open { note_page() } else { header_ribbon_bg() }))
                    .text_color(rgb(if open { onenote_ink() } else { header_ribbon_text() }))
                    .hover(|style| {
                        if open {
                            style
                        } else {
                            style
                                .bg(rgb(header_ribbon_bg_hover()))
                                .text_color(rgb(header_ribbon_text()))
                        }
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.ribbon_pinned {
                                this.ribbon_pane = RibbonPane::Home;
                                this.home_menu_open = false;
                                this.view_menu_open = false;
                                this.zoom_menu_open = false;
                            } else {
                                this.home_menu_open = !this.home_menu_open;
                                this.view_menu_open = false;
                            }
                            if !this.home_menu_open {
                                this.font_style_menu_open = false;
                                this.font_size_menu_open = false;
                                this.font_color_menu_open = false;
                                this.bg_color_menu_open = false;
                            }
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child("Home"),
            )
            .child(self.render_view_tab(cx))
            .into_any_element()
    }

    /// Pinned Home and View menus stay under the ribbon, above the sidebar and section list.
    pub(crate) fn render_docked_dropdowns(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.ribbon_pinned {
            return None;
        }
        let menu = match self.ribbon_pane {
            RibbonPane::Home => self.home_menu_bar(cx),
            RibbonPane::View => self.view_menu_bar(cx),
        };
        Some(
            div()
                .w_full()
                .flex_shrink_0()
                .bg(rgb(note_page()))
                .child(menu)
                .into_any_element(),
        )
    }

    fn render_view_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.view_menu_open
            || (self.ribbon_pinned && self.ribbon_pane == RibbonPane::View);
        div()
            .id("view-ribbon-tab")
            .h(px(32.0))
            .px(px(RIBBON_PAD_H))
            .cursor_pointer()
            .flex()
            .items_center()
            .text_size(px(RIBBON_TEXT))
            .font_family("Calibri")
            .font_weight(if open {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .bg(rgb(if open { note_page() } else { header_ribbon_bg() }))
            .text_color(rgb(if open { onenote_ink() } else { header_ribbon_text() }))
            .hover(|style| {
                if open {
                    style
                } else {
                    style
                        .bg(rgb(header_ribbon_bg_hover()))
                        .text_color(rgb(header_ribbon_text()))
                }
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.ribbon_pinned {
                        this.ribbon_pane = RibbonPane::View;
                        this.view_menu_open = false;
                        this.home_menu_open = false;
                        this.font_style_menu_open = false;
                        this.font_size_menu_open = false;
                        this.font_color_menu_open = false;
                        this.bg_color_menu_open = false;
                    } else {
                        this.view_menu_open = !this.view_menu_open;
                        this.home_menu_open = false;
                        this.font_style_menu_open = false;
                        this.font_size_menu_open = false;
                        this.font_color_menu_open = false;
                        this.bg_color_menu_open = false;
                    }
                    this.zoom_menu_open = false;
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .child("View")
            .into_any_element()
    }
}
