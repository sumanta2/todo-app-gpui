//! Create, select, and delete notebooks, and show or hide the sidebar.

use gpui::Context;

use crate::app::NotesApp;
use crate::models::Note;

impl NotesApp {
    /// Pins or unpins the left notebook list.
    pub(crate) fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        if self.selected_note().is_none() {
            self.is_sidebar_open = true;
            self.note_menu_open = false;
            cx.notify();
            return;
        }
        self.is_sidebar_open = !self.is_sidebar_open;
        if self.is_sidebar_open {
            self.note_menu_open = false;
        }
        self.save_layout();
        cx.notify();
    }

    /// The notebook list stays expanded when nothing is selected, so Create New Note stays reachable.
    pub(crate) fn sidebar_expanded(&self) -> bool {
        self.is_sidebar_open || self.selected_note().is_none()
    }

    /// Opens `id` as the current notebook and starts editing it.
    pub(crate) fn select_note(&mut self, id: String, cx: &mut Context<Self>) {
        if self.selected_note_id.as_ref() == Some(&id) {
            self.note_menu_open = false;
            cx.notify();
            return;
        }
        self.persist_edit();
        self.selected_note_id = Some(id);
        self.active_section_id = None;
        self.active_page_id = None;
        self.view_content = None;
        self.page_items_cache = None;
        self.image_cache.clear();
        self.note_menu_open = false;
        self.start_edit(cx);
        cx.notify();
    }

    /// Returns the currently selected note, if any.
    pub(crate) fn selected_note(&self) -> Option<&Note> {
        if let Some(ref id) = self.selected_note_id {
            self.notes.iter().find(|n| n.id == *id)
        } else {
            None
        }
    }

    /// Creates a new blank note, saves it immediately, and opens it in edit mode.
    pub(crate) fn create_note(&mut self, cx: &mut Context<Self>) {
        let id = chrono::Local::now().timestamp_millis().to_string();
        let created_at = chrono::Local::now()
            .format("%b %d, %Y %I:%M %p")
            .to_string();
        let new_note = Note {
            id: id.clone(),
            heading: "Untitled Note".to_owned(),
            body: String::new(),
            created_at,
            images: None,
            links: None,
            format: None,
        };
        self.notes.push(new_note);
        self.selected_note_id = Some(id);
        self.save_notes();
        self.start_edit(cx);
        cx.notify();
    }

    /// Deletes a note from the in-memory list and saves the updated collection.
    pub(crate) fn delete_note(&mut self, id: String, cx: &mut Context<Self>) {
        self.notes.retain(|n| n.id != id);
        if self.selected_note_id == Some(id) {
            self.clear_history();
            self.selected_note_id = None;
            self.is_editing = false;
        }
        self.save_notes();
        cx.notify();
    }
}
