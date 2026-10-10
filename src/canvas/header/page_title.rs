//! Page title, the rule under it, and the notebook date.

use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight, Window};

use crate::app::NotesApp;
use crate::text::shaping::{shaped_uniform_prefix, UniformFace};

impl NotesApp {
    pub(crate) fn page_title_with_rule(
        &self,
        window: &Window,
        title: AnyElement,
        label: &str,
        weight: FontWeight,
    ) -> AnyElement {
        let zoom = self.canvas_zoom.max(0.25);
        let text_w = shaped_uniform_prefix(
            window,
            label,
            UniformFace {
                family: self.font_family.as_str(),
                size: self.page_heading_font_size,
                weight,
                italic: false,
            },
            zoom,
        )
        .last()
        .copied()
        .unwrap_or(0.0)
            * zoom;
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
                                .bg(rgb(crate::constants::colors::onenote_ink_muted())),
                        ),
                    ),
            )
            .child(
                div()
                    .text_size(px(13.0 * zoom))
                    .text_color(rgb(crate::constants::colors::note_hint()))
                    .child(date),
            )
            .into_any_element()
    }
}
