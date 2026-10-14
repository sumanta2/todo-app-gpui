//! Sections and pages inside the notebook that is currently open.

use gpui::Context;

use crate::app::NotesApp;
use crate::models::{
    load_canvas_items, load_note_content, save_canvas_items, CanvasItem, NotePage, NoteSection,
};

impl NotesApp {
    /// Ensures the current active section and page still exist for the selected note.
    ///
    /// This keeps the UI focused on valid content even after a note is edited, loaded, or
    /// switched to a different section/page.
    pub(crate) fn initialize_active_section_page(&mut self) {
        if let Some(ref id) = self.selected_note_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == *id) {
                let content = if self.is_editing {
                    self.edit_content
                        .clone()
                        .unwrap_or_else(|| load_note_content(&note.body, &note.images))
                } else {
                    load_note_content(&note.body, &note.images)
                };

                if let Some(first_sec) = content.sections.first() {
                    if self.active_section_id.is_none()
                        || !content
                            .sections
                            .iter()
                            .any(|s| Some(&s.id) == self.active_section_id.as_ref())
                    {
                        self.active_section_id = Some(first_sec.id.clone());
                    }

                    let active_sec = content
                        .sections
                        .iter()
                        .find(|s| Some(&s.id) == self.active_section_id.as_ref())
                        .unwrap_or(first_sec);
                    if let Some(first_page) = active_sec.pages.first() {
                        if self.active_page_id.is_none()
                            || !active_sec
                                .pages
                                .iter()
                                .any(|p| Some(&p.id) == self.active_page_id.as_ref())
                        {
                            self.active_page_id = Some(first_page.id.clone());
                        }
                    } else {
                        self.active_page_id = None;
                    }
                } else {
                    self.active_section_id = None;
                    self.active_page_id = None;
                }
                if !self.is_editing {
                    self.view_content = Some(content);
                }
            }
        }
    }

    /// Writes the current in-memory page editor state back into the editable content model.
    ///
    /// This is used before switching content, saving, or leaving edit mode so the page's
    /// section name, heading, and canvas bodies all stay in sync.
    pub(crate) fn sync_current_page_state(&mut self) {
        self.sync_active_text_block();

        if let Some(ref mut content) = self.edit_content {
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter_mut().find(|s| s.id == *sec_id) {
                    section.name = self.edit_section_name.clone();

                    if let Some(ref page_id) = self.active_page_id {
                        if let Some(page) = section.pages.iter_mut().find(|p| p.id == *page_id) {
                            page.name = self.edit_heading.clone();

                            self.edit_canvas_items.retain(|item| match item {
                                CanvasItem::Text(t) => !t.text.trim().is_empty(),
                                CanvasItem::Image(_) => true,
                                CanvasItem::Mixed(m) => m.blocks.iter().any(|b| match b {
                                    crate::models::ContentBlock::Image { .. } => true,
                                    crate::models::ContentBlock::Text { text, .. } => {
                                        !text.trim().is_empty()
                                    }
                                }),
                            });

                            page.body = save_canvas_items(&self.edit_canvas_items);

                            let mut saved_images = Vec::new();
                            for item in &self.edit_canvas_items {
                                match item {
                                    CanvasItem::Image(img) => saved_images.push(img.path.clone()),
                                    CanvasItem::Mixed(m) => {
                                        for b in &m.blocks {
                                            if let crate::models::ContentBlock::Image {
                                                path, ..
                                            } = b
                                            {
                                                saved_images.push(path.clone());
                                            }
                                        }
                                    }
                                    CanvasItem::Text(_) => {}
                                }
                            }
                            self.edit_images = saved_images.clone();
                            page.images = Some(saved_images);
                        }
                    }
                }
            }
        }
    }

    /// Switches the active note to a specific page inside the current section.
    ///
    /// When the app is editing, the current page state is saved first and then the editor UI
    /// is rehydrated from the chosen page.
    pub(crate) fn switch_to_page(
        &mut self,
        section_id: String,
        page_id: String,
        cx: &mut Context<Self>,
    ) {
        if self.is_editing {
            self.persist_edit();
            self.sync_current_page_state();

            self.active_section_id = Some(section_id.clone());
            self.active_page_id = Some(page_id.clone());
            self.initialize_active_section_page();

            if let Some(ref content) = self.edit_content {
                if let Some(section) = content.sections.iter().find(|s| s.id == section_id) {
                    self.edit_section_name = section.name.clone();
                    self.edit_section_name_cursor = self.edit_section_name.chars().count();
                    self.edit_section_name_anchor = None;

                    if let Some(page) = section.pages.iter().find(|p| p.id == page_id) {
                        self.edit_canvas_items = load_canvas_items(&page.body);
                        self.edit_images = page.images.clone().unwrap_or_default();

                        self.edit_heading = page.name.clone();
                        self.edit_heading_cursor = self.edit_heading.chars().count();
                        self.edit_heading_anchor = None;

                        self.active_text_block_id = None;
                        self.pending_caret = None;
                        self.active_block_index = None;
                        self.edit_body = String::new();
                        self.reset_body_styles();
                        self.edit_body_cursor = 0;
                        self.edit_body_anchor = None;
                    }
                }
            }
        } else {
            self.active_section_id = Some(section_id);
            self.active_page_id = Some(page_id);
            self.initialize_active_section_page();
        }

        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.is_panning = false;

        cx.notify();
    }

    /// Switches the current page selection to the first page in the given section.
    pub(crate) fn switch_to_section(&mut self, section_id: String, cx: &mut Context<Self>) {
        let content = if self.is_editing {
            self.edit_content.clone().unwrap_or_else(|| {
                let note = self.selected_note().unwrap();
                load_note_content(&note.body, &note.images)
            })
        } else {
            let note = self.selected_note().unwrap();
            load_note_content(&note.body, &note.images)
        };

        if let Some(section) = content.sections.iter().find(|s| s.id == section_id) {
            if let Some(first_page) = section.pages.first() {
                self.switch_to_page(section_id, first_page.id.clone(), cx);
            }
        }
    }

    /// Adds a new section to the currently edited note and immediately opens its first page.
    pub(crate) fn add_section(&mut self, cx: &mut Context<Self>) {
        if self.is_editing && self.edit_content.is_some() {
            self.record_edit(crate::app::history::EditKind::Structure);
        }
        if let Some(ref mut content) = self.edit_content {
            let new_sec_id = chrono::Local::now().timestamp_millis().to_string();
            let new_page_id = format!("{}_page", new_sec_id);
            let created_at = chrono::Local::now()
                .format("%b %d, %Y %I:%M %p")
                .to_string();
            let new_section = NoteSection {
                id: new_sec_id.clone(),
                name: format!("Section {}", content.sections.len() + 1),
                pages: vec![NotePage {
                    id: new_page_id.clone(),
                    name: "Page 1".to_string(),
                    body: String::new(),
                    images: None,
                    created_at: Some(created_at),
                }],
            };
            content.sections.push(new_section);
            self.note_dirty = true;
            self.switch_to_page(new_sec_id, new_page_id, cx);
        }
    }

    /// Removes a section from the current editing content if more than one section remains.
    pub(crate) fn delete_section(&mut self, section_id: String, cx: &mut Context<Self>) {
        let removes = self.is_editing
            && self
                .edit_content
                .as_ref()
                .is_some_and(|content| content.sections.len() > 1);
        if removes {
            self.record_edit(crate::app::history::EditKind::Structure);
        }
        let mut should_switch = None;
        if let Some(ref mut content) = self.edit_content {
            if content.sections.len() > 1 {
                content.sections.retain(|s| s.id != section_id);
                if self.active_section_id == Some(section_id) {
                    if let Some(first_sec) = content.sections.first() {
                        if let Some(first_page) = first_sec.pages.first() {
                            should_switch = Some((first_sec.id.clone(), first_page.id.clone()));
                        }
                    }
                }
            }
        }
        self.note_dirty = true;
        if let Some((sec_id, page_id)) = should_switch {
            self.switch_to_page(sec_id, page_id, cx);
        } else {
            self.persist_edit();
            cx.notify();
        }
    }

    /// Adds a new page to the active section and immediately switches to it.
    pub(crate) fn add_page(&mut self, cx: &mut Context<Self>) {
        let adds = self.is_editing
            && match (&self.edit_content, &self.active_section_id) {
                (Some(content), Some(section_id)) => {
                    content.sections.iter().any(|section| section.id == *section_id)
                }
                _ => false,
            };
        if adds {
            self.record_edit(crate::app::history::EditKind::Structure);
        }
        if let Some(ref mut content) = self.edit_content {
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter_mut().find(|s| s.id == *sec_id) {
                    let new_page_id = chrono::Local::now().timestamp_millis().to_string();
                    let created_at = chrono::Local::now()
                        .format("%b %d, %Y %I:%M %p")
                        .to_string();
                    let new_page = NotePage {
                        id: new_page_id.clone(),
                        name: format!("Page {}", section.pages.len() + 1),
                        body: String::new(),
                        images: None,
                        created_at: Some(created_at),
                    };
                    section.pages.push(new_page);
                    self.note_dirty = true;
                    self.switch_to_page(sec_id.clone(), new_page_id, cx);
                }
            }
        }
    }

    /// Deletes a page from the active section while preserving the section when possible.
    pub(crate) fn delete_page(&mut self, page_id: String, cx: &mut Context<Self>) {
        let removes = self.is_editing
            && match (&self.edit_content, &self.active_section_id) {
                (Some(content), Some(section_id)) => content
                    .sections
                    .iter()
                    .find(|section| section.id == *section_id)
                    .is_some_and(|section| section.pages.len() > 1),
                _ => false,
            };
        if removes {
            self.record_edit(crate::app::history::EditKind::Structure);
        }
        let mut should_switch: Option<(String, String)> = None;
        if let Some(ref mut content) = self.edit_content {
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter_mut().find(|s| s.id == *sec_id) {
                    if section.pages.len() > 1 {
                        section.pages.retain(|p: &NotePage| p.id != page_id);
                        if self.active_page_id == Some(page_id) {
                            if let Some(first_page) = section.pages.first() {
                                should_switch = Some((sec_id.clone(), first_page.id.clone()));
                            }
                        }
                    }
                }
            }
        }
        self.note_dirty = true;
        if let Some((sec_id, page_id)) = should_switch {
            self.switch_to_page(sec_id, page_id, cx);
        } else {
            self.persist_edit();
            cx.notify();
        }
    }

    /// Reorders a page within the active section to the specified target slot.
    ///
    /// Slot 0 places the page at the very top.
    /// Slot `pages.len()` places the page at the very bottom.
    /// If the page actually changes position, an undo snapshot is recorded and changes are persisted.
    pub(crate) fn move_page(&mut self, page_id: &str, target_slot: usize) -> bool {
        if self.is_editing {
            self.sync_current_page_state();
        }

        let Some(sec_id) = self.active_section_id.clone() else {
            return false;
        };

        // Check whether the page will actually move before calling record_edit
        let will_move = if self.is_editing {
            if let Some(ref content) = self.edit_content {
                if let Some(section) = content.sections.iter().find(|s| s.id == sec_id) {
                    if let Some(source_idx) = section.pages.iter().position(|p| p.id == page_id) {
                        let num_pages = section.pages.len();
                        let target_slot = target_slot.min(num_pages);
                        target_slot != source_idx && target_slot != source_idx + 1
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if will_move {
            self.record_edit(crate::app::history::EditKind::Structure);
        }

        let mut changed = false;

        if let Some(ref mut content) = self.edit_content {
            if let Some(section) = content.sections.iter_mut().find(|s| s.id == sec_id) {
                if reorder_pages_vec(&mut section.pages, page_id, target_slot) {
                    self.note_dirty = true;
                    self.persist_edit();
                    changed = true;
                }
            }
        }

        if let Some(ref mut content) = self.view_content {
            if let Some(section) = content.sections.iter_mut().find(|s| s.id == sec_id) {
                if reorder_pages_vec(&mut section.pages, page_id, target_slot) {
                    changed = true;
                }
            }
        }

        changed
    }

    /// Updates the drag state of a page in the pages list as the pointer moves.
    ///
    /// Determines whether the drag threshold has been passed and calculates the target
    /// drop slot based on vertical pointer position relative to page rows.
    pub(crate) fn update_page_drag(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let Some(ref _source_id) = self.page_drag_source_id else {
            return false;
        };
        let Some((start_x, start_y)) = self.page_drag_start_pos else {
            return false;
        };

        if !self.is_dragging_page {
            let dy = (mouse_y - start_y).abs();
            let dx = (mouse_x - start_x).abs();
            if dy > 4.0 || dx > 4.0 {
                self.is_dragging_page = true;
            } else {
                return false;
            }
        }

        let active_pages: Vec<String> = if let Some(ref content) = self.edit_content {
            content
                .sections
                .iter()
                .find(|s| Some(&s.id) == self.active_section_id.as_ref())
                .map(|s| s.pages.iter().map(|p| p.id.clone()).collect())
                .unwrap_or_default()
        } else if let Some(ref content) = self.view_content {
            content
                .sections
                .iter()
                .find(|s| Some(&s.id) == self.active_section_id.as_ref())
                .map(|s| s.pages.iter().map(|p| p.id.clone()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        if active_pages.is_empty() {
            return false;
        }

        // Default to the very bottom slot
        let mut new_slot = active_pages.len();

        for (idx, page_id) in active_pages.iter().enumerate() {
            if let Some(&(top_y, height)) = self.page_row_bounds.get(page_id) {
                let midpoint = top_y + height * 0.5;
                if mouse_y < midpoint {
                    new_slot = idx;
                    break;
                }
            }
        }

        if self.page_drop_slot != Some(new_slot) {
            self.page_drop_slot = Some(new_slot);
            return true;
        }

        false
    }

    /// Concludes an active page drag-and-drop gesture and applies the reorder if valid.
    pub(crate) fn finish_page_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let was_dragging = self.is_dragging_page;
        let source_id = self.page_drag_source_id.take();
        let target_slot = self.page_drop_slot.take();
        self.page_drag_start_pos = None;
        self.is_dragging_page = false;

        if was_dragging {
            if let (Some(source_id), Some(slot)) = (source_id, target_slot) {
                let moved = self.move_page(&source_id, slot);
                if moved {
                    cx.notify();
                }
                return moved;
            }
        }
        false
    }

    /// Reorders a section within the current note to the specified target slot.
    ///
    /// Slot 0 places the section at the leftmost position.
    /// Slot `sections.len()` places the section at the rightmost position.
    ///
    /// Flow:
    /// 1. Syncs active page editor state before mutating structure.
    /// 2. Performs a pre-check without mutable borrow to see if the move modifies the section order.
    /// 3. Records an undo snapshot (`EditKind::Structure`) only if the section actually changes position.
    /// 4. Mutably updates sections vector via `reorder_sections_vec`, marks dirty, and autosaves.
    pub(crate) fn move_section(&mut self, section_id: &str, target_slot: usize) -> bool {
        if self.is_editing {
            self.sync_current_page_state();
        }

        // Check whether the section will actually move before calling record_edit.
        // This avoids creating empty/redundant undo entries when dropping back into the same place.
        let will_move = if self.is_editing {
            if let Some(ref content) = self.edit_content {
                if let Some(source_idx) = content.sections.iter().position(|s| s.id == section_id) {
                    let num_sections = content.sections.len();
                    let target_slot = target_slot.min(num_sections);
                    target_slot != source_idx && target_slot != source_idx + 1
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if will_move {
            self.record_edit(crate::app::history::EditKind::Structure);
        }

        let mut changed = false;

        // Apply to editable content model (edit mode)
        if let Some(ref mut content) = self.edit_content {
            if reorder_sections_vec(&mut content.sections, section_id, target_slot) {
                self.note_dirty = true;
                self.persist_edit();
                changed = true;
            }
        }

        // Also update view content model (viewer mode) if open
        if let Some(ref mut content) = self.view_content {
            if reorder_sections_vec(&mut content.sections, section_id, target_slot) {
                changed = true;
            }
        }

        changed
    }

    /// Updates the drag state of a section tab as the pointer moves horizontally.
    ///
    /// Functionality:
    /// - Checks for an initial 4px displacement threshold before activating drag mode to avoid
    ///   triggering reorders on simple clicks.
    /// - Compares the pointer's horizontal coordinate against the center midpoint of each section tab
    ///   (using values captured by `section_tab_bounds`).
    /// - When mouse_x is left of tab `k`'s midpoint, the target insertion slot is `k` (before tab `k`).
    /// - If mouse_x is right of all tab midpoints, the target insertion slot is `N` (after the last tab).
    /// - Returns `true` if the drop slot changed, signalling GPUI to repaint the drop indicator.
    pub(crate) fn update_section_drag(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let Some(ref _source_id) = self.section_drag_source_id else {
            return false;
        };
        let Some((start_x, start_y)) = self.section_drag_start_pos else {
            return false;
        };

        // Enforce intentional drag threshold: user must move > 4px before drag mode engages
        if !self.is_dragging_section {
            let dx = (mouse_x - start_x).abs();
            let dy = (mouse_y - start_y).abs();
            if dx > 4.0 || dy > 4.0 {
                self.is_dragging_section = true;
            } else {
                return false;
            }
        }

        let section_ids: Vec<String> = if let Some(ref content) = self.edit_content {
            content.sections.iter().map(|s| s.id.clone()).collect()
        } else if let Some(ref content) = self.view_content {
            content.sections.iter().map(|s| s.id.clone()).collect()
        } else {
            Vec::new()
        };

        if section_ids.is_empty() {
            return false;
        }

        // Default insertion point is at the very end of the tab list
        let mut new_slot = section_ids.len();

        // Find the first tab whose midpoint the pointer is to the left of
        for (idx, sec_id) in section_ids.iter().enumerate() {
            if let Some(&(left_x, width)) = self.section_tab_bounds.get(sec_id) {
                let midpoint = left_x + width * 0.5;
                if mouse_x < midpoint {
                    new_slot = idx;
                    break;
                }
            }
        }

        // Only trigger repaint if the drop target slot actually changed
        if self.section_drop_slot != Some(new_slot) {
            self.section_drop_slot = Some(new_slot);
            return true;
        }

        false
    }

    /// Concludes an active section drag-and-drop gesture and applies the reorder if valid.
    ///
    /// Resets pointer drag tracking fields and calls `move_section` with the target slot.
    pub(crate) fn finish_section_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let was_dragging = self.is_dragging_section;
        let source_id = self.section_drag_source_id.take();
        let target_slot = self.section_drop_slot.take();
        self.section_drag_start_pos = None;
        self.is_dragging_section = false;

        if was_dragging {
            if let (Some(source_id), Some(slot)) = (source_id, target_slot) {
                let moved = self.move_section(&source_id, slot);
                if moved {
                    cx.notify();
                }
                return moved;
            }
        }
        false
    }
}

/// Helper to reorder a page inside a pages vector given a target insertion slot (0..=pages.len()).
/// Slot 0 inserts before index 0 (top).
/// Slot N inserts after index N-1 (bottom).
/// Returns true if the order was modified.
pub(crate) fn reorder_pages_vec(
    pages: &mut Vec<NotePage>,
    page_id: &str,
    target_slot: usize,
) -> bool {
    let Some(source_idx) = pages.iter().position(|p| p.id == page_id) else {
        return false;
    };
    let num_pages = pages.len();
    let target_slot = target_slot.min(num_pages);
    if target_slot == source_idx || target_slot == source_idx + 1 {
        return false;
    }
    let page = pages.remove(source_idx);
    let insert_idx = if target_slot > source_idx {
        target_slot - 1
    } else {
        target_slot
    };
    let insert_idx = insert_idx.min(pages.len());
    pages.insert(insert_idx, page);
    true
}

/// Helper to reorder a section inside a sections vector given a target insertion slot (0..=sections.len()).
///
/// Behavior & Index Math:
/// - Target slot `0` inserts before index 0 (leftmost position).
/// - Target slot `N` inserts after the last item (rightmost position).
/// - If `target_slot == source_idx` or `target_slot == source_idx + 1`, the section is dropped in place
///   and no modification occurs (returns `false`).
/// - When moving rightward (`target_slot > source_idx`), removing `source_idx` shifts subsequent elements left by 1,
///   so the target insertion index is adjusted to `target_slot - 1`.
/// - Returns `true` if the vector elements were reordered.
pub(crate) fn reorder_sections_vec(
    sections: &mut Vec<NoteSection>,
    section_id: &str,
    target_slot: usize,
) -> bool {
    let Some(source_idx) = sections.iter().position(|s| s.id == section_id) else {
        return false;
    };
    let num_sections = sections.len();
    let target_slot = target_slot.min(num_sections);
    if target_slot == source_idx || target_slot == source_idx + 1 {
        return false;
    }
    let section = sections.remove(source_idx);
    let insert_idx = if target_slot > source_idx {
        target_slot - 1
    } else {
        target_slot
    };
    let insert_idx = insert_idx.min(sections.len());
    sections.insert(insert_idx, section);
    true
}
