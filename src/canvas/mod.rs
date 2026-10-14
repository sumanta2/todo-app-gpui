//! Canvas surfaces for the open page.
//!
//! - `editor` — editable canvas shell (pan, drag, pointer routing)
//! - `header` — section tabs, section name, page title, and page heading
//! - `blocks` — text, image, and mixed blocks
//! - `text_segment` — one editable text segment
//! - `viewer` — read-only page canvas
//! - `viewer_text` — selection inside viewer text
//! - `text_editor` — reusable styled text field

pub(crate) mod blocks;
pub(crate) mod editor;
pub(crate) mod header;
pub(crate) mod selection_overlay;
pub(crate) mod text_editor;
pub(crate) mod text_segment;
pub(crate) mod viewer;
pub(crate) mod viewer_text;

use gpui::{prelude::*, Entity, IntoElement};

use crate::app::NotesApp;

/// Which layout edge a shared bounds canvas should write.
///
/// One canvas type covers every call site. A second `gpui::canvas` closure would
/// monomorphize GPUI's canvas element again and add a large chunk of binary size.
pub(crate) enum BoundsKind {
    /// Canvas container top, used to convert window clicks into page coordinates.
    CanvasTop,
    /// Section tab origin and width, used while reordering tabs.
    SectionTab(String),
    /// Page-row origin and height, used while reordering pages.
    PageRow(String),
}

/// Invisible full-size layer that records one element's laid-out bounds.
pub(crate) fn bounds_tracker(entity: Entity<NotesApp>, kind: BoundsKind) -> impl IntoElement {
    gpui::canvas(
        move |bounds, _, cx| {
            let x = bounds.origin.x.as_f32();
            let y = bounds.origin.y.as_f32();
            let w = bounds.size.width.as_f32();
            let h = bounds.size.height.as_f32();
            entity.update(cx, |this, _| match &kind {
                BoundsKind::CanvasTop => this.canvas_top_y = y,
                BoundsKind::SectionTab(id) => {
                    this.section_tab_bounds.insert(id.clone(), (x, w));
                }
                BoundsKind::PageRow(id) => {
                    this.page_row_bounds.insert(id.clone(), (y, h));
                }
            });
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}

/// Invisible full-size layer that records the canvas container's top edge in window coordinates.
///
/// Canvas hit-testing measures click positions relative to `canvas_top_y`, so it must follow the
/// real layout whenever header or heading font sizes change.
pub(crate) fn canvas_top_tracker(entity: Entity<NotesApp>) -> impl IntoElement {
    bounds_tracker(entity, BoundsKind::CanvasTop)
}
