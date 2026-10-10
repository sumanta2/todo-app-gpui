//! Home ribbon menu: clipboard, font, styles, color, and paragraph layout.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context, MouseButton};

use crate::app::formatting::TextStyleKind;
use crate::app::NotesApp;
use crate::constants::{
    colors::{
        bg_color_options, font_color_options, note_page, onenote_bar_line, onenote_ink,
        onenote_ink_muted,
    },
    typography::{
        font_size_label,         font_style_name, RIBBON_DROPDOWN_W, RIBBON_GAP, RIBBON_ICON, RIBBON_PAD_H,
        RIBBON_PAD_V, RIBBON_TEXT,
    },
};

use super::controls::{
    clip_button, color_button, color_list, font_size_list, font_style_list, format_button,
    layout_button, ribbon_dropdown, ClipAction, LayoutAction,
};
use super::dropdown::{dropdown_popup, menu_pin_button, DropdownFlow, DROPDOWN_PAD};
use crate::app::RibbonPane;

impl NotesApp {
    
    pub(crate) fn render_home_menu_layers(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        if self.ribbon_pinned || !self.home_menu_open {
            return Vec::new();
        }
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
                        this.font_style_menu_open = false;
                        this.font_size_menu_open = false;
                        this.font_color_menu_open = false;
                        this.bg_color_menu_open = false;
                        cx.notify();
                    }),
                )
                .into_any_element(),
            div()
                .absolute()
                .top(px(40.0))
                .child(self.home_menu_bar(cx))
                .into_any_element(),
        ]
    }

    pub(super) fn home_menu_bar(&mut self, cx: &mut Context<Self>) -> AnyElement {

        let editing = self.is_editing;
        let bold_on = self.text_style_is_on(TextStyleKind::Bold);
        let italic_on = self.text_style_is_on(TextStyleKind::Italic);
        let underline_on = self.text_style_is_on(TextStyleKind::Underline);
        let strike_on = self.text_style_is_on(TextStyleKind::Strike);

        let docked = self.ribbon_pinned;
        let mut menu = div()
            .relative()
            .pr(px(32.0))
            .bg(rgb(note_page()))
            .left(px(0.0))
            .border_color(rgb(onenote_bar_line()))
            .flex()
            .flex_col()
            .gap(px(RIBBON_GAP))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            });
        menu = if docked {
            menu.w_full().border_b_1().border_l_1().border_r_1()
        } else {
            menu.border_1()
        };

        if !editing {
            menu = menu.child(
                div()
                    .px(px(RIBBON_PAD_H))
                    .py(px(RIBBON_PAD_V))
                    .text_size(px(RIBBON_TEXT))
                    .text_color(rgb(onenote_ink_muted()))
                    .child("Edit the note to format text"),
            );
        }

        let font_label = font_style_name(self.typing_font_family).to_string();
        let size_label = font_size_label(self.typing_font_size_px);
        let font_open = self.font_style_menu_open;
        let size_open = self.font_size_menu_open;
        let font_color_open = self.font_color_menu_open;
        let bg_color_open = self.bg_color_menu_open;
        let shown_font_color = self.shown_font_color();
        let shown_bg_color = self.shown_bg_color();
        let align = self.current_align();
        let can_copy = self.has_clipboard_selection();
        let can_cut = editing && can_copy;
        let can_paste = editing;

        let font_body = font_style_list(self.typing_font_family, DropdownFlow::Column, cx);
        let font_popup = font_open.then(|| dropdown_popup(132.0, 0.0, font_body));
        let size_body = font_size_list(self.typing_font_size_px, DropdownFlow::Column, cx);
        let size_popup = size_open.then(|| dropdown_popup(56.0, 0.0, size_body));
        let font_colors = font_color_options();
        let font_color_body = color_list(
            &font_colors,
            shown_font_color,
            true,
            DropdownFlow::Wrap,
            cx,
        );
        let font_color_popup = font_color_open.then(|| {
            dropdown_popup(RIBBON_DROPDOWN_W, DROPDOWN_PAD * 0.5, font_color_body)
        });
        let bg_colors = bg_color_options();
        let bg_color_body = color_list(
            &bg_colors,
            shown_bg_color,
            false,
            DropdownFlow::Wrap,
            cx,
        );
        let bg_color_popup =
            bg_color_open.then(|| dropdown_popup(RIBBON_DROPDOWN_W, DROPDOWN_PAD * 0.5, bg_color_body));

        // Clipboard buttons
        menu = menu.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .child(clip_button(
                    "fmt-paste",
                    "\u{E77F}",
                    "Paste",
                    "Ctrl+V",
                    "Insert clipboard text or an image into the note.",
                    can_paste,
                    cx,
                    ClipAction::Paste,
                ))
                .child(clip_button(
                    "fmt-cut",
                    "\u{E8C6}",
                    "Cut",
                    "Ctrl+X",
                    "Remove the selection and put it on the clipboard.",
                    can_cut,
                    cx,
                    ClipAction::Cut,
                ))
                .child(clip_button(
                    "fmt-copy",
                    "\u{E8C8}",
                    "Copy",
                    "Ctrl+C",
                    "Copy the selection to the clipboard.",
                    can_copy,
                    cx,
                    ClipAction::Copy,
                ))
                .child(
                    div()
                        .w(px(1.0))
                        .h(px(RIBBON_ICON))
                        .mx(px(4.0))
                        .bg(rgb(onenote_bar_line())),
                )
                .child(ribbon_dropdown(
                    "fmt-font-style",
                    font_label,
                    132.0,
                    font_open,
                    font_popup,
                    cx,
                    |this, _, _, cx| {
                        this.font_style_menu_open = !this.font_style_menu_open;
                        this.font_size_menu_open = false;
                        this.font_color_menu_open = false;
                        this.bg_color_menu_open = false;
                        cx.notify();
                        cx.stop_propagation();
                    },
                ))
                .child(ribbon_dropdown(
                    "fmt-font-size",
                    size_label,
                    56.0,
                    size_open,
                    size_popup,
                    cx,
                    |this, _, _, cx| {
                        this.font_size_menu_open = !this.font_size_menu_open;
                        this.font_style_menu_open = false;
                        this.font_color_menu_open = false;
                        this.bg_color_menu_open = false;
                        cx.notify();
                        cx.stop_propagation();
                    },
                ))
                .child(
                    div()
                        .w(px(1.0))
                        .h(px(RIBBON_ICON))
                        .mx(px(4.0))
                        .bg(rgb(onenote_bar_line())),
                )
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
                ))
                .child(color_button(
                    "fmt-font-color",
                    "A",
                    shown_font_color,
                    onenote_ink(),
                    true,
                    font_color_open,
                    font_color_popup,
                    cx,
                    |this, _, _, cx| {
                        this.font_color_menu_open = !this.font_color_menu_open;
                        this.bg_color_menu_open = false;
                        this.font_style_menu_open = false;
                        this.font_size_menu_open = false;
                        cx.notify();
                        cx.stop_propagation();
                    },
                ))
                .child(color_button(
                    "fmt-bg-color",
                    "\u{E7E6}",
                    shown_bg_color,
                    0,
                    false,
                    bg_color_open,
                    bg_color_popup,
                    cx,
                    |this, _, _, cx| {
                        this.bg_color_menu_open = !this.bg_color_menu_open;
                        this.font_color_menu_open = false;
                        this.font_style_menu_open = false;
                        this.font_size_menu_open = false;
                        cx.notify();
                        cx.stop_propagation();
                    },
                ))
                .child(clip_button(
                    "fmt-delete",
                    "\u{E74D}",
                    "Delete",
                    "Delete",
                    "Delete the selected text or image, or the character after the cursor.",
                    editing || self.selected_inline_image.is_some(),
                    cx,
                    ClipAction::Delete,
                ))
                .child(layout_button(
                    "fmt-indent-dec",
                    "\u{E7EB}",
                    "Decrease Indent",
                    "Shift the selected lines or image left.",
                    false,
                    editing || self.selected_inline_image.is_some(),
                    cx,
                    LayoutAction::DecreaseIndent,
                ))
                .child(layout_button(
                    "fmt-indent-inc",
                    "\u{E7EC}",
                    "Increase Indent",
                    "Shift the selected lines or image right.",
                    false,
                    editing || self.selected_inline_image.is_some(),
                    cx,
                    LayoutAction::IncreaseIndent,
                ))
                .child(layout_button(
                    "fmt-align-left",
                    "\u{E8E4}",
                    "Align Left",
                    "Align the selected lines or image to the left.",
                    align == 0,
                    editing || self.selected_inline_image.is_some(),
                    cx,
                    LayoutAction::AlignLeft,
                ))
                .child(layout_button(
                    "fmt-align-center",
                    "\u{E8E3}",
                    "Align Center",
                    "Center the selected lines or image.",
                    align == 1,
                    editing || self.selected_inline_image.is_some(),
                    cx,
                    LayoutAction::AlignCenter,
                ))
                .child(layout_button(
                    "fmt-align-right",
                    "\u{E8E2}",
                    "Align Right",
                    "Align the selected lines or image to the right.",
                    align == 2,
                    editing || self.selected_inline_image.is_some(),
                    cx,
                    LayoutAction::AlignRight,
                )),
        );
        menu.child(menu_pin_button(RibbonPane::Home, docked, cx))
            .into_any_element()
    }
}
