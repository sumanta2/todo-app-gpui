//! Ribbon tab strip. Home and View each open their own menu.

mod controls;
mod dropdown;
mod home;
mod view;

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, MouseButton};

use crate::app::NotesApp;
use crate::constants::typography::{RIBBON_PAD_H, RIBBON_PAD_V, RIBBON_TEXT};

impl NotesApp {
    pub(crate) fn render_home_ribbon(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.home_menu_open;

        div()
            .w_full()
            .h(px(36.0))
            .flex_shrink_0()
            .bg(rgb(crate::constants::colors::ONENOTE_BAR))
            .border_b_1()
            .border_color(rgb(crate::constants::colors::ONENOTE_BAR_LINE))
            .flex()
            .flex_row()
            .items_center()
            .px(px(RIBBON_PAD_H))
            .child(
                div()
                    .id("home-ribbon-tab")
                    .px(px(RIBBON_PAD_H))
                    .py(px(RIBBON_PAD_V))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .text_size(px(RIBBON_TEXT))
                    .font_family("Calibri")
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .bg(if open {
                        rgb(crate::constants::colors::ONENOTE_BAR_SELECTED)
                    } else {
                        rgb(crate::constants::colors::ONENOTE_BAR)
                    })
                    .text_color(rgb(crate::constants::colors::ONENOTE_INK))
                    .hover(|style| {
                        style.bg(rgb(if open {
                            crate::constants::colors::ONENOTE_BAR_SELECTED
                        } else {
                            crate::constants::colors::ONENOTE_BAR_HOVER
                        }))
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.home_menu_open = !this.home_menu_open;
                            this.view_menu_open = false;
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

    fn render_view_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.view_menu_open;
        div()
            .id("view-ribbon-tab")
            .px(px(RIBBON_PAD_H))
            .py(px(RIBBON_PAD_V))
            .cursor_pointer()
            .flex()
            .items_center()
            .text_size(px(RIBBON_TEXT))
            .font_family("Calibri")
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .bg(if open {
                rgb(crate::constants::colors::ONENOTE_BAR_SELECTED)
            } else {
                rgb(crate::constants::colors::ONENOTE_BAR)
            })
            .text_color(rgb(crate::constants::colors::ONENOTE_INK))
            .hover(|style| {
                style.bg(rgb(if open {
                    crate::constants::colors::ONENOTE_BAR_SELECTED
                } else {
                    crate::constants::colors::ONENOTE_BAR_HOVER
                }))
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.view_menu_open = !this.view_menu_open;
                    this.zoom_menu_open = false;
                    this.home_menu_open = false;
                    this.font_style_menu_open = false;
                    this.font_size_menu_open = false;
                    this.font_color_menu_open = false;
                    this.bg_color_menu_open = false;
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .child("View")
            .into_any_element()
    }
}
