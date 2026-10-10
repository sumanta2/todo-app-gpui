//! Open a notebook for editing, keep the active text block in sync, then save or cancel.

use gpui::Context;

use crate::app::NotesApp;
use crate::models::{
    load_canvas_items, load_note_content, ActiveField, CanvasItem, ContentBlock, TextItem,
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
        let layout = crate::app::metadata::AppMetadata::load();
        crate::constants::colors::install(layout.colors.clone());

        let mut app = Self {
            notes,
            selected_note_id,
            view_content: None,
            page_items_cache: None,
            image_cache: std::collections::HashMap::new(),
            defer_images: true,
            is_editing: false,
            note_dirty: false,
            history: crate::app::history::History::default(),
            save_queued: false,
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
            edit_body_font_family: Vec::new(),
            edit_body_font_size: Vec::new(),
            edit_body_font_color: Vec::new(),
            edit_body_bg_color: Vec::new(),
            edit_body_line_layouts: Vec::new(),
            line_layout_anchor: String::new(),
            body_layout_stamp: 0,
            body_hit_cache: None,
            viewer_hit_cache: None,
            viewer_hit_cache_id: None,
            body_selection_overlay: std::cell::RefCell::new(None),
            viewer_selection_overlay: std::cell::RefCell::new(None),
            selection_drag_sample: None,
            selection_drag_queued: false,
            body_drag_origin: None,
            viewer_drag_origin: None,
            edit_canvas_items: Vec::new(),
            active_text_block_id: None,
            pending_caret: None,
            hovered_canvas_item_id: None,
            hovered_canvas_edge_id: None,
            drag_item_id: None,
            drag_start_mouse: None,
            drag_start_item_pos: None,
            resize_item_id: None,
            resize_start_mouse: None,
            resize_start_size: None,
            selected_inline_image: None,
            inline_image_gesture: None,
            pan_x: 0.0,
            pan_y: 0.0,
            canvas_zoom: 1.0,
            is_panning: false,
            pan_has_dragged: false,
            pan_start_mouse: None,
            pan_start_val: None,
            canvas_top_y: crate::constants::layout::INITIAL_CANVAS_TOP_Y,
            note_heading_origin_x: 0.0,
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
            is_sidebar_open: layout.sidebar_pinned,
            title_search: String::new(),
            note_menu_open: false,
            section_menu_at: None,
            page_menu_at: None,
            section_renaming: false,
            page_renaming: false,
            page_sidebar_width: 180.0,
            page_sidebar_resizing: false,
            page_sidebar_resize_start_x: 0.0,
            page_sidebar_resize_start_w: 180.0,
            home_menu_open: false,
            view_menu_open: false,
            zoom_menu_open: false,
            full_page_view: false,
            font_style_menu_open: false,
            font_size_menu_open: false,
            typing_font_family: 0,
            typing_font_size_px: crate::constants::typography::CANVAS_BODY_FONT_SIZE,
            font_color_menu_open: false,
            bg_color_menu_open: false,
            ribbon_pinned: layout.ribbon_pinned,
            ribbon_pane: layout.ribbon_pane,
            typing_font_color: 0,
            typing_bg_color: 0,
            font_color_pinned: false,
            font_color_pin_at: None,
            bg_color_pinned: false,
            bg_color_pin_at: None,
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
                            // The caret is only painted while editing or while a viewer block is active.
                            // Idle view mode has nothing to blink, so skip the full-window rebuild.
                            let caret_shown =
                                this.is_editing || this.viewer_active_text_block_id.is_some();
                            if !caret_shown {
                                return;
                            }
                            this.cursor_visible = !this.cursor_visible;
                            this.refresh_body_selection(cx, false);
                            this.refresh_viewer_selection(cx, false);
                            // The floating caret is part of the main canvas, so it needs a full redraw.
                            if this.pending_caret.is_some()
                                && this.is_editing
                                && this.active_field == ActiveField::Body
                                && this.active_text_block_id.is_none()
                            {
                                cx.notify();
                            }
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
        if app.selected_note_id.is_some() {
            app.start_edit(cx);
        }
        app
    }

    /// Begins the notebook editor workflow for the selected note.
    ///
    /// The function clones the saved note content into the editable structure, loads the
    /// active section/page state, and resets the input fields used by the editing UI.
    pub(crate) fn start_edit(&mut self, cx: &mut Context<Self>) {
        let Some(selected_id) = self.selected_note_id.clone() else {
            return;
        };
        if self.notes.iter().any(|n| n.id == selected_id) {
            self.clear_history();
        }
        if let Some(note) = self.notes.iter().find(|n| n.id == selected_id) {
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
                self.pending_caret = None;
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

    /// Copies the current body editor state into the active text block on the canvas.
    ///
    /// This keeps the text content and the canvas item model aligned whenever the user types,
    /// edits the selection, or leaves a text block.
    pub(crate) fn sync_active_text_block(&mut self) {
        if self.active_field == ActiveField::Body {
            self.reconcile_line_layouts();
            if let Some(ref active_id) = self.active_text_block_id {
                let pan_x = self.pan_x;
                let window_w = self.window_w;
                let sidebar_w = self.layout_sidebar_w();
                let page_sidebar_w = self.layout_page_sidebar_w();
                let edit_body = self.edit_body.clone();
                let body_bold_spans = crate::text::styles::bool_vec_to_spans(&self.edit_body_bold);
                let body_italic_spans =
                    crate::text::styles::bool_vec_to_spans(&self.edit_body_italic);
                let body_underline_spans =
                    crate::text::styles::bool_vec_to_spans(&self.edit_body_underline);
                let body_strike_spans = crate::text::styles::bool_vec_to_spans(&self.edit_body_strike);
                let line_layouts =
                    crate::app::paragraph::saved_line_layouts(&self.edit_body_line_layouts);
                let font_runs = crate::text::styles::font_vecs_to_runs(
                    &self.edit_body_font_family,
                    &self.edit_body_font_size,
                    &self.edit_body_font_color,
                    &self.edit_body_bg_color,
                );
                let active_block_index = self.active_block_index;

                let line_text_w = self.body_text_advance();
                let needed_width = line_text_w + 24.0;
                let canvas_visible_w = (window_w - sidebar_w - page_sidebar_w - 30.0).max(300.0);

                if let Some(item) = self.edit_canvas_items.iter_mut().find(|i| match i {
                    CanvasItem::Text(t) => t.id == *active_id,
                    CanvasItem::Mixed(m) => m.id == *active_id,
                    CanvasItem::Image(_) => false,
                }) {
                    match item {
                        CanvasItem::Text(t) => {
                            t.text = edit_body;
                            t.bold_spans = body_bold_spans;
                            t.italic_spans = body_italic_spans;
                            t.underline_spans = body_underline_spans;
                            t.strike_spans = body_strike_spans;
                            t.font_runs = font_runs;
                            t.line_layouts = line_layouts;

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
                                    font_runs: block_font_runs,
                                    line_layouts: block_line_layouts,
                                }) = m.blocks.get_mut(idx)
                                {
                                    *text = edit_body;
                                    *bold_spans = body_bold_spans;
                                    *italic_spans = body_italic_spans;
                                    *underline_spans = body_underline_spans;
                                    *strike_spans = body_strike_spans;
                                    *block_font_runs = font_runs;
                                    *block_line_layouts = line_layouts;
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

    /// Writes the open edit session to `notes.json` without leaving edit mode.
    ///
    /// Scroll position, the caret, and the open page stay as they are. Nothing is written
    /// when the session has no unsaved changes.
    pub(crate) fn persist_edit(&mut self) {
        if !self.is_editing {
            return;
        }
        self.sync_current_page_state();
        if !self.note_dirty {
            return;
        }
        let Some(selected_id) = self.selected_note_id.clone() else {
            self.note_dirty = false;
            return;
        };
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == selected_id) {
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
        }
        self.note_dirty = false;
        self.save_notes();
    }

    /// Marks the edit session dirty and writes it after a short idle pause.
    pub(crate) fn schedule_autosave(&mut self, cx: &mut Context<Self>) {
        if !self.is_editing {
            return;
        }
        self.note_dirty = true;
        if self.save_queued {
            return;
        }
        self.save_queued = true;
        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(700))
                    .await;
                let _ = this.update(&mut cx, |this, _cx| {
                    this.save_queued = false;
                    this.persist_edit();
                });
            }
        })
        .detach();
    }

    /// Drops text boxes that have no visible characters so a later click can start clean.
    pub(crate) fn discard_empty_text_items(&mut self) {
        let active = self.active_text_block_id.clone();
        self.edit_canvas_items.retain(|item| match item {
            CanvasItem::Text(t) => !t.text.trim().is_empty(),
            CanvasItem::Mixed(m) => m.blocks.iter().any(|block| match block {
                ContentBlock::Image { .. } => true,
                ContentBlock::Text { text, .. } => !text.trim().is_empty(),
            }),
            CanvasItem::Image(_) => true,
        });
        if let Some(id) = active {
            let still_there = self.edit_canvas_items.iter().any(|item| match item {
                CanvasItem::Text(t) => t.id == id,
                CanvasItem::Mixed(m) => m.id == id,
                CanvasItem::Image(_) => false,
            });
            if !still_there {
                self.active_text_block_id = None;
                self.active_block_index = None;
                self.edit_body.clear();
                self.reset_body_styles();
                self.edit_body_cursor = 0;
                self.edit_body_anchor = None;
            }
        }
    }

    /// Parks a blinking caret at the click. No text box is created until the user types.
    pub(crate) fn arm_pending_caret(&mut self, click_x: f32, click_y: f32) {
        let x = click_x.max(TEXT_BOX_CHROME_X);
        let y = click_y.max(TEXT_BOX_TEXT_TOP + TEXT_CARET_NUDGE_Y);
        self.pending_caret = Some((x, y));
        self.active_text_block_id = None;
        self.active_block_index = None;
        self.edit_body.clear();
        self.reset_body_styles();
        self.edit_body_cursor = 0;
        self.edit_body_anchor = None;
        self.is_selecting_body = false;
        self.active_field = ActiveField::Body;
        self.cursor_visible = true;
    }

    /// Creates the text box for a caret that was waiting for the first typed character.
    pub(crate) fn materialize_pending_caret(&mut self) {
        let Some((caret_x, caret_y)) = self.pending_caret.take() else {
            return;
        };
        let x = (caret_x - TEXT_BOX_CHROME_X).max(0.0);
        let y = (caret_y - TEXT_BOX_TEXT_TOP - TEXT_CARET_NUDGE_Y).max(0.0);
        let new_id = chrono::Local::now().timestamp_millis().to_string();
        self.edit_canvas_items.push(CanvasItem::Text(TextItem {
            id: new_id.clone(),
            x,
            y,
            text: String::new(),
            width: Some(250.0),
            bold_spans: Vec::new(),
            italic_spans: Vec::new(),
            underline_spans: Vec::new(),
            strike_spans: Vec::new(),
            font_runs: Vec::new(),
            line_layouts: Vec::new(),
        }));
        self.active_text_block_id = Some(new_id);
        self.active_block_index = None;
        self.edit_body.clear();
        self.reset_body_styles();
        self.edit_body_cursor = 0;
        self.edit_body_anchor = None;
        self.active_field = ActiveField::Body;
        self.cursor_visible = true;
    }

    /// Moves into an existing text box and puts the caret at the end of the line under the click.
    pub(crate) fn focus_text_line_at_click(
        &mut self,
        id: &str,
        block_index: Option<usize>,
        click_y: f32,
        text_top: f32,
    ) {
        self.pending_caret = None;
        self.selected_inline_image = None;
        self.sync_active_text_block();
        self.active_text_block_id = Some(id.to_string());
        self.active_block_index = block_index;
        self.active_field = ActiveField::Body;
        if self.capture_segment(id, block_index).is_none() {
            return;
        }
        self.edit_body_cursor = end_of_line_at(
            &self.edit_body,
            text_top,
            click_y,
            self.canvas_line_height(),
        );
        self.edit_body_anchor = None;
        self.is_selecting_body = false;
        self.cursor_visible = true;
    }

    /// A text box whose rectangle contains the click, if one exists.
    ///
    /// Empty canvas beside a box is not a hit. The third value is the y of the first text line,
    /// used to choose which line the caret joins.
    pub(crate) fn text_target_at(
        &self,
        click_x: f32,
        click_y: f32,
    ) -> Option<(String, Option<usize>, f32)> {
        let line_h = self.canvas_line_height();

        for item in &self.edit_canvas_items {
            match item {
                CanvasItem::Text(t) => {
                    let lines = t.text.split('\n').count().max(1) as f32;
                    let text_top = t.y + TEXT_BOX_TEXT_TOP;
                    let bottom = text_top + lines * line_h + 6.0;
                    let width = t.width.unwrap_or(250.0);
                    if click_x >= t.x && click_x < t.x + width && click_y >= t.y && click_y < bottom
                    {
                        return Some((t.id.clone(), None, text_top));
                    }
                }
                CanvasItem::Mixed(m) => {
                    let width = m.width.unwrap_or(250.0);
                    if click_x < m.x || click_x >= m.x + width {
                        continue;
                    }
                    if let Some((idx, text_top, box_bottom)) = mixed_text_band(m, click_y, line_h) {
                        if click_y >= m.y && click_y < box_bottom {
                            return Some((m.id.clone(), Some(idx), text_top));
                        }
                    }
                }
                CanvasItem::Image(_) => {}
            }
        }

        None
    }

    /// Hides a plain text box once its last character is removed and leaves the caret behind.
    pub(crate) fn collapse_empty_active_text_box(&mut self) {
        if self.active_field != ActiveField::Body || !self.edit_body.trim().is_empty() {
            return;
        }
        let Some(id) = self.active_text_block_id.clone() else {
            return;
        };
        let pos = self.edit_canvas_items.iter().find_map(|item| match item {
            CanvasItem::Text(t) if t.id == id => Some((t.x, t.y)),
            _ => None,
        });
        let Some((x, y)) = pos else {
            return;
        };
        self.edit_canvas_items.retain(|item| match item {
            CanvasItem::Text(t) => t.id != id,
            _ => true,
        });
        self.active_text_block_id = None;
        self.active_block_index = None;
        self.edit_body_cursor = 0;
        self.edit_body_anchor = None;
        self.pending_caret = Some((
            x + TEXT_BOX_CHROME_X,
            y + TEXT_BOX_TEXT_TOP + TEXT_CARET_NUDGE_Y,
        ));
        self.cursor_visible = true;
    }
}

/// Gap from the box origin to the text, matching the header, border, and padding.
const TEXT_BOX_CHROME_X: f32 = 6.0;
const TEXT_BOX_TEXT_TOP: f32 = 18.0;

/// Screen pixels from the top of a mixed box to the first line of one text segment.
///
/// A mixed box stacks a 12px header, then images and text. The text under an image does not
/// start at the header, so hit testing has to skip every block above that segment.
pub(crate) fn mixed_text_line_screen_top(
    blocks: &[ContentBlock],
    block_index: usize,
    line_h: f32,
    zoom: f32,
    border: f32,
) -> f32 {
    let zoom = if zoom <= 0.05 { 1.0 } else { zoom };
    let mut screen = border + 12.0;
    for block in blocks.iter().take(block_index) {
        match block {
            ContentBlock::Image { height, .. } => screen += *height * zoom,
            ContentBlock::Text { text, .. } => {
                let lines = text.split('\n').count().max(1) as f32;
                screen += 10.0 + lines * line_h * zoom;
            }
        }
    }
    screen + 5.0
}
const TEXT_BOX_HEADER_H: f32 = 12.0;
const TEXT_CARET_NUDGE_Y: f32 = 2.0;

fn vertical_gap(click_y: f32, top: f32, bottom: f32) -> f32 {
    if click_y < top {
        top - click_y
    } else if click_y > bottom {
        click_y - bottom
    } else {
        0.0
    }
}

/// Character index at the end of the line whose height contains `click_y`.
fn end_of_line_at(text: &str, text_top: f32, click_y: f32, line_h: f32) -> usize {
    let line_count = text.split('\n').count().max(1);
    let rel = (click_y - text_top).max(0.0);
    let line_idx = if line_h <= 0.0 {
        0
    } else {
        ((rel / line_h).floor() as usize).min(line_count - 1)
    };
    let mut offset = 0usize;
    for (i, line) in text.split('\n').enumerate() {
        let len = line.chars().count();
        if i == line_idx {
            return offset + len;
        }
        offset += len + 1;
    }
    text.chars().count()
}

fn mixed_text_band(
    item: &crate::models::MixedItem,
    click_y: f32,
    line_h: f32,
) -> Option<(usize, f32, f32)> {
    let mut y = item.y + TEXT_BOX_HEADER_H + 1.0;
    let mut segments: Vec<(usize, f32, f32, f32)> = Vec::new();
    for (idx, block) in item.blocks.iter().enumerate() {
        match block {
            ContentBlock::Image { height, .. } => {
                y += *height;
            }
            ContentBlock::Text { text, .. } => {
                let band_top = y;
                let text_top = y + 5.0;
                let lines = text.split('\n').count().max(1) as f32;
                let band_bottom = text_top + lines * line_h + 5.0;
                segments.push((idx, band_top, text_top, band_bottom));
                y = band_bottom;
            }
        }
    }
    if segments.is_empty() {
        return None;
    }
    let box_bottom = y + 1.0;
    if click_y < item.y || click_y >= box_bottom {
        return None;
    }
    let chosen = segments
        .iter()
        .find(|(_, top, _, bottom)| click_y >= *top && click_y < *bottom)
        .or_else(|| {
            segments.iter().min_by(|a, b| {
                let da = vertical_gap(click_y, a.1, a.3);
                let db = vertical_gap(click_y, b.1, b.3);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
        })?;
    Some((chosen.0, chosen.2, box_bottom))
}

#[cfg(test)]
mod tests {
    use super::mixed_text_line_screen_top;
    use crate::models::ContentBlock;

    fn text_block(text: &str) -> ContentBlock {
        ContentBlock::Text {
            text: text.to_string(),
            bold_spans: Vec::new(),
            italic_spans: Vec::new(),
            underline_spans: Vec::new(),
            strike_spans: Vec::new(),
            font_runs: Vec::new(),
            line_layouts: Vec::new(),
        }
    }

    fn image_block(height: f32) -> ContentBlock {
        ContentBlock::Image {
            path: String::new(),
            width: 80.0,
            height,
            offset_x: 0.0,
        }
    }

    #[test]
    fn text_under_an_image_starts_below_that_image() {
        let blocks = vec![image_block(100.0), text_block("under")];
        let top = mixed_text_line_screen_top(&blocks, 1, 20.0, 1.0, 1.0);
        // Border 1 + header 12 + image 100 + segment padding 5.
        assert!((top - 118.0).abs() < 0.01);
    }

    #[test]
    fn first_text_segment_still_starts_under_the_header() {
        let blocks = vec![text_block("hello"), image_block(40.0)];
        let top = mixed_text_line_screen_top(&blocks, 0, 20.0, 1.0, 1.0);
        assert!((top - 18.0).abs() < 0.01);
    }
}
