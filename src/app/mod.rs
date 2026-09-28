pub(crate) mod actions;
pub(crate) mod key_handler;
pub(crate) mod storage;

use gpui::{App, FocusHandle, Focusable};

use crate::models::{ActiveField, CanvasItem, Note, NoteContent};

/// Root application state for the notes workspace.
///
/// This struct keeps all note data, the active editing state, and the canvas-level
/// interaction state in one place so the UI can read and update the current document
/// consistently from a single source of truth.
pub struct NotesApp {
    pub(crate) notes: Vec<Note>,
    pub(crate) selected_note_id: Option<String>,
    pub(crate) is_editing: bool,
    pub(crate) active_field: ActiveField,

    // Edit Heading State
    pub(crate) edit_heading: String,
    pub(crate) edit_heading_cursor: usize,
    pub(crate) edit_heading_anchor: Option<usize>,
    pub(crate) is_selecting_heading: bool,

    // Edit Body State
    pub(crate) edit_body: String,
    pub(crate) edit_body_cursor: usize,
    pub(crate) edit_body_anchor: Option<usize>,
    pub(crate) is_selecting_body: bool,
    pub(crate) edit_images: Vec<String>,
    pub(crate) edit_body_bold: Vec<bool>,

    // Canvas State
    pub(crate) edit_canvas_items: Vec<CanvasItem>,
    pub(crate) active_text_block_id: Option<String>,

    // Drag/Pan/Resize State
    pub(crate) drag_item_id: Option<String>,
    pub(crate) drag_start_mouse: Option<gpui::Point<gpui::Pixels>>,
    pub(crate) drag_start_item_pos: Option<(f32, f32)>,

    pub(crate) resize_item_id: Option<String>,
    pub(crate) resize_start_mouse: Option<gpui::Point<gpui::Pixels>>,
    pub(crate) resize_start_size: Option<(f32, f32)>,

    pub(crate) pan_x: f32,
    pub(crate) pan_y: f32,
    pub(crate) is_panning: bool,
    pub(crate) pan_has_dragged: bool,
    pub(crate) pan_start_mouse: Option<gpui::Point<gpui::Pixels>>,
    pub(crate) pan_start_val: Option<(f32, f32)>,

    // Measured canvas top Y offset in window coordinates (updated on first canvas interaction)
    pub(crate) canvas_top_y: f32,

    pub(crate) focus_handle: FocusHandle,
    pub(crate) active_section_id: Option<String>,
    pub(crate) active_page_id: Option<String>,
    pub(crate) edit_content: Option<NoteContent>,

    // Edit Notebook Name State
    pub(crate) edit_note_heading: String,
    pub(crate) edit_note_heading_cursor: usize,
    pub(crate) edit_note_heading_anchor: Option<usize>,
    pub(crate) is_selecting_note_heading: bool,

    // Edit Section Name State
    pub(crate) edit_section_name: String,
    pub(crate) edit_section_name_cursor: usize,
    pub(crate) edit_section_name_anchor: Option<usize>,
    pub(crate) is_selecting_section_name: bool,
    pub(crate) active_section_tab_x: f32,

    // Sidebar State
    pub(crate) is_sidebar_open: bool,

    // Viewer Text Selection State
    pub(crate) viewer_active_text_block_id: Option<String>,
    pub(crate) viewer_text_cursor: usize,
    pub(crate) viewer_text_anchor: Option<usize>,
    pub(crate) is_selecting_viewer_text: bool,

    // Window dimensions for canvas boundaries
    pub(crate) window_w: f32,

    // Cursor Blink State
    pub(crate) cursor_visible: bool,

    // Standardized Typography Configuration
    pub(crate) font_family: String,
    pub(crate) note_heading_font_size: f32,
    pub(crate) section_name_font_size: f32,
    pub(crate) page_heading_font_size: f32,
    pub(crate) canvas_body_font_size: f32,
    pub(crate) page_list_font_size: f32,
}

impl Focusable for NotesApp {
    /// Returns the app-wide keyboard focus handle used by GPUI widgets.
    ///
    /// This lets the sidebar, canvas, and text editors share the same focus scope so
    /// key events are routed to the currently active field.
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
