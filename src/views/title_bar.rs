//! Window caption: app name, note search, and minimize, maximize, and close.

use gpui::{
    div, prelude::*, px, rgb, Context, IntoElement, MouseButton, Render, Window, WindowControlArea,
};

use crate::app::NotesApp;
use crate::models::ActiveField;

impl NotesApp {
    pub(crate) fn render_title_bar(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let maximized = window.is_maximized();
        let query = self.title_search.clone();
        let search_active = self.active_field == ActiveField::Search;
        let can_undo = self.can_undo();
        let can_redo = self.can_redo();
        div()
            .id("app-title-bar")
            .w_full()
            .h(px(36.0))
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .bg(rgb(crate::constants::colors::header_ribbon_bg()))
            .child(history_caption_button(
                "title-undo",
                "\u{E7A7}",
                can_undo,
                cx,
                true,
            ))
            .child(history_caption_button(
                "title-redo",
                "\u{E7A6}",
                can_redo,
                cx,
                false,
            ))
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
                            .text_color(rgb(crate::constants::colors::header_ribbon_text()))
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
                    .bg(rgb(crate::constants::colors::note_page()))
                    .border_1()
                    .border_color(rgb(if search_active {
                        crate::constants::colors::onenote_accent()
                    } else {
                        crate::constants::colors::onenote_bar_line()
                    }))
                    .cursor_text()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.active_field = ActiveField::Search;
                            this.cursor_visible = true;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_size(px(13.0))
                            .text_color(rgb(if query.is_empty() && !search_active {
                                crate::constants::colors::note_hint()
                            } else {
                                crate::constants::colors::onenote_ink()
                            }))
                            .child(if query.is_empty() && !search_active {
                                "Search notes".to_string()
                            } else {
                                query
                            })
                            .when(search_active && self.cursor_visible, |this| {
                                this.child(
                                    div()
                                        .w(px(crate::constants::typography::CURSOR_WIDTH))
                                        .h(px(14.0))
                                        .ml(px(1.0))
                                        .flex_shrink_0()
                                        .bg(rgb(crate::constants::colors::cursor())),
                                )
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

fn history_caption_button(
    id: &'static str,
    glyph: &'static str,
    enabled: bool,
    cx: &mut Context<NotesApp>,
    undo: bool,
) -> gpui::AnyElement {
    let ink = if enabled {
        crate::constants::colors::header_ribbon_text()
    } else {
        crate::constants::colors::header_ribbon_text_muted()
    };
    let label = if undo { "Undo" } else { "Redo" };
    let shortcut = if undo { "Ctrl+Z" } else { "Ctrl+Y" };
    let mut button = div()
        .id(id)
        .ml(px(if undo { 8.0 } else { 0.0 }))
        .w(px(28.0))
        .h(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.0))
        .tooltip(move |_window, cx| {
            cx.new(|_| HistoryTooltip { label, shortcut }).into()
        })
        .child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(14.0))
                .text_color(rgb(ink))
                .child(glyph),
        );
    if enabled {
        button = button
            .cursor_pointer()
            .hover(|style| style.bg(rgb(crate::constants::colors::header_ribbon_bg_hover())))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                if undo {
                    this.undo(cx);
                } else {
                    this.redo(cx);
                }
                cx.stop_propagation();
            }));
    }
    button.into_any_element()
}

struct HistoryTooltip {
    label: &'static str,
    shortcut: &'static str,
}

impl Render for HistoryTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.0))
            .py(px(4.0))
            .bg(rgb(crate::constants::colors::note_page()))
            .border_1()
            .border_color(rgb(crate::constants::colors::onenote_bar_line()))
            .text_size(px(12.0))
            .text_color(rgb(crate::constants::colors::onenote_ink()))
            .child(format!("{} ({})", self.label, self.shortcut))
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
                crate::constants::colors::header_ribbon_bg_hover()
            }))
        })
        .child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(10.0))
                .text_color(rgb(crate::constants::colors::header_ribbon_text()))
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
