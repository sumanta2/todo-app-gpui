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
                                            if let crate::models::ContentBlock::Image { path, .. } = b {
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
        if let Some(ref mut content) = self.edit_content {
            let new_sec_id = chrono::Local::now().timestamp_millis().to_string();
            let new_page_id = format!("{}_page", new_sec_id);
            let new_section = NoteSection {
                id: new_sec_id.clone(),
                name: format!("Section {}", content.sections.len() + 1),
                pages: vec![NotePage {
                    id: new_page_id.clone(),
                    name: "Page 1".to_string(),
                    body: String::new(),
                    images: None,
                }],
            };
            content.sections.push(new_section);
            self.switch_to_page(new_sec_id, new_page_id, cx);
        }
    }

    /// Removes a section from the current editing content if more than one section remains.
    pub(crate) fn delete_section(&mut self, section_id: String, cx: &mut Context<Self>) {
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
        if let Some((sec_id, page_id)) = should_switch {
            self.switch_to_page(sec_id, page_id, cx);
        } else {
            cx.notify();
        }
    }

    /// Adds a new page to the active section and immediately switches to it.
    pub(crate) fn add_page(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut content) = self.edit_content {
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter_mut().find(|s| s.id == *sec_id) {
                    let new_page_id = chrono::Local::now().timestamp_millis().to_string();
                    let new_page = NotePage {
                        id: new_page_id.clone(),
                        name: format!("Page {}", section.pages.len() + 1),
                        body: String::new(),
                        images: None,
                    };
                    section.pages.push(new_page);
                    self.switch_to_page(sec_id.clone(), new_page_id, cx);
                }
            }
        }
    }

    /// Deletes a page from the active section while preserving the section when possible.
    pub(crate) fn delete_page(&mut self, page_id: String, cx: &mut Context<Self>) {
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
        if let Some((sec_id, page_id)) = should_switch {
            self.switch_to_page(sec_id, page_id, cx);
        } else {
            cx.notify();
        }
    }
}
