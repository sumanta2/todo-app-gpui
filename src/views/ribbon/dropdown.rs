//! Custom ribbon dropdown popup with its own width.
//!
//! The popup is attached as a child of its trigger button but drawn through
//! `deferred(anchored(..))`, so it floats above the whole window instead of taking part in
//! the ribbon menu layout. Its width is whatever the caller asks for, its height follows the
//! content, and it is kept inside the window edges.

use gpui::{
    anchored, deferred, div, prelude::*, px, relative, rgb, Anchor, AnyElement, Context, MouseButton,
};

use crate::app::{NotesApp, RibbonPane};
use crate::constants::colors::{note_page, onenote_bar_line, onenote_ink};

/// Gap kept between the popup and the window edges when it would overflow.
const WINDOW_MARGIN: f32 = 8.0;

/// Padding inside a ribbon dropdown panel.
pub(super) const DROPDOWN_PAD: f32 = 4.0;

/// How the parent lays out the dropdown's children.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DropdownFlow {
    /// Each child is its own row.
    Column,
    /// Children share a line and move down when the next one would pass the panel width.
    Wrap,
}

/// One pin for the ribbon. It is shared: pinning either menu fills the same slot.
pub(super) fn menu_pin_button(
    menu: RibbonPane,
    pinned: bool,
    cx: &mut Context<NotesApp>,
) -> AnyElement {
    let id = match menu {
        RibbonPane::Home => "home-menu-pin",
        RibbonPane::View => "view-menu-pin",
    };
    div()
        .id(id)
        .absolute()
        .right(px(4.0))
        .bottom(px(4.0))
        .cursor_pointer()
        .w(px(22.0))
        .h(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.0))
        .hover(|style| style.bg(rgb(onenote_bar_line())))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                if this.ribbon_pinned && this.ribbon_pane == menu {
                    this.ribbon_pinned = false;
                } else {
                    this.ribbon_pinned = true;
                    this.ribbon_pane = menu;
                    this.home_menu_open = false;
                    this.view_menu_open = false;
                    this.zoom_menu_open = false;
                    this.font_style_menu_open = false;
                    this.font_size_menu_open = false;
                    this.font_color_menu_open = false;
                    this.bg_color_menu_open = false;
                }
                this.save_layout();
                cx.notify();
                cx.stop_propagation();
            }),
        )
        .child(
            div()
                .font_family("Segoe MDL2 Assets")
                .text_size(px(14.0))
                .text_color(rgb(onenote_ink()))
                .child(if pinned { "\u{E196}" } else { "\u{E718}" }),
        )
        .into_any_element()
}

/// Empty row of dropdown children. The parent picks `flow`; callers then add each child.
pub(super) fn dropdown_items(flow: DropdownFlow, gap: f32) -> gpui::Div {
    let items = div().w_full().flex().gap(px(gap));
    match flow {
        DropdownFlow::Column => items.flex_col(),
        DropdownFlow::Wrap => items.flex_row().flex_wrap(),
    }
}

/// Floating panel for an open dropdown. Add the returned element as a child of the trigger
/// button; the trigger must be `relative()` so the panel hangs from its bottom-left corner.
pub(super) fn dropdown_popup(width: f32, padding: f32, content: AnyElement) -> AnyElement {
    div()
        .absolute()
        .left(px(0.0))
        .top(relative(1.0))
        .child(
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .snap_to_window_with_margin(px(WINDOW_MARGIN))
                    .child(
                        div()
                            .occlude()
                            .w(px(width))
                            .py(px(padding))
                            .bg(rgb(note_page()))
                            .border_1()
                            .border_color(rgb(onenote_bar_line()))
                            .shadow_md()
                            .flex()
                            .flex_col()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();
                            })
                            .child(content),
                    ),
            )
            .with_priority(1),
        )
        .into_any_element()
}
