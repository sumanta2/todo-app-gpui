//! Kinds of document edits. Typing in one field merges; every other kind is its own step.

use crate::models::ActiveField;

/// What the user just changed. New features pick a kind and pass it to `record_edit`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum EditKind {
    /// Keystrokes in one field. `block_id` is the canvas text box, when there is one.
    Typing {
        field: ActiveField,
        block_id: Option<String>,
    },
    /// Bold, italic, underline, strikethrough, color, indent, or alignment.
    Format,
    /// Cut, paste, or delete from the clipboard commands.
    Clipboard,
    /// Insert an image onto the page.
    Image,
    /// Move or resize a box, or move or zoom an image inside a text box.
    CanvasGesture,
    /// Delete a text box, image, or mixed box.
    Canvas,
    /// Add or delete a section or page.
    Structure,
}

impl EditKind {
    pub(crate) fn typing(field: ActiveField, block_id: Option<String>) -> Self {
        EditKind::Typing { field, block_id }
    }

    /// Consecutive typing in the same field merges. A box created by the first keystroke
    /// starts with no block id and then adopts the id of the box it just opened.
    pub(crate) fn merges_with(&self, next: &EditKind) -> bool {
        match (self, next) {
            (
                EditKind::Typing {
                    field: left_field,
                    block_id: left_block,
                },
                EditKind::Typing {
                    field: right_field,
                    block_id: right_block,
                },
            ) => left_field == right_field && (left_block == right_block || left_block.is_none()),
            _ => false,
        }
    }
}
