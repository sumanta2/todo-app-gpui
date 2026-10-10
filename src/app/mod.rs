//! Application state and the operations that change it.
//!
//! `NotesApp` below is the single source of truth. Each child module owns one job:
//! - `notebook` — create, select, and delete notebooks; sidebar visibility
//! - `outline` — sections and pages inside the open notebook
//! - `editing` — open, sync, save, and cancel an editing session
//! - `formatting` — character styles and font settings
//! - `keyboard` — key routing into the active field
//! - `clipboard` — cut, copy, and paste for the focused note field
//! - `paragraph` — line indent and alignment inside a text box
//! - `storage` — notes.json and image files
//! - `metadata` — pinned left bar and ribbon, saved for the next launch
//! - `history` — undo and redo snapshots for the open edit session

pub(crate) mod clipboard;
pub(crate) mod editing;
pub(crate) mod history;
pub(crate) mod formatting;
pub(crate) mod keyboard;
pub(crate) mod metadata;
pub(crate) mod notebook;
pub(crate) mod outline;
pub(crate) mod paragraph;
pub(crate) mod storage;
pub(crate) mod zoom;

use gpui::{App, FocusHandle, Focusable};

use crate::models::{ActiveField, CanvasItem, Note, NoteContent};

/// Which frame handle is scaling an image inside a text box.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}

/// An in-progress move or zoom of an image that lives inside a text box.
#[derive(Clone)]
pub(crate) struct InlineImageGesture {
    pub(crate) item_id: String,
    pub(crate) block_index: usize,
    pub(crate) box_width: f32,
    pub(crate) kind: InlineGestureKind,
}

/// Move keeps the image on its row. Zoom scales from the dragged handle and keeps the aspect ratio.
#[derive(Clone)]
pub(crate) enum InlineGestureKind {
    Move {
        start_x: f32,
        start_offset: f32,
    },
    Zoom {
        handle: ImageHandle,
        start_x: f32,
        start_y: f32,
        start_w: f32,
        start_h: f32,
        start_offset: f32,
    },
}

/// Which ribbon menu fills the shared pin slot under the ribbon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RibbonPane {
    #[default]
    Home,
    View,
}

/// Root application state for the notes workspace.
///
/// This struct keeps all note data, the active editing state, and the canvas-level
/// interaction state in one place so the UI can read and update the current document
/// consistently from a single source of truth.
pub struct NotesApp {
    pub(crate) notes: Vec<Note>,
    pub(crate) selected_note_id: Option<String>,
    /// Parsed notebook for the open note in view mode, so each frame does not re-read JSON.
    pub(crate) view_content: Option<NoteContent>,
    /// Parsed canvas items for the page currently on screen, keyed by page id.
    pub(crate) page_items_cache: Option<(String, Vec<crate::models::CanvasItem>)>,
    /// Decrypted images, keyed by path. The first frame skips this and fills it afterward.
    pub(crate) image_cache: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>,
    /// While true, the opening frame draws text only. Images load on the following frame.
    pub(crate) defer_images: bool,
    pub(crate) is_editing: bool,
    /// Edit session has changes that are not in `notes.json` yet.
    pub(crate) note_dirty: bool,
    /// Undo and redo for this edit session. Not written to `notes.json`.
    pub(crate) history: history::History,
    /// A delayed auto-save is already waiting to write.
    pub(crate) save_queued: bool,
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
    /// Font family index for each body character. Missing entries use Calibri.
    pub(crate) edit_body_font_family: Vec<u8>,
    /// Pixel size for each body character. Missing entries use the default body size.
    pub(crate) edit_body_font_size: Vec<f32>,
    /// Text color for each body character. `0` means the default body color.
    pub(crate) edit_body_font_color: Vec<u32>,
    /// Highlight color for each body character. `0` means no background.
    pub(crate) edit_body_bg_color: Vec<u32>,
    /// Indent and alignment of each visual line in the body being edited.
    pub(crate) edit_body_line_layouts: Vec<crate::models::LineLayout>,
    /// Body text the line layouts were last reconciled against.
    pub(crate) line_layout_anchor: String,
    /// Bumped when body text, bold flags, or the canvas font change. The hit cache stores the
    /// stamp it was built with so pointer moves can reuse it.
    pub(crate) body_layout_stamp: u64,
    pub(crate) body_hit_cache: Option<crate::text::selection::TextHitCache>,
    /// Read-only viewer selection cache, keyed by the active segment id.
    pub(crate) viewer_hit_cache: Option<crate::text::selection::TextHitCache>,
    pub(crate) viewer_hit_cache_id: Option<String>,
    /// Highlight layers. Notified on their own while a multi-character drag is in progress.
    pub(crate) body_selection_overlay: std::cell::RefCell<
        Option<gpui::Entity<crate::canvas::selection_overlay::SelectionOverlay>>,
    >,
    pub(crate) viewer_selection_overlay: std::cell::RefCell<
        Option<gpui::Entity<crate::canvas::selection_overlay::SelectionOverlay>>,
    >,
    /// Latest pointer sample for an in-progress text drag. Applied once per frame.
    pub(crate) selection_drag_sample: Option<(
        crate::canvas::selection_overlay::HighlightKind,
        gpui::Point<gpui::Pixels>,
    )>,
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
    /// Right edge of a text box under the pointer. Kept apart from the box hover so leaving
    /// the edge does not hide the border while the pointer is still on the box.
    pub(crate) hovered_canvas_edge_id: Option<String>,
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

    /// Image inside a mixed text box that is showing its frame, `(item id, block index)`.
    pub(crate) selected_inline_image: Option<(String, usize)>,
    /// Drag that moves or zooms the selected inline image.
    pub(crate) inline_image_gesture: Option<InlineImageGesture>,

    pub(crate) pan_x: f32,
    pub(crate) pan_y: f32,
    /// Canvas magnification. `1.0` is 100%.
    pub(crate) canvas_zoom: f32,
    pub(crate) is_panning: bool,
    pub(crate) pan_has_dragged: bool,
    pub(crate) pan_start_mouse: Option<gpui::Point<gpui::Pixels>>,
    pub(crate) pan_start_val: Option<(f32, f32)>,

    // Measured canvas top Y offset in window coordinates (updated on first canvas interaction)
    pub(crate) canvas_top_y: f32,
    /// Window x of the sidebar notebook-name editor, measured from its layout.
    pub(crate) note_heading_origin_x: f32,

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
    /// When true, the notebook list stays pinned on the left. Saved in `metadata.json`.
    pub(crate) is_sidebar_open: bool,
    /// Text typed into the title-bar search box. Filters the notebook list.
    pub(crate) title_search: String,
    /// When true, keystrokes go to the title-bar search box.
    pub(crate) title_search_focused: bool,
    /// When the left bar is unpinned, this opens the note list before the section tabs.
    pub(crate) note_menu_open: bool,
    /// Width of the pages list on the right of the canvas. Dragging its left edge changes this.
    pub(crate) page_sidebar_width: f32,
    pub(crate) page_sidebar_resizing: bool,
    pub(crate) page_sidebar_resize_start_x: f32,
    pub(crate) page_sidebar_resize_start_w: f32,
    /// Whether the Home formatting menu is open.
    pub(crate) home_menu_open: bool,
    /// Whether the View ribbon menu is open.
    pub(crate) view_menu_open: bool,
    /// Whether the View ribbon zoom list is open.
    pub(crate) zoom_menu_open: bool,
    /// When set, only the canvas is shown and it fills the window.
    pub(crate) full_page_view: bool,
    /// Whether the Home ribbon font-style list is open.
    pub(crate) font_style_menu_open: bool,
    /// Whether the Home ribbon font-size list is open.
    pub(crate) font_size_menu_open: bool,
    /// Family index applied to characters typed after the ribbon selection.
    pub(crate) typing_font_family: u8,
    /// Pixel size applied to characters typed after the ribbon selection.
    pub(crate) typing_font_size_px: f32,
    /// Whether the font-color list is open.
    pub(crate) font_color_menu_open: bool,
    /// Whether the highlight-color list is open.
    pub(crate) bg_color_menu_open: bool,
    /// One shared pin. The menu in `ribbon_pane` stays fixed under the ribbon. Saved in `metadata.json`.
    pub(crate) ribbon_pinned: bool,
    /// Which ribbon menu occupies the shared pin slot.
    pub(crate) ribbon_pane: RibbonPane,
    /// Text color applied to characters typed after a ribbon color choice. `0` is the default.
    pub(crate) typing_font_color: u32,
    /// Highlight applied to characters typed after a ribbon color choice. `0` is none.
    pub(crate) typing_bg_color: u32,
    /// The font color stays in force until the caret moves away from this index.
    pub(crate) font_color_pinned: bool,
    pub(crate) font_color_pin_at: Option<usize>,
    /// The highlight stays in force until the caret moves away from this index.
    pub(crate) bg_color_pinned: bool,
    pub(crate) bg_color_pin_at: Option<usize>,

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
