//! Page title, the rule under it, and the notebook date.

use gpui::{div, prelude::*, px, rgb, AnyElement};

use crate::app::NotesApp;
use crate::constants::typography::WEIGHT_NORMAL;
use crate::text::selection::calculate_text_width_for_font;

impl NotesApp {
    pub(crate) fn page_title_with_rule(&self, title: AnyElement, label: &str) -> AnyElement {
        let zoom = self.canvas_zoom.max(0.25);
        let text_w = calculate_text_width_for_font(
            label,
            self.page_heading_font_size,
            WEIGHT_NORMAL,
            self.font_type(),
        ) * zoom;
        let line_w = text_w.max(48.0 * zoom);
        let max_w = (self.window_w - self.layout_sidebar_w() - self.layout_page_sidebar_w() - 96.0)
            .max(160.0);
        let date = self
            .selected_note()
            .map(|note| note.created_at.clone())
            .unwrap_or_default();
        div()
            .flex()
            .flex_col()
            .gap(px(4.0 * zoom))
            .child(
                div()
                    .id("page-title-scroller")
                    .overflow_x_scroll()
                    .max_w(px(max_w))
                    .child(
                        div().flex().flex_col().w(px(line_w)).child(title).child(
                            div()
                                .mt(px(2.0))
                                .h(px(1.0))
                                .w(px(line_w))
                                .bg(rgb(crate::constants::colors::ONENOTE_INK_MUTED)),
                        ),
                    ),
            )
            .child(
                div()
                    .text_size(px(13.0 * zoom))
                    .text_color(rgb(crate::constants::colors::NOTE_HINT))
                    .child(date),
            )
            .into_any_element()
    }
}
