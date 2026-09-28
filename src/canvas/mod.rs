//! Canvas surfaces for the open page.
//!
//! - `editor` — editable canvas shell (pan, drag, pointer routing)
//! - `header` — section tabs and inline heading editors
//! - `blocks` — text, image, and mixed blocks
//! - `text_segment` — one editable text segment
//! - `viewer` — read-only page canvas
//! - `viewer_text` — selection inside viewer text
//! - `text_editor` — reusable styled text field

pub(crate) mod blocks;
pub(crate) mod editor;
pub(crate) mod header;
pub(crate) mod text_editor;
pub(crate) mod text_segment;
pub(crate) mod viewer;
pub(crate) mod viewer_text;

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
