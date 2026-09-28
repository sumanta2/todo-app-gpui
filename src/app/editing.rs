//! Open a notebook for editing, keep the active text block in sync, then save or cancel.

use gpui::Context;

use crate::app::NotesApp;
use crate::models::{load_canvas_items, load_note_content, ActiveField, CanvasItem};

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
                            t.bold_spans = crate::text::styles::bool_vec_to_spans(&edit_body_bold);
                            t.italic_spans = crate::text::styles::bool_vec_to_spans(&edit_body_italic);
                            t.underline_spans = crate::text::styles::bool_vec_to_spans(&edit_body_underline);
                            t.strike_spans = crate::text::styles::bool_vec_to_spans(&edit_body_strike);

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
                                    *bold_spans = crate::text::styles::bool_vec_to_spans(&edit_body_bold);
                                    *italic_spans = crate::text::styles::bool_vec_to_spans(&edit_body_italic);
                                    *underline_spans =
                                        crate::text::styles::bool_vec_to_spans(&edit_body_underline);
                                    *strike_spans = crate::text::styles::bool_vec_to_spans(&edit_body_strike);
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
}
