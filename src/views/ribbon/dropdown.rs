//! Custom ribbon dropdown popup with its own width.
//!
//! The popup is attached as a child of its trigger button but drawn through
//! `deferred(anchored(..))`, so it floats above the whole window instead of taking part in
//! the ribbon menu layout. Its width is whatever the caller asks for, its height follows the
//! content, and it is kept inside the window edges.

use gpui::{anchored, deferred, div, prelude::*, px, relative, rgb, Anchor, AnyElement, MouseButton};

use crate::constants::colors::{NOTE_PAGE, ONENOTE_BAR_LINE};

/// Gap kept between the popup and the window edges when it would overflow.
const WINDOW_MARGIN: f32 = 8.0;

/// Padding inside a ribbon dropdown panel.
pub(super) const DROPDOWN_PAD: f32 = 4.0;

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
                            .bg(rgb(NOTE_PAGE))
                            .border_1()
                            .border_color(rgb(ONENOTE_BAR_LINE))
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
