//! Application state and the operations that change it.
//!
//! `NotesApp` below is the single source of truth. Each child module owns one job:
//! - `notebook` — create, select, and delete notebooks; sidebar visibility
//! - `outline` — sections and pages inside the open notebook
//! - `editing` — open, sync, save, and cancel an editing session
//! - `formatting` — character styles and font settings
//! - `keyboard` — key routing into the active field
//! - `storage` — notes.json and image files

pub(crate) mod editing;
pub(crate) mod formatting;
pub(crate) mod keyboard;
pub(crate) mod notebook;
pub(crate) mod outline;
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
    pub(crate) edit_body_italic: Vec<bool>,
    pub(crate) edit_body_underline: Vec<bool>,
    pub(crate) edit_body_strike: Vec<bool>,
    /// Bumped when body text, bold flags, or the canvas font change. The hit cache stores the
    /// stamp it was built with so pointer moves can reuse it.
    pub(crate) body_layout_stamp: u64,
    pub(crate) body_hit_cache: Option<crate::text::selection::TextHitCache>,
    /// Read-only viewer selection cache, keyed by the active segment id.
    pub(crate) viewer_hit_cache: Option<crate::text::selection::TextHitCache>,
    pub(crate) viewer_hit_cache_id: Option<String>,
    /// Highlight layers. Notified on their own while a multi-character drag is in progress.
    pub(crate) body_selection_overlay:
        std::cell::RefCell<Option<gpui::Entity<crate::canvas::selection_overlay::SelectionOverlay>>>,
    pub(crate) viewer_selection_overlay:
        std::cell::RefCell<Option<gpui::Entity<crate::canvas::selection_overlay::SelectionOverlay>>>,
    /// Latest pointer sample for an in-progress text drag. Applied once per frame.
    pub(crate) selection_drag_sample:
        Option<(crate::canvas::selection_overlay::HighlightKind, gpui::Point<gpui::Pixels>)>,
    pub(crate) selection_drag_queued: bool,
    /// Canvas position of the block the body or viewer drag started in.
    pub(crate) body_drag_origin: Option<(f32, f32)>,
    pub(crate) viewer_drag_origin: Option<(f32, f32)>,

    // Canvas State
    pub(crate) edit_canvas_items: Vec<CanvasItem>,
    pub(crate) active_text_block_id: Option<String>,
    /// Click point of a blinking caret that has not opened a text box yet.
    /// Edit mode only: the box is created when the user types.
    pub(crate) pending_caret: Option<(f32, f32)>,
    /// Text or mixed box currently under the pointer. Its header and border show while set.
    pub(crate) hovered_canvas_item_id: Option<String>,
    /// When the active block belongs to a `CanvasItem::Mixed`, this is the index of the
    /// `Text` entry inside `blocks` that `edit_body`/`edit_body_bold` mirror.
    pub(crate) active_block_index: Option<usize>,

    // Drag/Pan/Resize State
    pub(crate) drag_item_id: Option<String>,
    pub(crate) drag_start_mouse: Option<gpui::Point<gpui::Pixels>>,
    pub(crate) drag_start_item_pos: Option<(f32, f32)>,

    pub(crate) resize_item_id: Option<String>,
    /// When resizing an image inside a `CanvasItem::Mixed`, the index of that `Image` block
    /// inside `blocks`. `None` means the whole item (or its text width) is being resized.
    pub(crate) resize_block_index: Option<usize>,
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
    /// Whether the Home formatting menu is open.
    pub(crate) home_menu_open: bool,

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
