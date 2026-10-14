//! Page title, the rule under it, and the notebook date.

use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight, Window};

use crate::app::NotesApp;
use crate::text::shaping::{shaped_uniform_prefix, UniformFace};

impl NotesApp {
    /// Formatted creation date/time for the active page, with intelligent fallbacks.
    pub(crate) fn active_page_created_date(&self) -> String {
        // 1. Check if active page has an explicit created_at string
        let page_created = self
            .edit_content
            .as_ref()
            .or(self.view_content.as_ref())
            .and_then(|content| {
                let sec_id = self.active_section_id.as_ref()?;
                let page_id = self.active_page_id.as_ref()?;
                let sec = content.sections.iter().find(|s| s.id == *sec_id)?;
                let page = sec.pages.iter().find(|p| p.id == *page_id)?;
                page.created_at.clone().filter(|d| !d.trim().is_empty())
            });

        if let Some(date) = page_created {
            return date;
        }

        // 2. If page ID is a millisecond timestamp, format that timestamp
        if let Some(ref page_id) = self.active_page_id {
            let clean_id = page_id.strip_suffix("_page").unwrap_or(page_id);
            if let Ok(millis) = clean_id.parse::<i64>() {
                if let Some(dt) = chrono::DateTime::from_timestamp_millis(millis) {
                    let local_dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from(dt);
                    return local_dt.format("%b %d, %Y %I:%M %p").to_string();
                }
            }
        }

        // 3. Fall back to parent note's created_at if non-empty
        let note_created = self
            .selected_note()
            .map(|note| note.created_at.trim().to_string())
            .filter(|d| !d.is_empty());

        if let Some(date) = note_created {
            return date;
        }

        // 4. If note ID is a millisecond timestamp, format that timestamp
        if let Some(ref note_id) = self.selected_note_id {
            if let Ok(millis) = note_id.parse::<i64>() {
                if let Some(dt) = chrono::DateTime::from_timestamp_millis(millis) {
                    let local_dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from(dt);
                    return local_dt.format("%b %d, %Y %I:%M %p").to_string();
                }
            }
        }

        // 5. Fallback to current local time
        chrono::Local::now()
            .format("%b %d, %Y %I:%M %p")
            .to_string()
    }

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
        let date = self.active_page_created_date();
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
