use gpui::Context;

use crate::app::NotesApp;
use crate::models::{
    load_canvas_items, load_note_content, save_canvas_items, ActiveField, CanvasItem, Note,
    NotePage, NoteSection,
};

impl NotesApp {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let notes = Self::load_notes().unwrap_or_default();
        let selected_note_id = notes.first().map(|n| n.id.clone());

        let mut app = Self {
            notes,
            selected_note_id,
            is_editing: false,
            active_field: ActiveField::Heading,
            edit_heading: String::new(),
            edit_heading_cursor: 0,
            edit_heading_anchor: None,
            is_selecting_heading: false,
            edit_body: String::new(),
            edit_body_cursor: 0,
            edit_body_anchor: None,
            is_selecting_body: false,
            edit_images: Vec::new(),
            edit_body_bold: Vec::new(),
            edit_canvas_items: Vec::new(),
            active_text_block_id: None,
            drag_item_id: None,
            drag_start_mouse: None,
            drag_start_item_pos: None,
            resize_item_id: None,
            resize_start_mouse: None,
            resize_start_size: None,
            pan_x: 0.0,
            pan_y: 0.0,
            is_panning: false,
            pan_has_dragged: false,
            pan_start_mouse: None,
            pan_start_val: None,
            canvas_top_y: 78.0,
            focus_handle: cx.focus_handle(),
            active_section_id: None,
            active_page_id: None,
            edit_content: None,
            edit_note_heading: String::new(),
            edit_note_heading_cursor: 0,
            edit_note_heading_anchor: None,
            is_selecting_note_heading: false,
            edit_section_name: String::new(),
            edit_section_name_cursor: 0,
            edit_section_name_anchor: None,
            is_selecting_section_name: false,
            is_sidebar_open: false,
            viewer_active_text_block_id: None,
            viewer_text_cursor: 0,
            viewer_text_anchor: None,
            is_selecting_viewer_text: false,
            window_w: 800.0,
            cursor_visible: true,
        };

        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(500))
                        .await;
                    if this
                        .update(&mut cx, |this, cx| {
                            this.cursor_visible = !this.cursor_visible;
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }
        })
        .detach();

        app.initialize_active_section_page();
        app
    }

    pub(crate) fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.is_sidebar_open = !self.is_sidebar_open;
        cx.notify();
    }

    pub(crate) fn selected_note(&self) -> Option<&Note> {
        if let Some(ref id) = self.selected_note_id {
            self.notes.iter().find(|n| n.id == *id)
        } else {
            None
        }
    }

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
                            });

                            page.body = save_canvas_items(&self.edit_canvas_items);

                            let mut saved_images = Vec::new();
                            for item in &self.edit_canvas_items {
                                if let CanvasItem::Image(img) = item {
                                    saved_images.push(img.path.clone());
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
                        self.edit_body = String::new();
                        self.edit_body_bold = Vec::new();
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

    pub(crate) fn delete_page(&mut self, page_id: String, cx: &mut Context<Self>) {
        let mut should_switch = None;
        if let Some(ref mut content) = self.edit_content {
            if let Some(ref sec_id) = self.active_section_id {
                if let Some(section) = content.sections.iter_mut().find(|s| s.id == *sec_id) {
                    if section.pages.len() > 1 {
                        section.pages.retain(|p| p.id != page_id);
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

    pub(crate) fn start_edit(&mut self, cx: &mut Context<Self>) {
        if let Some(ref selected_id) = self.selected_note_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == *selected_id) {
                let content = load_note_content(&note.body, &note.images);
                self.edit_content = Some(content.clone());

                self.edit_note_heading = note.heading.clone();
                self.edit_note_heading_cursor = self.edit_note_heading.chars().count();
                self.edit_note_heading_anchor = None;
                self.is_selecting_note_heading = false;

                self.initialize_active_section_page();

                if let Some(ref sec_id) = self.active_section_id {
                    if let Some(section) = content.sections.iter().find(|s| s.id == *sec_id) {
                        self.edit_section_name = section.name.clone();
                        self.edit_section_name_cursor = self.edit_section_name.chars().count();
                        self.edit_section_name_anchor = None;
                        self.is_selecting_section_name = false;

                        if let Some(ref page_id) = self.active_page_id {
                            if let Some(page) = section.pages.iter().find(|p| p.id == *page_id) {
                                self.edit_canvas_items = load_canvas_items(&page.body);
                                self.edit_images = page.images.clone().unwrap_or_default();

                                self.edit_heading = page.name.clone();
                                self.edit_heading_cursor = self.edit_heading.chars().count();
                                self.edit_heading_anchor = None;
                            }
                        }
                    }
                }

                self.is_selecting_heading = false;
                self.active_text_block_id = None;
                self.edit_body = String::new();
                self.edit_body_bold = Vec::new();
                self.edit_body_cursor = 0;
                self.edit_body_anchor = None;
                self.is_selecting_body = false;

                self.pan_x = 0.0;
                self.pan_y = 0.0;
                self.is_panning = false;
                self.pan_has_dragged = false;
                self.pan_start_mouse = None;
                self.pan_start_val = None;
                self.drag_item_id = None;
                self.drag_start_mouse = None;
                self.drag_start_item_pos = None;
                self.resize_item_id = None;
                self.resize_start_mouse = None;
                self.resize_start_size = None;

                self.is_editing = true;
                self.active_field = ActiveField::Heading;
                cx.notify();
            }
        }
    }

    pub(crate) fn sync_active_text_block(&mut self) {
        if self.active_field == ActiveField::Body {
            if let Some(ref active_id) = self.active_text_block_id {
                let pan_x = self.pan_x;
                let window_w = self.window_w;
                let is_sidebar_open = self.is_sidebar_open;
                if let Some(item) = self.edit_canvas_items.iter_mut().find(|i| match i {
                    CanvasItem::Text(t) => t.id == *active_id,
                    _ => false,
                }) {
                    if let CanvasItem::Text(t) = item {
                        t.text = self.edit_body.clone();
                        t.bold_spans = crate::helpers::bool_vec_to_spans(&self.edit_body_bold);

                        let max_line_len = self
                            .edit_body
                            .lines()
                            .map(|line| line.chars().count())
                            .max()
                            .unwrap_or(0);

                        let line_text_w = max_line_len as f32 * 7.0;
                        let needed_width = line_text_w + 20.0;
                        let base_w = t.width.unwrap_or(250.0);
                        let desired_w = needed_width.max(base_w).max(250.0);

                        let sidebar_w = if is_sidebar_open { 220.0 } else { 44.0 };
                        let page_sidebar_w = 180.0;
                        let canvas_visible_w =
                            (window_w - sidebar_w - page_sidebar_w - 30.0).max(300.0);
                        let canvas_max_right = canvas_visible_w - pan_x;
                        let max_allowed_width = (canvas_max_right - t.x).max(150.0);

                        t.width = Some(desired_w.min(max_allowed_width));
                    }
                }
            }
        }
    }

    pub(crate) fn save_edit(&mut self, cx: &mut Context<Self>) {
        self.sync_current_page_state();

        if let Some(ref selected_id) = self.selected_note_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *selected_id) {
                note.heading = self.edit_note_heading.clone();

                if let Some(ref content) = self.edit_content {
                    note.body = serde_json::to_string(content).unwrap_or_default();

                    let mut all_images = Vec::new();
                    for section in &content.sections {
                        for page in &section.pages {
                            if let Some(ref imgs) = page.images {
                                for img in imgs {
                                    if !all_images.contains(img) {
                                        all_images.push(img.clone());
                                    }
                                }
                            }
                        }
                    }
                    note.images = Some(all_images);
                }

                self.is_editing = false;
                self.save_notes();
                cx.notify();
            }
        }
    }

    pub(crate) fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.is_editing = false;
        cx.notify();
    }

    pub(crate) fn delete_note(&mut self, id: String, cx: &mut Context<Self>) {
        self.notes.retain(|n| n.id != id);
        if self.selected_note_id == Some(id) {
            self.selected_note_id = None;
            self.is_editing = false;
        }
        self.save_notes();
        cx.notify();
    }

    pub(crate) fn toggle_bold(&mut self, cx: &mut Context<Self>) {
        if self.active_field != ActiveField::Body {
            return;
        }

        let char_count = self.edit_body.chars().count();
        if self.edit_body_bold.len() < char_count {
            self.edit_body_bold.resize(char_count, false);
        }

        if let Some((start, end)) = crate::text_selection::get_selection_range(
            self.edit_body_cursor,
            self.edit_body_anchor,
        ) {
            let start = start.min(char_count);
            let end = end.min(char_count);
            if start < end {
                let all_bold = self.edit_body_bold[start..end].iter().all(|&b| b);
                let new_bold = !all_bold;
                for i in start..end {
                    self.edit_body_bold[i] = new_bold;
                }
            }
        } else {
            let pos = self.edit_body_cursor;
            if pos < char_count {
                self.edit_body_bold[pos] = !self.edit_body_bold[pos];
            } else if pos > 0 && pos - 1 < char_count {
                self.edit_body_bold[pos - 1] = !self.edit_body_bold[pos - 1];
            }
        }

        self.sync_active_text_block();
        cx.notify();
    }
}
