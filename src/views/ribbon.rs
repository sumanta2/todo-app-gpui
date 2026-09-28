use gpui::{div, prelude::*, px, rgb, AnyElement, Context, IntoElement, MouseButton};

use crate::app::actions::TextStyleKind;
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
            .w(px(196.0))
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

        menu = menu
            .child(format_row(
                "fmt-bold",
                "B",
                "Bold",
                "Ctrl+B",
                false,
                false,
                false,
                bold_on,
                editing,
                cx,
                TextStyleKind::Bold,
            ))
            .child(format_row(
                "fmt-italic",
                "I",
                "Italic",
                "Ctrl+I",
                true,
                false,
                false,
                italic_on,
                editing,
                cx,
                TextStyleKind::Italic,
            ))
            .child(format_row(
                "fmt-underline",
                "U",
                "Underline",
                "Ctrl+U",
                false,
                true,
                false,
                underline_on,
                editing,
                cx,
                TextStyleKind::Underline,
            ))
            .child(format_row(
                "fmt-strike",
                "S",
                "Strikethrough",
                "Ctrl+Shift+X",
                false,
                false,
                true,
                strike_on,
                editing,
                cx,
                TextStyleKind::Strike,
            ));

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

fn format_row(
    id: &'static str,
    mark: &'static str,
    label: &'static str,
    shortcut: &'static str,
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
        .text_color(rgb(label_color));
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
                .left(px(0.0))
                .w(px(12.0))
                .h(px(1.0))
                .bg(rgb(label_color)),
        );
    } else {
        mark_el = mark_el.child(mark);
    }

    let mut row = div()
        .id(id)
        .px(px(8.0))
        .py(px(5.0))
        .rounded(px(4.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .text_size(px(BUTTON_FONT_SIZE))
        .text_color(rgb(label_color));

    if active && enabled {
        row = row.bg(rgb(0x094771));
    } else if enabled {
        row = row.cursor_pointer().hover(|style| style.bg(rgb(0x3d3d3d)));
    }

    row = row.child(mark_el).child(div().flex_1().child(label)).child(
        div()
            .text_size(px(11.0))
            .text_color(rgb(if enabled { TEXT_SECONDARY } else { TEXT_HINT }))
            .child(shortcut),
    );

    if enabled {
        row = row.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.toggle_text_style(kind, cx);
                cx.stop_propagation();
            }),
        );
    }

    row.into_any_element()
}
