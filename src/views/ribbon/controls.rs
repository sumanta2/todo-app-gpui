//! Shared ribbon controls: dropdowns, style buttons, and color swatches.

use gpui::{
    div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Render, Window,
};

use super::dropdown::{dropdown_items, DropdownFlow};
use crate::app::formatting::TextStyleKind;
use crate::app::NotesApp;
use crate::constants::{
    colors::{
        NOTE_PAGE, ONENOTE_ACCENT, ONENOTE_BAR_LINE, ONENOTE_INK, ONENOTE_INK_MUTED,
        ONENOTE_PAGE_SELECTED,
    },
    typography::{
        font_size_label, FONT_SIZE_OPTIONS, FONT_STYLE_OPTIONS, RIBBON_GAP, RIBBON_ICON,
        RIBBON_PAD_H, RIBBON_PAD_V, RIBBON_TEXT,
    },
};

/// Hover card for a formatting icon: command name with shortcut, then a short description.
pub(super) struct FormatTooltip {
    pub(super) label: &'static str,
    pub(super) shortcut: &'static str,
    pub(super) description: &'static str,
}

impl Render for FormatTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p(px(8.0))
            .bg(rgb(NOTE_PAGE))
            .border_1()
            .border_color(rgb(ONENOTE_BAR_LINE))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .text_size(px(RIBBON_TEXT))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(ONENOTE_INK))
                    .child(format!("{} ({})", self.label, self.shortcut)),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(ONENOTE_INK_MUTED))
                    .child(self.description),
            )
    }
}

/// Dropdown trigger. `popup` is the open list, built with `dropdown_popup`.
pub(super) fn ribbon_dropdown(
    id: &'static str,
    label: String,
    width: f32,
    open: bool,
    popup: Option<AnyElement>,
    cx: &mut Context<NotesApp>,
    on_click: impl Fn(&mut NotesApp, &gpui::MouseDownEvent, &mut Window, &mut Context<NotesApp>)
        + 'static,
) -> AnyElement {
    div()
        .id(id)
        .relative()
        .w(px(width))
        .py(px(RIBBON_PAD_V))
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
        .on_mouse_down(MouseButton::Left, cx.listener(on_click))
        .child(
            div()
                .text_size(px(RIBBON_TEXT))
                .text_color(rgb(ONENOTE_INK))
                .child(label),
        )
        .child(
            div()
                .text_size(px(RIBBON_TEXT))
                .text_color(rgb(ONENOTE_INK_MUTED))
                .child(if open { "▴" } else { "▾" }),
        )
        .children(popup)
        .into_any_element()
}

pub(super) fn font_style_list(
    selected: u8,
    flow: DropdownFlow,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    let mut list = dropdown_items(flow, RIBBON_GAP).pt(px(2.0));
    for (index, name) in FONT_STYLE_OPTIONS.iter().enumerate() {
        let index = index as u8;
        list = list.child(menu_choice(
            (*name).to_string(),
            index == selected,
            cx,
            move |this, _, _, cx| {
                this.typing_font_family = index;
                this.font_style_menu_open = false;
                cx.notify();
                cx.stop_propagation();
            },
        ));
    }
    list.into_any_element()
}

pub(super) fn font_size_list(
    selected_px: f32,
    flow: DropdownFlow,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    let mut list = dropdown_items(flow, RIBBON_GAP).pt(px(2.0));
    for (_, size_px) in FONT_SIZE_OPTIONS {
        let size_px = *size_px;
        let chosen = (size_px - selected_px).abs() < 0.05;
        list = list.child(menu_choice(
            font_size_label(size_px),
            chosen,
            cx,
            move |this, _, _, cx| {
                this.typing_font_size_px = size_px;
                this.font_size_menu_open = false;
                cx.notify();
                cx.stop_propagation();
            },
        ));
    }
    list.into_any_element()
}

fn menu_choice(
    label: String,
    selected: bool,
    cx: &mut Context<NotesApp>,
    on_click: impl Fn(&mut NotesApp, &gpui::MouseDownEvent, &mut Window, &mut Context<NotesApp>)
        + 'static,
) -> AnyElement {
    div()
        .w_full()
        .py(px(RIBBON_PAD_V))
        .flex()
        .items_center()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(ONENOTE_BAR_LINE)))
        .on_mouse_down(MouseButton::Left, cx.listener(on_click))
        .child(
            div()
                .text_size(px(RIBBON_TEXT))
                .text_color(rgb(ONENOTE_INK))
                .when(selected, |this| this.font_weight(gpui::FontWeight::SEMIBOLD))
                .child(label),
        )
        .into_any_element()
}

#[derive(Clone, Copy)]
pub(super) enum ClipAction {
    Cut,
    Copy,
    Paste,
    Delete,
    Undo,
    Redo,
}

#[derive(Clone, Copy)]
pub(super) enum LayoutAction {
    DecreaseIndent,
    IncreaseIndent,
    AlignLeft,
    AlignCenter,
    AlignRight,
}

pub(super) fn layout_button(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    active: bool,
    enabled: bool,
    cx: &mut Context<NotesApp>,
    action: LayoutAction,
) -> AnyElement {
    let label_color = if !enabled {
        ONENOTE_INK_MUTED
    } else if active {
        ONENOTE_INK
    } else {
        ONENOTE_INK
    };
    let mut button = div()
        .id(id)
        .px(px(RIBBON_PAD_H))
        .py(px(RIBBON_PAD_V))
        .flex()
        .items_center()
        .justify_center()
        .tooltip(move |_window, cx| {
            cx.new(|_| FormatTooltip {
                label,
                shortcut: "Layout",
                description,
            })
            .into()
        })
        .child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(RIBBON_ICON))
                .text_color(rgb(label_color))
                .child(icon),
        );
    if active && enabled {
        button = button.bg(rgb(ONENOTE_PAGE_SELECTED));
    }
    if enabled {
        button = button
            .cursor_pointer()
            .hover(|style| {
                style.bg(if active {
                    rgb(ONENOTE_PAGE_SELECTED)
                } else {
                    rgb(ONENOTE_BAR_LINE)
                })
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.font_style_menu_open = false;
                    this.font_size_menu_open = false;
                    this.font_color_menu_open = false;
                    this.bg_color_menu_open = false;
                    match action {
                        LayoutAction::DecreaseIndent => this.decrease_indent(cx),
                        LayoutAction::IncreaseIndent => this.increase_indent(cx),
                        LayoutAction::AlignLeft => this.align_content(0, cx),
                        LayoutAction::AlignCenter => this.align_content(1, cx),
                        LayoutAction::AlignRight => this.align_content(2, cx),
                    }
                    cx.stop_propagation();
                }),
            );
    }
    button.into_any_element()
}

pub(super) fn clip_button(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    shortcut: &'static str,
    description: &'static str,
    enabled: bool,
    cx: &mut Context<NotesApp>,
    action: ClipAction,
) -> AnyElement {
    let label_color = if enabled {
        ONENOTE_INK
    } else {
        ONENOTE_INK_MUTED
    };
    let mut button = div()
        .id(id)
        .px(px(RIBBON_PAD_H))
        .py(px(RIBBON_PAD_V))
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(label_color))
        .tooltip(move |_window, cx| {
            cx.new(|_| FormatTooltip {
                label,
                shortcut,
                description,
            })
            .into()
        })
        .child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(RIBBON_ICON))
                .text_color(rgb(label_color))
                .child(icon),
        );
    if enabled {
        button = button
            .cursor_pointer()
            .hover(|style| style.bg(rgb(ONENOTE_BAR_LINE)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.font_style_menu_open = false;
                    this.font_size_menu_open = false;
                    this.font_color_menu_open = false;
                    this.bg_color_menu_open = false;
                    match action {
                        ClipAction::Cut => this.cut_selection(cx),
                        ClipAction::Copy => this.copy_selection(cx),
                        ClipAction::Paste => this.paste_clipboard(cx),
                        ClipAction::Delete => this.delete_forward(cx),
                        ClipAction::Undo => this.undo(cx),
                        ClipAction::Redo => this.redo(cx),
                    }
                    cx.stop_propagation();
                }),
            );
    }
    button.into_any_element()
}

pub(super) fn color_button(
    id: &'static str,
    mark: &'static str,
    current: u32,
    empty_bar: u32,
    font_mark: bool,
    open: bool,
    popup: Option<AnyElement>,
    cx: &mut Context<NotesApp>,
    on_click: impl Fn(&mut NotesApp, &gpui::MouseDownEvent, &mut Window, &mut Context<NotesApp>)
        + 'static,
) -> AnyElement {
    let bar = if current == 0 { empty_bar } else { current };
    let mut glyph = div()
        .h(px(14.0))
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(ONENOTE_INK))
        .child(mark);
    if font_mark {
        glyph = glyph
            .text_size(px(RIBBON_ICON))
            .font_weight(gpui::FontWeight::BOLD);
    } else {
        glyph = glyph
            .font_family("Segoe MDL2 Assets")
            .text_size(px(RIBBON_ICON));
    }
    let mut bar_el = div().w(px(14.0)).h(px(3.0));
    if bar == 0 {
        bar_el = bar_el.border_1().border_color(rgb(ONENOTE_INK_MUTED));
    } else {
        bar_el = bar_el.bg(rgb(bar));
    }
    div()
        .id(id)
        .relative()
        .px(px(RIBBON_PAD_H))
        .py(px(RIBBON_PAD_V))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(2.0))
        .cursor_pointer()
        .when(open, |this| this.bg(rgb(ONENOTE_PAGE_SELECTED)))
        .hover(|style| {
            if open {
                style.bg(rgb(ONENOTE_PAGE_SELECTED))
            } else {
                style.bg(rgb(ONENOTE_BAR_LINE))
            }
        })
        .tooltip(move |_window, cx| {
            cx.new(|_| FormatTooltip {
                label: if font_mark { "Font Color" } else { "Highlight" },
                shortcut: if font_mark { "Text" } else { "Background" },
                description: if font_mark {
                    "Color the selected text, or the text you type next."
                } else {
                    "Highlight the selected text, or the text you type next."
                },
            })
            .into()
        })
        .on_mouse_down(MouseButton::Left, cx.listener(on_click))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(RIBBON_GAP))
                .child(glyph)
                .child(bar_el),
        )
        .child(
            div()
                .text_size(px(RIBBON_TEXT))
                .text_color(rgb(ONENOTE_INK_MUTED))
                .child(if open { "▴" } else { "▾" }),
        )
        .children(popup)
        .into_any_element()
}

/// Side of each color sample. Kept the same in every color dropdown.
const COLOR_BOX: f32 = 20.0;

pub(super) fn color_list(
    options: &'static [u32],
    selected: u32,
    font: bool,
    samples: DropdownFlow,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    let default_chosen = selected == 0;
    color_palette(options, selected, font, samples, default_chosen, cx)
}

/// Color menu: a Default button on its own row, then samples laid out by `samples_flow`.
fn color_palette(
    options: &'static [u32],
    selected: u32,
    font: bool,
    samples_flow: DropdownFlow,
    default_chosen: bool,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    let gap = RIBBON_GAP * 0.5;
    let mut samples = dropdown_items(samples_flow, gap);
    for &color in options {
        samples = samples.child(color_swatch(color, color == selected, font, cx));
    }

    dropdown_items(DropdownFlow::Column, gap)
        .pt(px(2.0))
        .child(default_color_choice(
            font,
            default_chosen,
            0.0,
            RIBBON_PAD_V * 0.5,
            true,
            cx,
        ))
        .child(samples)
        .into_any_element()
}

fn color_swatch(
    color: u32,
    chosen: bool,
    font: bool,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    div()
        .id(("fmt-color-swatch", color))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .border_1()
        .border_color(rgb(if chosen {
            ONENOTE_ACCENT
        } else {
            ONENOTE_BAR_LINE
        }))
        .when(chosen, |this| this.bg(rgb(ONENOTE_PAGE_SELECTED)))
        .hover(|style| {
            style.border_color(rgb(ONENOTE_ACCENT)).bg(if chosen {
                rgb(ONENOTE_PAGE_SELECTED)
            } else {
                rgb(ONENOTE_BAR_LINE)
            })
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                if font {
                    this.set_font_color(color, cx);
                    this.font_color_menu_open = false;
                } else {
                    this.set_bg_color(color, cx);
                    this.bg_color_menu_open = false;
                }
                cx.stop_propagation();
            }),
        )
        .child(div().w(px(COLOR_BOX)).h(px(COLOR_BOX)).bg(rgb(color)))
        .into_any_element()
}

/// First control in each color list. Clears a custom font color and highlight.
fn default_color_choice(
    font: bool,
    chosen: bool,
    pad_h: f32,
    pad_v: f32,
    fill: bool,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    div()
        .id(if font {
            "fmt-font-color-default"
        } else {
            "fmt-bg-color-default"
        })
        .when(fill, |this| this.w_full())
        .py(px(pad_v))
        .px(px(pad_h))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(ONENOTE_BAR_LINE)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.set_font_color(0, cx);
                this.set_bg_color(0, cx);
                this.font_color_menu_open = false;
                this.bg_color_menu_open = false;
                cx.stop_propagation();
            }),
        )
        .child(
            div()
                .text_size(px(RIBBON_PAD_H))
                .text_color(rgb(ONENOTE_INK))
                .when(chosen, |this| this.font_weight(gpui::FontWeight::SEMIBOLD))
                .child("Default"),
        )
        .into_any_element()
}

pub(super) fn format_button(
    id: &'static str,
    mark: &'static str,
    label: &'static str,
    shortcut: &'static str,
    description: &'static str,
    italic: bool,
    underline: bool,
    strike: bool,
    active: bool,
    enabled: bool,
    cx: &mut Context<NotesApp>,
    kind: TextStyleKind,
) -> AnyElement {
    let label_color = if !enabled {
        ONENOTE_INK_MUTED
    } else if active {
        ONENOTE_INK
    } else {
        ONENOTE_INK
    };

    let mut mark_el = div()
        .w(px(RIBBON_ICON))
        .text_size(px(RIBBON_ICON))
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(rgb(label_color))
        .h(px(29.0))
        .flex()
        .items_center()
        .justify_center();
    if italic {
        mark_el = mark_el.italic();
    }
    if underline {
        mark_el = mark_el.border_b_1().border_color(rgb(label_color));
    }
    if strike {
        mark_el = mark_el.relative().child(mark).child(
            div()
                .absolute()
                .top(px(8.0))
                .left(px(3.0))
                .w(px(RIBBON_PAD_H))
                .h(px(1.0))
                .bg(rgb(label_color)),
        );
    } else {
        mark_el = mark_el.child(mark);
    }

    let mut button = div()
        .id(id)
        .px(px(RIBBON_PAD_H))
        .py(px(RIBBON_PAD_V))
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(label_color))
        .tooltip(move |_window, cx| {
            cx.new(|_| FormatTooltip {
                label,
                shortcut,
                description,
            })
            .into()
        });

    if active && enabled {
        button = button.bg(rgb(ONENOTE_PAGE_SELECTED));
    } else if enabled {
        button = button
            .cursor_pointer()
            .hover(|style| style.bg(rgb(ONENOTE_BAR_LINE)));
    }

    button = button.child(mark_el);

    if enabled {
        button = button.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.font_style_menu_open = false;
                this.font_size_menu_open = false;
                this.font_color_menu_open = false;
                this.bg_color_menu_open = false;
                this.toggle_text_style(kind, cx);
                cx.stop_propagation();
            }),
        );
    }

    button.into_any_element()
}
