use gpui::Context;

use crate::app::NotesApp;

/// One of the character styles toggled from the Home ribbon or a keyboard shortcut.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextStyleKind {
    Bold,
    Italic,
    Underline,
    Strike,
}
use crate::models::{
    load_canvas_items, load_note_content, save_canvas_items, ActiveField, CanvasItem, Note,
    NotePage, NoteSection,
};

impl NotesApp {
    /// Called by GPUI when the app window is created during startup in main().
    ///
    /// This constructor initializes the saved notes, picks the first note as the default
    /// selection, and starts the cursor-blink timer used by the editor UI.
    /// This constructor is called from the app startup in main.rs:66:NotesApp::new using this syntax
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {

        // this below "load_note" function present at src\app\storage.rs filed
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
            edit_body_italic: Vec::new(),
            edit_body_underline: Vec::new(),
            edit_body_strike: Vec::new(),
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
            canvas_top_y: crate::constants::layout::INITIAL_CANVAS_TOP_Y,
            focus_handle: cx.focus_handle(),
            active_block_index: None,
            resize_block_index: None,
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
            active_section_tab_x: 0.0,
            is_sidebar_open: false,
            home_menu_open: false,
            viewer_active_text_block_id: None,
            viewer_text_cursor: 0,
            viewer_text_anchor: None,
            is_selecting_viewer_text: false,
            window_w: 800.0,
            cursor_visible: true,
            note_heading_font_size: crate::constants::typography::NOTE_HEADING_FONT_SIZE,
            section_name_font_size: crate::constants::typography::SECTION_NAME_FONT_SIZE,
            page_heading_font_size: crate::constants::typography::PAGE_HEADING_FONT_SIZE,
            canvas_body_font_size: crate::constants::typography::CANVAS_BODY_FONT_SIZE,
            page_list_font_size: crate::constants::typography::PAGE_LIST_FONT_SIZE,
            font_family: crate::constants::typography::DEFAULT_FONT_FAMILY.to_string(),
        };

        // Keep the Cursor blinking by toggling visibility every 500 ms and re-rendering.
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

    /// Toggles the collapsed/expanded state of the left sidebar and re-renders the UI.
    pub(crate) fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.is_sidebar_open = !self.is_sidebar_open;
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

    /// Begins the notebook editor workflow for the selected note.
    ///
    /// The function clones the saved note content into the editable structure, loads the
    /// active section/page state, and resets the input fields used by the editing UI.
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
                self.active_block_index = None;
                self.edit_body = String::new();
                self.reset_body_styles();
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

    /// Copies the current body editor state into the active text block on the canvas.
    ///
    /// This keeps the text content and the canvas item model aligned whenever the user types,
    /// edits the selection, or leaves a text block.
    pub(crate) fn sync_active_text_block(&mut self) {
        if self.active_field == ActiveField::Body {
            if let Some(ref active_id) = self.active_text_block_id {
                let pan_x = self.pan_x;
                let window_w = self.window_w;
                let is_sidebar_open = self.is_sidebar_open;
                let edit_body = self.edit_body.clone();
                let edit_body_bold = self.edit_body_bold.clone();
                let edit_body_italic = self.edit_body_italic.clone();
                let edit_body_underline = self.edit_body_underline.clone();
                let edit_body_strike = self.edit_body_strike.clone();
                let active_block_index = self.active_block_index;

                let max_line_len = edit_body
                    .lines()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0);
                let line_text_w = max_line_len as f32 * 7.0;
                let needed_width = line_text_w + 20.0;
                let sidebar_w = if is_sidebar_open { 220.0 } else { 44.0 };
                let page_sidebar_w = 180.0;
                let canvas_visible_w =
                    (window_w - sidebar_w - page_sidebar_w - 30.0).max(300.0);

                if let Some(item) = self
                    .edit_canvas_items
                    .iter_mut()
                    .find(|i| match i {
                        CanvasItem::Text(t) => t.id == *active_id,
                        CanvasItem::Mixed(m) => m.id == *active_id,
                        CanvasItem::Image(_) => false,
                    })
                {
                    match item {
                        CanvasItem::Text(t) => {
                            t.text = edit_body;
                            t.bold_spans = crate::helpers::bool_vec_to_spans(&edit_body_bold);
                            t.italic_spans = crate::helpers::bool_vec_to_spans(&edit_body_italic);
                            t.underline_spans = crate::helpers::bool_vec_to_spans(&edit_body_underline);
                            t.strike_spans = crate::helpers::bool_vec_to_spans(&edit_body_strike);

                            let base_w = t.width.unwrap_or(250.0);
                            let desired_w = needed_width.max(base_w).max(250.0);
                            let canvas_max_right = canvas_visible_w - pan_x;
                            let max_allowed_width = (canvas_max_right - t.x).max(150.0);
                            t.width = Some(desired_w.min(max_allowed_width));
                        }
                        CanvasItem::Mixed(m) => {
                            if let Some(idx) = active_block_index {
                                if let Some(crate::models::ContentBlock::Text {
                                    text,
                                    bold_spans,
                                    italic_spans,
                                    underline_spans,
                                    strike_spans,
                                }) = m.blocks.get_mut(idx)
                                {
                                    *text = edit_body;
                                    *bold_spans = crate::helpers::bool_vec_to_spans(&edit_body_bold);
                                    *italic_spans = crate::helpers::bool_vec_to_spans(&edit_body_italic);
                                    *underline_spans =
                                        crate::helpers::bool_vec_to_spans(&edit_body_underline);
                                    *strike_spans = crate::helpers::bool_vec_to_spans(&edit_body_strike);
                                }
                            }

                            let base_w = m.width.unwrap_or(250.0);
                            let desired_w = needed_width.max(base_w).max(250.0);
                            let canvas_max_right = canvas_visible_w - pan_x;
                            let max_allowed_width = (canvas_max_right - m.x).max(150.0);
                            m.width = Some(desired_w.min(max_allowed_width));
                        }
                        CanvasItem::Image(_) => {}
                    }
                }
            }
        }
    }

    /// Persists the current note edit session back to the selected note object.
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

    /// Exits edit mode without saving changes to the note payload.
    pub(crate) fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.is_editing = false;
        cx.notify();
    }

    /// Deletes a note from the in-memory list and saves the updated collection.
    pub(crate) fn delete_note(&mut self, id: String, cx: &mut Context<Self>) {
        self.notes.retain(|n| n.id != id);
        if self.selected_note_id == Some(id) {
            self.selected_note_id = None;
            self.is_editing = false;
        }
        self.save_notes();
        cx.notify();
    }

    /// Clears every per-character style flag for the text that is currently being edited.
    pub(crate) fn reset_body_styles(&mut self) {
        self.edit_body_bold.clear();
        self.edit_body_italic.clear();
        self.edit_body_underline.clear();
        self.edit_body_strike.clear();
    }

    /// Loads bold, italic, underline, and strikethrough spans into the active editor buffers.
    pub(crate) fn load_body_styles(&mut self, styles: &crate::models::TextStyleSpans, len: usize) {
        self.edit_body_bold = crate::helpers::spans_to_bool_vec(&styles.bold, len);
        self.edit_body_italic = crate::helpers::spans_to_bool_vec(&styles.italic, len);
        self.edit_body_underline = crate::helpers::spans_to_bool_vec(&styles.underline, len);
        self.edit_body_strike = crate::helpers::spans_to_bool_vec(&styles.strike, len);
    }

    fn body_style_flags_mut(&mut self, kind: TextStyleKind) -> &mut Vec<bool> {
        match kind {
            TextStyleKind::Bold => &mut self.edit_body_bold,
            TextStyleKind::Italic => &mut self.edit_body_italic,
            TextStyleKind::Underline => &mut self.edit_body_underline,
            TextStyleKind::Strike => &mut self.edit_body_strike,
        }
    }

    fn body_style_flags(&self, kind: TextStyleKind) -> &Vec<bool> {
        match kind {
            TextStyleKind::Bold => &self.edit_body_bold,
            TextStyleKind::Italic => &self.edit_body_italic,
            TextStyleKind::Underline => &self.edit_body_underline,
            TextStyleKind::Strike => &self.edit_body_strike,
        }
    }

    /// True when the current selection (or the character before the caret) uses this style.
    pub(crate) fn text_style_is_on(&self, kind: TextStyleKind) -> bool {
        if !self.is_editing || self.active_field != ActiveField::Body {
            return false;
        }
        let flags = self.body_style_flags(kind);
        let char_count = self.edit_body.chars().count();
        if let Some((start, end)) = crate::text_selection::get_selection_range(
            self.edit_body_cursor,
            self.edit_body_anchor,
        ) {
            let start = start.min(flags.len()).min(char_count);
            let end = end.min(flags.len()).min(char_count);
            if start < end {
                return flags[start..end].iter().all(|flag| *flag);
            }
        }
        let pos = self.edit_body_cursor;
        pos > 0 && pos - 1 < flags.len() && flags[pos - 1]
    }

    /// Toggles bold formatting on the current selection or, if no selection exists, on the
    /// active character.
    pub(crate) fn toggle_bold(&mut self, cx: &mut Context<Self>) {
        self.toggle_text_style(TextStyleKind::Bold, cx);
    }

    /// Toggles one character style on the selection, or on the character at the caret.
    pub(crate) fn toggle_text_style(&mut self, kind: TextStyleKind, cx: &mut Context<Self>) {
        if !self.is_editing || self.active_field != ActiveField::Body {
            return;
        }

        let char_count = self.edit_body.chars().count();
        let cursor = self.edit_body_cursor;
        let anchor = self.edit_body_anchor;
        {
            let flags = self.body_style_flags_mut(kind);
            if flags.len() < char_count {
                flags.resize(char_count, false);
            }

            if let Some((start, end)) = crate::text_selection::get_selection_range(cursor, anchor) {
                let start = start.min(char_count);
                let end = end.min(char_count);
                if start < end {
                    let all_on = flags[start..end].iter().all(|flag| *flag);
                    let new_value = !all_on;
                    for flag in &mut flags[start..end] {
                        *flag = new_value;
                    }
                }
            } else if cursor < char_count && cursor < flags.len() {
                flags[cursor] = !flags[cursor];
            } else if cursor > 0 && cursor - 1 < char_count && cursor - 1 < flags.len() {
                flags[cursor - 1] = !flags[cursor - 1];
            }
        }

        self.sync_active_text_block();
        cx.notify();
    }

    /// Updates the font size used for the notebook name in the sidebar.
    #[allow(dead_code)]
    pub(crate) fn set_note_heading_font_size(&mut self, size: f32) {
        self.note_heading_font_size = size;
    }

    /// Updates the font size used for section tabs.
    #[allow(dead_code)]
    pub(crate) fn set_section_name_font_size(&mut self, size: f32) {
        self.section_name_font_size = size;
    }

    /// Updates the font size used for page headings.
    #[allow(dead_code)]
    pub(crate) fn set_page_heading_font_size(&mut self, size: f32) {
        self.page_heading_font_size = size;
    }

    /// Updates the font size used for canvas text blocks.
    #[allow(dead_code)]
    pub(crate) fn set_canvas_body_font_size(&mut self, size: f32) {
        self.canvas_body_font_size = size;
    }

    /// Updates the font size used for items in the page sidebar list.
    #[allow(dead_code)]
    pub(crate) fn set_page_list_font_size(&mut self, size: f32) {
        self.page_list_font_size = size;
    }

    /// Updates the global font family and updates associated metrics.
    #[allow(dead_code)]
    pub(crate) fn set_font_family(&mut self, family: impl Into<String>) {
        self.font_family = family.into();
    }

    /// Returns the active FontType inferred from the currently configured font family.
    #[inline]
    pub(crate) fn font_type(&self) -> crate::constants::typography::FontType {
        crate::constants::typography::FontType::from_family_name(&self.font_family)
    }

    /// Computes the visual line height for canvas text blocks at current standardized font size.
    #[inline]
    pub(crate) fn canvas_line_height(&self) -> f32 {
        crate::constants::typography::line_height_for_font_size(self.canvas_body_font_size)
    }

    /// Computes the cursor indicator height for canvas text blocks at current standardized font size.
    #[inline]
    pub(crate) fn canvas_cursor_height(&self) -> f32 {
        crate::constants::typography::cursor_height_for_font_size(self.canvas_body_font_size)
    }

    /// Computes the selection highlight height for canvas text blocks at current standardized font size.
    #[inline]
    pub(crate) fn canvas_selection_height(&self) -> f32 {
        crate::constants::typography::selection_height_for_font_size(self.canvas_body_font_size)
    }
}

