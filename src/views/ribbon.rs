//! Home ribbon for font, style, and formatting commands.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton, Render, Window};

use crate::app::formatting::TextStyleKind;
use crate::app::NotesApp;
use crate::constants::{
    colors::{TEXT_HINT, TEXT_PRIMARY, TEXT_SECONDARY},
    typography::BUTTON_FONT_SIZE,
};

impl NotesApp {
    /// Dark Home tab. Clicking it opens or closes the formatting menu.
    pub(crate) fn render_home_ribbon(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.home_menu_open;

        div()
            .w_full()
            .h(px(36.0))
            .flex_shrink_0()
            .bg(rgb(0x1e1e1e))
            .border_b_1()
            .border_color(rgb(0x2d2d2d))
            .flex()
            .flex_row()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .id("home-ribbon-tab")
                    .px(px(12.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(px(BUTTON_FONT_SIZE))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .bg(if open { rgb(0x2d2d2d) } else { rgb(0x1e1e1e) })
                    .text_color(if open { rgb(0xffffff) } else { rgb(TEXT_PRIMARY) })
                    .border_b_2()
                    .border_color(if open { rgb(0x0078d4) } else { rgb(0x1e1e1e) })
                    .hover(|style| style.bg(rgb(0x2d2d2d)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.home_menu_open = !this.home_menu_open;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    )
                    .child("Home")
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(if open { "▴" } else { "▾" }),
                    ),
            )
            .into_any_element()
    }

    /// Click-away layer and the formatting dropdown, painted above the note when Home is open.
    pub(crate) fn render_home_menu_layers(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        if !self.home_menu_open {
            return Vec::new();
        }

        let editing = self.is_editing;
        let bold_on = self.text_style_is_on(TextStyleKind::Bold);
        let italic_on = self.text_style_is_on(TextStyleKind::Italic);
        let underline_on = self.text_style_is_on(TextStyleKind::Underline);
        let strike_on = self.text_style_is_on(TextStyleKind::Strike);

        let mut menu = div()
            .absolute()
            .top(px(40.0))
            .left(px(8.0))
            .p(px(4.0))
            .bg(rgb(0x252526))
            .border_1()
            .border_color(rgb(0x3d3d3d))
            .rounded(px(6.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            });

        if !editing {
            menu = menu.child(
                div()
                    .px(px(8.0))
                    .py(px(6.0))
                    .text_size(px(BUTTON_FONT_SIZE))
                    .text_color(rgb(TEXT_HINT))
                    .child("Edit the note to format text"),
            );
        }

        menu = menu.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.0))
                .child(format_button(
                    "fmt-bold",
                    "B",
                    "Bold",
                    "Ctrl+B",
                    "Make your text bold.",
                    false,
                    false,
                    false,
                    bold_on,
                    editing,
                    cx,
                    TextStyleKind::Bold,
                ))
                .child(format_button(
                    "fmt-italic",
                    "I",
                    "Italic",
                    "Ctrl+I",
                    "Make your text italic.",
                    true,
                    false,
                    false,
                    italic_on,
                    editing,
                    cx,
                    TextStyleKind::Italic,
                ))
                .child(format_button(
                    "fmt-underline",
                    "U",
                    "Underline",
                    "Ctrl+U",
                    "Underline your text.",
                    false,
                    true,
                    false,
                    underline_on,
                    editing,
                    cx,
                    TextStyleKind::Underline,
                ))
                .child(format_button(
                    "fmt-strike",
                    "S",
                    "Strikethrough",
                    "Ctrl+Shift+X",
                    "Draw a line through your text.",
                    false,
                    false,
                    true,
                    strike_on,
                    editing,
                    cx,
                    TextStyleKind::Strike,
                )),
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
                        this.home_menu_open = false;
                        cx.notify();
                    }),
                )
                .into_any_element(),
            menu.into_any_element(),
        ]
    }
}

/// Hover card for a formatting icon: command name with shortcut, then a short description.
struct FormatTooltip {
    label: &'static str,
    shortcut: &'static str,
    description: &'static str,
}

impl Render for FormatTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p(px(8.0))
            .bg(rgb(0x252526))
            .border_1()
            .border_color(rgb(0x3d3d3d))
            .rounded(px(4.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .text_size(px(BUTTON_FONT_SIZE))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(TEXT_PRIMARY))
                    .child(format!("{} ({})", self.label, self.shortcut)),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(TEXT_SECONDARY))
                    .child(self.description),
            )
    }
}

fn format_button(
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
        TEXT_HINT
    } else if active {
        0xffffff
    } else {
        TEXT_PRIMARY
    };

    let mut mark_el = div()
        .w(px(18.0))
        .text_size(px(14.0))
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(rgb(label_color))
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
                .w(px(12.0))
                .h(px(1.0))
                .bg(rgb(label_color)),
        );
    } else {
        mark_el = mark_el.child(mark);
    }

    let mut button = div()
        .id(id)
        .w(px(28.0))
        .h(px(28.0))
        .rounded(px(4.0))
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
        button = button.bg(rgb(0x094771));
    } else if enabled {
        button = button
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x3d3d3d)));
    }

    button = button.child(mark_el);

    if enabled {
        button = button.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.toggle_text_style(kind, cx);
                cx.stop_propagation();
            }),
        );
    }

    button.into_any_element()
}
