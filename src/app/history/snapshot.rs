//! One saved copy of the open notebook, plus the caret, so undo can put it back.
//!
//! `capture` flushes the active text box, then copies names, sections, pages, canvas items,
//! body styles, and the caret. `restore` writes those fields back and does not record a new
//! step. A new document field must be added in both functions or undo will drop it.
//! Equality uses the note id, notebook name, content, and canvas items. The caret is restored
//! but does not by itself make a new undo step.

use crate::app::NotesApp;
use crate::models::{
    save_canvas_items, ActiveField, CanvasItem, ContentBlock, LineLayout, NoteContent,
};

/// Document and caret state for a single undo step. Not written to `notes.json`.
#[derive(Clone)]
pub(super) struct EditSnapshot {
    note_id: String,
    note_heading: String,
    note_heading_cursor: usize,
    note_heading_anchor: Option<usize>,
    content: NoteContent,
    active_section_id: Option<String>,
    active_page_id: Option<String>,
    section_name: String,
    section_cursor: usize,
    section_anchor: Option<usize>,
    page_name: String,
    heading_cursor: usize,
    heading_anchor: Option<usize>,
    active_field: ActiveField,
    active_text_block_id: Option<String>,
    active_block_index: Option<usize>,
    pending_caret: Option<(f32, f32)>,
    edit_body: String,
    edit_body_cursor: usize,
    edit_body_anchor: Option<usize>,
    edit_body_bold: Vec<bool>,
    edit_body_italic: Vec<bool>,
    edit_body_underline: Vec<bool>,
    edit_body_strike: Vec<bool>,
    edit_body_font_family: Vec<u8>,
    edit_body_font_size: Vec<f32>,
    edit_body_font_color: Vec<u32>,
    edit_body_bg_color: Vec<u32>,
    edit_body_line_layouts: Vec<LineLayout>,
    canvas_items: Vec<CanvasItem>,
    edit_images: Vec<String>,
    selected_inline_image: Option<(String, usize)>,
    document_key: String,
}

impl EditSnapshot {
    pub(super) fn same_document(&self, other: &EditSnapshot) -> bool {
        self.document_key == other.document_key
    }
}

/// Flushes the active text box, then copies the open notebook and the caret.
pub(super) fn capture(app: &mut NotesApp) -> EditSnapshot {
    app.sync_active_text_block();

    let note_id = app.selected_note_id.clone().unwrap_or_default();
    let note_heading = app.edit_note_heading.clone();
    let mut content = app
        .edit_content
        .clone()
        .unwrap_or(NoteContent { sections: Vec::new() });
    let images = image_paths(&app.edit_canvas_items);
    patch_open_page(&mut content, app, &images);

    let document_key = document_key(&note_id, &note_heading, &content, &app.edit_canvas_items);

    EditSnapshot {
        note_id,
        note_heading_cursor: app.edit_note_heading_cursor,
        note_heading_anchor: app.edit_note_heading_anchor,
        note_heading,
        content,
        active_section_id: app.active_section_id.clone(),
        active_page_id: app.active_page_id.clone(),
        section_name: app.edit_section_name.clone(),
        section_cursor: app.edit_section_name_cursor,
        section_anchor: app.edit_section_name_anchor,
        page_name: app.edit_heading.clone(),
        heading_cursor: app.edit_heading_cursor,
        heading_anchor: app.edit_heading_anchor,
        active_field: app.active_field,
        active_text_block_id: app.active_text_block_id.clone(),
        active_block_index: app.active_block_index,
        pending_caret: app.pending_caret,
        edit_body: app.edit_body.clone(),
        edit_body_cursor: app.edit_body_cursor,
        edit_body_anchor: app.edit_body_anchor,
        edit_body_bold: app.edit_body_bold.clone(),
        edit_body_italic: app.edit_body_italic.clone(),
        edit_body_underline: app.edit_body_underline.clone(),
        edit_body_strike: app.edit_body_strike.clone(),
        edit_body_font_family: app.edit_body_font_family.clone(),
        edit_body_font_size: app.edit_body_font_size.clone(),
        edit_body_font_color: app.edit_body_font_color.clone(),
        edit_body_bg_color: app.edit_body_bg_color.clone(),
        edit_body_line_layouts: app.edit_body_line_layouts.clone(),
        canvas_items: app.edit_canvas_items.clone(),
        edit_images: images,
        selected_inline_image: app.selected_inline_image.clone(),
        document_key,
    }
}

/// Puts a snapshot back into the open editor. Does not record another history step.
pub(super) fn restore(app: &mut NotesApp, snap: EditSnapshot) {
    if let Some(note) = app.notes.iter_mut().find(|note| note.id == snap.note_id) {
        note.heading = snap.note_heading.clone();
    }

    app.edit_note_heading = snap.note_heading;
    app.edit_note_heading_cursor = clamp_cursor(&app.edit_note_heading, snap.note_heading_cursor);
    app.edit_note_heading_anchor = snap
        .note_heading_anchor
        .map(|cursor| clamp_cursor(&app.edit_note_heading, cursor));

    app.edit_content = Some(snap.content);
    app.active_section_id = snap.active_section_id;
    app.active_page_id = snap.active_page_id;

    app.edit_section_name = snap.section_name;
    app.edit_section_name_cursor = clamp_cursor(&app.edit_section_name, snap.section_cursor);
    app.edit_section_name_anchor = snap
        .section_anchor
        .map(|cursor| clamp_cursor(&app.edit_section_name, cursor));

    app.edit_heading = snap.page_name;
    app.edit_heading_cursor = clamp_cursor(&app.edit_heading, snap.heading_cursor);
    app.edit_heading_anchor = snap
        .heading_anchor
        .map(|cursor| clamp_cursor(&app.edit_heading, cursor));

    app.edit_canvas_items = snap.canvas_items;
    app.edit_images = snap.edit_images;
    app.active_field = snap.active_field;
    app.active_text_block_id = snap.active_text_block_id;
    app.active_block_index = snap.active_block_index;
    app.pending_caret = snap.pending_caret;

    app.edit_body = snap.edit_body;
    app.edit_body_cursor = clamp_cursor(&app.edit_body, snap.edit_body_cursor);
    app.edit_body_anchor = snap
        .edit_body_anchor
        .map(|cursor| clamp_cursor(&app.edit_body, cursor));
    app.edit_body_bold = snap.edit_body_bold;
    app.edit_body_italic = snap.edit_body_italic;
    app.edit_body_underline = snap.edit_body_underline;
    app.edit_body_strike = snap.edit_body_strike;
    app.edit_body_font_family = snap.edit_body_font_family;
    app.edit_body_font_size = snap.edit_body_font_size;
    app.edit_body_font_color = snap.edit_body_font_color;
    app.edit_body_bg_color = snap.edit_body_bg_color;
    app.edit_body_line_layouts = snap.edit_body_line_layouts;
    app.bind_line_layouts();

    app.selected_inline_image = snap.selected_inline_image;
    app.drag_item_id = None;
    app.drag_start_mouse = None;
    app.drag_start_item_pos = None;
    app.resize_item_id = None;
    app.resize_block_index = None;
    app.resize_start_mouse = None;
    app.resize_start_size = None;
    app.inline_image_gesture = None;
    app.is_selecting_body = false;
    app.is_selecting_heading = false;
    app.is_selecting_section_name = false;
    app.is_selecting_note_heading = false;
    app.page_items_cache = None;
    app.cursor_visible = true;
    app.touch_body_layout();
    if app.active_field == ActiveField::Body {
        app.ensure_body_hit_cache();
    }
}

fn patch_open_page(content: &mut NoteContent, app: &NotesApp, images: &[String]) {
    let Some(section_id) = &app.active_section_id else {
        return;
    };
    let Some(section) = content
        .sections
        .iter_mut()
        .find(|section| section.id == *section_id)
    else {
        return;
    };
    section.name = app.edit_section_name.clone();
    let Some(page_id) = &app.active_page_id else {
        return;
    };
    let Some(page) = section.pages.iter_mut().find(|page| page.id == *page_id) else {
        return;
    };
    page.name = app.edit_heading.clone();
    page.body = save_canvas_items(&app.edit_canvas_items);
    page.images = Some(images.to_vec());
}

fn image_paths(items: &[CanvasItem]) -> Vec<String> {
    let mut paths = Vec::new();
    for item in items {
        match item {
            CanvasItem::Image(image) => paths.push(image.path.clone()),
            CanvasItem::Mixed(mixed) => {
                for block in &mixed.blocks {
                    if let ContentBlock::Image { path, .. } = block {
                        paths.push(path.clone());
                    }
                }
            }
            CanvasItem::Text(_) => {}
        }
    }
    paths
}

fn document_key(
    note_id: &str,
    heading: &str,
    content: &NoteContent,
    canvas: &[CanvasItem],
) -> String {
    let content_json = serde_json::to_string(content).unwrap_or_default();
    let canvas_json = serde_json::to_string(canvas).unwrap_or_default();
    format!("{note_id}\u{1}{heading}\u{1}{content_json}\u{1}{canvas_json}")
}

fn clamp_cursor(text: &str, cursor: usize) -> usize {
    cursor.min(text.chars().count())
}
