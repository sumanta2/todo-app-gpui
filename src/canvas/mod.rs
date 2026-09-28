pub(crate) mod editor;
pub(crate) mod editor_header;
pub(crate) mod editor_items;
pub(crate) mod text_editor;
pub(crate) mod viewer;

use gpui::{prelude::*, Entity, IntoElement};

use crate::app::NotesApp;

/// Invisible full-size layer that records the canvas container's top edge in window coordinates.
///
/// Canvas hit-testing measures click positions relative to `canvas_top_y`, so it must follow the
/// real layout whenever header or heading font sizes change.
pub(crate) fn canvas_top_tracker(entity: Entity<NotesApp>) -> impl IntoElement {
    gpui::canvas(
        move |bounds, _, cx| {
            entity.update(cx, |this, _| this.canvas_top_y = bounds.origin.y.as_f32());
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}
