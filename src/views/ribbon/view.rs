//! View ribbon menu: normal and full-page layout, plus canvas zoom.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, MouseButton};

use crate::app::NotesApp;
use crate::constants::{
        colors::{NOTE_PAGE, ONENOTE_BAR_LINE, ONENOTE_INK_MUTED, ONENOTE_PAGE_SELECTED},
    typography::{RIBBON_GAP, RIBBON_ICON, RIBBON_PAD_H, RIBBON_PAD_V},
};

use super::controls::FormatTooltip;
use super::dropdown::{dropdown_popup, DROPDOWN_PAD};

impl NotesApp {
    pub(crate) fn render_view_menu_layers(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        if !self.view_menu_open {
            return Vec::new();
        }
        let full = self.full_page_view;
        let zoom_open = self.zoom_menu_open;
        let zoom_label = self.zoom_percent_label();
        // Zoom popup
        let zoom_popup = zoom_open.then(|| {
            dropdown_popup(
                ZOOM_DROPDOWN_W,
                DROPDOWN_PAD,
                zoom_option_list(self.canvas_zoom, cx),
            )
        });
        let menu = div()
            .absolute()
            .top(px(40.0))
            .bg(rgb(NOTE_PAGE))
            .border_1()
            .border_color(rgb(ONENOTE_BAR_LINE))
            .flex()
            .flex_col()
            .gap(px(RIBBON_GAP))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(view_mode_button(
                        "view-normal",
                        "Normal view",
                        !full,
                        false,
                        cx,
                    ))
                    .child(view_mode_button(
                        "view-full-page",
                        "Full page view",
                        full,
                        true,
                        cx,
                    ))
                    .child(
                        div()
                            .w(px(1.0))
                            .h(px(RIBBON_ICON))
                            .mx(px(4.0))
                            .bg(rgb(ONENOTE_BAR_LINE)),
                    )
                    .child(zoom_step_button(
                        "zoom-100",
                        "100%",
                        "Zoom to 100%.",
                        (self.canvas_zoom - 1.0).abs() < 0.02,
                        cx,
                        true,
                    ))
                    .child(zoom_step_button("zoom-in", "+", "Zoom in.", false, cx, false))
                    .child(zoom_step_button("zoom-out", "−", "Zoom out.", false, cx, false))
                    .child(zoom_picker(zoom_label, zoom_open, zoom_popup, cx)),
            );
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
                        this.view_menu_open = false;
                        this.zoom_menu_open = false;
                        cx.notify();
                    }),
                )
                .into_any_element(),
            menu.into_any_element(),
        ]
    }
}

const VIEW_TEXT: f32 = 16.0;
const VIEW_INK: u32 = 0x4a3f2c;
/// Zoom dropdown trigger. One third narrower than the 132px font dropdown.
const ZOOM_DROPDOWN_W: f32 = 70.0;

fn zoom_step_button(
    id: &'static str,
    mark: &'static str,
    description: &'static str,
    active: bool,
    cx: &mut Context<NotesApp>,
    reset: bool,
) -> AnyElement {
    let mut button = div()
        .id(id)
        .py(px(RIBBON_PAD_V))
        .min_w(px(36.0))
        .px(px(RIBBON_PAD_H))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer();
    if active {
        button = button.bg(rgb(ONENOTE_PAGE_SELECTED));
    } else {
        button = button.hover(|style| style.bg(rgb(ONENOTE_BAR_LINE)));
    }
    button
        .tooltip(move |_window, cx| {
            cx.new(|_| FormatTooltip {
                label: if reset {
                    "100%"
                } else if mark == "+" {
                    "Zoom In"
                } else {
                    "Zoom Out"
                },
                shortcut: if reset {
                    "Ctrl+0"
                } else if mark == "+" {
                    "Ctrl+Plus"
                } else {
                    "Ctrl+Minus"
                },
                description,
            })
            .into()
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                if reset {
                    this.reset_canvas_zoom(cx);
                } else if mark == "+" {
                    this.zoom_by(1.1, (0.0, 0.0), cx);
                } else {
                    this.zoom_by(1.0 / 1.1, (0.0, 0.0), cx);
                }
                cx.stop_propagation();
            }),
        )
        .child(
            div()
                .text_size(px(VIEW_TEXT))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(VIEW_INK))
                .child(mark),
        )
        .into_any_element()
}

fn zoom_picker(
    label: String,
    open: bool,
    popup: Option<AnyElement>,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    div()
        .id("zoom-picker")
        .relative()
        .py(px(RIBBON_PAD_V))
        .w(px(ZOOM_DROPDOWN_W))
        .px(px(RIBBON_PAD_H))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .when(open, |this| this.bg(rgb(ONENOTE_PAGE_SELECTED)))
        .hover(|style| {
            if open {
                style.bg(rgb(ONENOTE_PAGE_SELECTED))
            } else {
                style.bg(rgb(ONENOTE_BAR_LINE))
            }
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                this.zoom_menu_open = !this.zoom_menu_open;
                cx.notify();
                cx.stop_propagation();
            }),
        )
        .child(
            div()
                .text_size(px(VIEW_TEXT))
                .text_color(rgb(VIEW_INK))
                .child(label),
        )
        .child(
            div()
                .text_size(px(VIEW_TEXT))
                .text_color(rgb(ONENOTE_INK_MUTED))
                .child(if open { "▴" } else { "▾" }),
        )
        .children(popup)
        .into_any_element()
}

fn zoom_option_list(current: f32, cx: &mut Context<NotesApp>) -> AnyElement {
    let mut list = div().flex().flex_row().flex_wrap().gap(px(RIBBON_GAP));
    for zoom in crate::app::zoom::ZOOM_OPTIONS {
        let zoom = *zoom;
        let chosen = (current - zoom).abs() < 0.02;
        let label = format!("{}%", (zoom * 100.0).round() as i32);
        list = list.child(
            div()
                .id(("zoom-option", (zoom * 100.0) as usize))
                .py(px(RIBBON_PAD_V))
                .flex()
                .items_center()
                .cursor_pointer()
                .hover(|style| style.bg(rgb(ONENOTE_BAR_LINE)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.set_canvas_zoom(zoom, (0.0, 0.0), cx);
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .text_size(px(VIEW_TEXT))
                        .text_color(rgb(VIEW_INK))
                        .when(chosen, |this| this.font_weight(gpui::FontWeight::SEMIBOLD))
                        .child(label),
                ),
        );
    }
    list.into_any_element()
}

fn view_mode_button(
    id: &'static str,
    label: &'static str,
    active: bool,
    full_page: bool,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    div()
        .id(id)
        .py(px(RIBBON_PAD_V))
        .px(px(RIBBON_PAD_H))
        .flex()
        .items_center()
        .cursor_pointer()
        .when(active, |this| this.bg(rgb(ONENOTE_PAGE_SELECTED)))
        .hover(|style| {
            if active {
                style.bg(rgb(ONENOTE_PAGE_SELECTED))
            } else {
                style.bg(rgb(ONENOTE_BAR_LINE))
            }
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.full_page_view = full_page;
                this.view_menu_open = false;
                this.home_menu_open = false;
                cx.notify();
                cx.stop_propagation();
            }),
        )
        .child(
            div()
                .text_size(px(VIEW_TEXT))
                .text_color(rgb(VIEW_INK))
                .child(label),
        )
        .into_any_element()
}
