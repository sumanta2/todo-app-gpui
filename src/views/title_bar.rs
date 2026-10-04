//! Window caption: app name, note search, and minimize, maximize, and close.

use gpui::{div, prelude::*, px, rgb, Context, Window, WindowControlArea};

use crate::app::NotesApp;

impl NotesApp {
    pub(crate) fn render_title_bar(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let maximized = window.is_maximized();
        let query = self.title_search.clone();
        let search_focused = self.title_search_focused;
        div()
            .id("app-title-bar")
            .w_full()
            .h(px(36.0))
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .bg(rgb(crate::constants::colors::ONENOTE_BAR))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .px(px(12.0))
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(crate::constants::colors::ONENOTE_INK))
                            .child("Modern Notes"),
                    ),
            )
            .child(
                div()
                    .id("title-search")
                    .w(px(240.0))
                    .h(px(24.0))
                    .px(px(8.0))
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .bg(rgb(crate::constants::colors::NOTE_PAGE))
                    .border_1()
                    .border_color(rgb(if search_focused {
                        crate::constants::colors::ONENOTE_ACCENT
                    } else {
                        crate::constants::colors::ONENOTE_BAR_LINE
                    }))
                    .cursor_text()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.title_search_focused = true;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(if query.is_empty() && !search_focused {
                                crate::constants::colors::NOTE_HINT
                            } else {
                                crate::constants::colors::ONENOTE_INK
                            }))
                            .child(if query.is_empty() && !search_focused {
                                "Search notes".to_string()
                            } else if search_focused {
                                format!("{query}|")
                            } else {
                                query
                            }),
                    ),
            )
            .child(caption_button(
                "title-min",
                "\u{E921}",
                WindowControlArea::Min,
                false,
            ))
            .child(caption_button(
                "title-max",
                if maximized { "\u{E923}" } else { "\u{E922}" },
                WindowControlArea::Max,
                false,
            ))
            .child(caption_button(
                "title-close",
                "\u{E8BB}",
                WindowControlArea::Close,
                true,
            ))
            .into_any_element()
    }
}

fn caption_button(
    id: &'static str,
    glyph: &'static str,
    area: WindowControlArea,
    close: bool,
) -> gpui::AnyElement {
    div()
        .id(id)
        .w(px(46.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .window_control_area(area)
        .hover(|style| {
            style.bg(rgb(if close {
                0xc42b1c
            } else {
                crate::constants::colors::ONENOTE_BAR_HOVER
            }))
        })
        .child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(10.0))
                .text_color(rgb(if close {
                    crate::constants::colors::ONENOTE_INK
                } else {
                    crate::constants::colors::ONENOTE_INK
                }))
                .hover(|style| {
                    if close {
                        style.text_color(rgb(0xffffff))
                    } else {
                        style
                    }
                })
                .child(glyph),
        )
        .into_any_element()
}
