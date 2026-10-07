//! Undo and redo for the notebook that is open in the editor.
//!
//! `kind` names the edit. `snapshot` copies the document and puts it back. This file owns the
//! stacks. Call `record_edit` before changing notebook content. The maintenance rules, the
//! call sites, and what is intentionally left out are in `UNDO_REDO.md` at the repo root.

mod kind;
mod snapshot;

use std::time::{Duration, Instant};

use gpui::Context;

use super::NotesApp;
pub(crate) use kind::EditKind;

const MAX_STEPS: usize = 100;
const COALESCE_WINDOW: Duration = Duration::from_secs(1);

/// Stacks for the current edit session. Cleared when editing starts or the note is deleted.
pub(crate) struct History {
    undo: Vec<snapshot::EditSnapshot>,
    redo: Vec<snapshot::EditSnapshot>,
    open: Option<OpenEdit>,
    restoring: bool,
}

struct OpenEdit {
    kind: EditKind,
    last: Instant,
    stashed_redo: Vec<snapshot::EditSnapshot>,
}

impl Default for History {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            open: None,
            restoring: false,
        }
    }
}

impl History {
    fn push_undo(&mut self, snap: snapshot::EditSnapshot) {
        self.undo.push(snap);
        if self.undo.len() > MAX_STEPS {
            let extra = self.undo.len() - MAX_STEPS;
            self.undo.drain(0..extra);
        }
    }
}

impl NotesApp {
    pub(crate) fn can_undo(&self) -> bool {
        self.is_editing && !self.history.undo.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.is_editing && !self.history.redo.is_empty()
    }

    /// Drops every step. Used when an edit session starts or the open note is deleted.
    pub(crate) fn clear_history(&mut self) {
        self.history = History::default();
    }

    /// Saves the document as it is now, then the caller changes it.
    ///
    /// A new feature that changes notebook content calls this first with an `EditKind`.
    /// Restore does not call it.
    pub(crate) fn record_edit(&mut self, kind: EditKind) {
        if !self.is_editing || self.history.restoring {
            return;
        }
        let now = Instant::now();
        if self.history.open.as_ref().is_some_and(|open| {
            open.kind.merges_with(&kind) && now.duration_since(open.last) < COALESCE_WINDOW
        }) {
            if let Some(open) = self.history.open.as_mut() {
                open.last = now;
                if let (
                    EditKind::Typing {
                        block_id: open_block,
                        ..
                    },
                    EditKind::Typing {
                        block_id: Some(block_id),
                        ..
                    },
                ) = (&mut open.kind, &kind)
                {
                    if open_block.is_none() {
                        *open_block = Some(block_id.clone());
                    }
                }
            }
            return;
        }

        let snap = snapshot::capture(self);
        self.history.push_undo(snap);
        let stashed_redo = std::mem::take(&mut self.history.redo);
        self.history.open = Some(OpenEdit {
            kind,
            last: now,
            stashed_redo,
        });
    }

    /// Drops the checkpoint when the gesture or command did not change the document.
    pub(crate) fn discard_unchanged_edit(&mut self) {
        let Some(open) = self.history.open.take() else {
            return;
        };
        let current = snapshot::capture(self);
        let unchanged = self
            .history
            .undo
            .last()
            .is_some_and(|top| top.same_document(&current));
        if unchanged {
            self.history.undo.pop();
            self.history.redo = open.stashed_redo;
        }
    }

    /// Marks the start of a drag, resize, or inline-image gesture. One step for the gesture.
    pub(crate) fn begin_canvas_gesture(&mut self) {
        self.record_edit(EditKind::CanvasGesture);
    }

    /// Ends the gesture. A click that did not move anything is not an undo step.
    pub(crate) fn end_canvas_gesture(&mut self, cx: &mut Context<Self>) {
        let marked = self.history.open.is_some();
        let depth = self.history.undo.len();
        self.discard_unchanged_edit();
        if marked && self.history.undo.len() == depth {
            self.schedule_autosave(cx);
        }
    }

    pub(crate) fn undo(&mut self, cx: &mut Context<Self>) {
        if !self.can_undo() {
            return;
        }
        let Some(previous) = self.swap_with_undo() else {
            return;
        };
        self.apply_snapshot(previous, cx);
    }

    pub(crate) fn redo(&mut self, cx: &mut Context<Self>) {
        if !self.can_redo() {
            return;
        }
        let Some(next) = self.swap_with_redo() else {
            return;
        };
        self.apply_snapshot(next, cx);
    }

    fn swap_with_undo(&mut self) -> Option<snapshot::EditSnapshot> {
        let current = snapshot::capture(self);
        let previous = self.history.undo.pop()?;
        self.history.redo.push(current);
        if self.history.redo.len() > MAX_STEPS {
            let extra = self.history.redo.len() - MAX_STEPS;
            self.history.redo.drain(0..extra);
        }
        self.history.open = None;
        Some(previous)
    }

    fn swap_with_redo(&mut self) -> Option<snapshot::EditSnapshot> {
        let current = snapshot::capture(self);
        let next = self.history.redo.pop()?;
        self.history.push_undo(current);
        self.history.open = None;
        Some(next)
    }

    fn apply_snapshot(&mut self, snap: snapshot::EditSnapshot, cx: &mut Context<Self>) {
        self.history.restoring = true;
        snapshot::restore(self, snap);
        self.history.restoring = false;
        self.schedule_autosave(cx);
        cx.notify();
    }
}

#[cfg(test)]
mod tests;
