//! Regression tests for undo and redo. They build an editor in memory, so they do not
//! open a window or write `notes.json`.

use super::snapshot;
use super::EditKind;
use crate::app::NotesApp;
use crate::models::{
    ActiveField, CanvasItem, ImageItem, Note, NoteContent, NotePage, NoteSection, TextItem,
};

fn with_editor(check: impl FnOnce(&mut NotesApp) + 'static) {
    gpui_platform::application().run(move |cx| {
        let mut app = editor_in(cx);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(&mut app)));
        cx.quit();
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    });
}

fn editor_in(cx: &mut gpui::App) -> NotesApp {
    let mut app = NotesApp {
        notes: Vec::new(),
        selected_note_id: None,
        view_content: None,
        page_items_cache: None,
        image_cache: std::collections::HashMap::new(),
        defer_images: true,
        is_editing: true,
        note_dirty: false,
        history: super::History::default(),
        save_queued: false,
        active_field: ActiveField::Body,
        edit_heading: "Page".to_string(),
        edit_heading_cursor: 4,
        edit_heading_anchor: None,
        is_selecting_heading: false,
        edit_body: "Hello".to_string(),
        edit_body_cursor: 5,
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
        line_layout_anchor: "Hello".to_string(),
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
        edit_canvas_items: vec![CanvasItem::Text(TextItem {
            id: "box-1".to_string(),
            x: 10.0,
            y: 20.0,
            text: "Hello".to_string(),
            width: Some(250.0),
            bold_spans: Vec::new(),
            italic_spans: Vec::new(),
            underline_spans: Vec::new(),
            strike_spans: Vec::new(),
            font_runs: Vec::new(),
            line_layouts: Vec::new(),
        })],
        active_text_block_id: Some("box-1".to_string()),
        pending_caret: None,
        hovered_canvas_item_id: None,
        hovered_canvas_edge_id: None,
        active_block_index: None,
        drag_item_id: None,
        drag_start_mouse: None,
        drag_start_item_pos: None,
        resize_item_id: None,
        resize_block_index: None,
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
        active_section_id: Some("sec-1".to_string()),
        active_page_id: Some("page-1".to_string()),
        edit_content: Some(NoteContent {
            sections: vec![NoteSection {
                id: "sec-1".to_string(),
                name: "Section".to_string(),
                pages: vec![NotePage {
                    id: "page-1".to_string(),
                    name: "Page".to_string(),
                    body: String::new(),
                    images: None,
                }],
            }],
        }),
        edit_note_heading: "Notebook".to_string(),
        edit_note_heading_cursor: 8,
        edit_note_heading_anchor: None,
        is_selecting_note_heading: false,
        edit_section_name: "Section".to_string(),
        edit_section_name_cursor: 7,
        edit_section_name_anchor: None,
        is_selecting_section_name: false,
        active_section_tab_x: 0.0,
        is_sidebar_open: true,
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
        ribbon_pinned: false,
        ribbon_pane: crate::app::RibbonPane::Home,
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
    app.notes.push(Note {
        id: "note-1".to_string(),
        heading: "Notebook".to_string(),
        body: String::new(),
        created_at: String::new(),
        images: None,
        links: None,
        format: None,
    });
    app.selected_note_id = Some("note-1".to_string());
    app
}

fn apply_undo(app: &mut NotesApp) {
    let previous = app.swap_with_undo().expect("an undo step");
    app.history.restoring = true;
    snapshot::restore(app, previous);
    app.history.restoring = false;
}

fn apply_redo(app: &mut NotesApp) {
    let next = app.swap_with_redo().expect("a redo step");
    app.history.restoring = true;
    snapshot::restore(app, next);
    app.history.restoring = false;
}

fn box_pos(app: &NotesApp) -> (f32, f32) {
    match &app.edit_canvas_items[0] {
        CanvasItem::Text(text) => (text.x, text.y),
        _ => panic!("the first canvas item should stay a text box"),
    }
}

#[test]
fn typing_in_one_field_merges_and_other_commands_do_not() {
    let same = EditKind::typing(ActiveField::Body, Some("box-1".to_string()));
    assert!(same.merges_with(&EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string())
    )));
    assert!(EditKind::typing(ActiveField::Body, None).merges_with(&same));
    assert!(!same.merges_with(&EditKind::typing(ActiveField::Heading, Some("box-1".to_string()))));
    assert!(!same.merges_with(&EditKind::typing(
        ActiveField::Body,
        Some("box-2".to_string())
    )));
    assert!(!same.merges_with(&EditKind::Format));
    assert!(!EditKind::Format.merges_with(&EditKind::Format));
    assert!(!EditKind::CanvasGesture.merges_with(&EditKind::Canvas));
}

#[test]
fn undo_and_redo_roundtrip_canvas_text() {
    with_editor(|app| {
    assert!(!app.can_undo());
    assert!(!app.can_redo());

    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello!".to_string();
    app.edit_body_cursor = 6;

    assert!(app.can_undo());
    apply_undo(app);
    assert_eq!(app.edit_body, "Hello");
    assert!(app.can_redo());
    assert!(!app.can_undo());

    apply_redo(app);
    assert_eq!(app.edit_body, "Hello!");
    assert!(app.can_undo());
    assert!(!app.can_redo());
    });
}

#[test]
fn consecutive_typing_is_one_step_and_a_new_box_still_merges() {
    with_editor(|app| {
    app.record_edit(EditKind::typing(ActiveField::Body, None));
    app.edit_body = "Hello a".to_string();
    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello ab".to_string();

    assert_eq!(app.history.undo.len(), 1);
    apply_undo(app);
    assert_eq!(app.edit_body, "Hello");
    assert!(!app.can_undo());
    });
}

#[test]
fn typing_and_bold_are_separate_steps() {
    with_editor(|app| {
    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello!".to_string();

    app.record_edit(EditKind::Format);
    app.edit_body_bold = vec![true, false, false, false, false, false];

    assert_eq!(app.history.undo.len(), 2);
    apply_undo(app);
    assert_eq!(app.edit_body, "Hello!");
    assert!(app.edit_body_bold.iter().all(|flag| !flag));

    apply_undo(app);
    assert_eq!(app.edit_body, "Hello");
    });
}

#[test]
fn undo_restores_notebook_section_and_page_names() {
    with_editor(|app| {
    app.record_edit(EditKind::typing(ActiveField::NoteHeading, None));
    app.edit_note_heading = "Renamed notebook".to_string();
    app.active_field = ActiveField::SectionName;
    app.record_edit(EditKind::typing(ActiveField::SectionName, None));
    app.edit_section_name = "Renamed section".to_string();
    app.active_field = ActiveField::Heading;
    app.record_edit(EditKind::typing(ActiveField::Heading, None));
    app.edit_heading = "Renamed page".to_string();

    apply_undo(app);
    assert_eq!(app.edit_heading, "Page");
    assert_eq!(app.edit_section_name, "Renamed section");

    apply_undo(app);
    assert_eq!(app.edit_section_name, "Section");
    assert_eq!(app.edit_note_heading, "Renamed notebook");

    apply_undo(app);
    assert_eq!(app.edit_note_heading, "Notebook");
    assert_eq!(app.notes[0].heading, "Notebook");
    });
}

#[test]
fn a_new_edit_after_undo_drops_redo() {
    with_editor(|app| {
    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello!".to_string();
    apply_undo(app);
    assert!(app.can_redo());

    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello?".to_string();
    assert!(!app.can_redo());
    apply_undo(app);
    assert_eq!(app.edit_body, "Hello");
    });
}

#[test]
fn an_unmoved_gesture_is_dropped_and_keeps_redo() {
    with_editor(|app| {
    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello!".to_string();
    apply_undo(app);

    app.begin_canvas_gesture();
    app.discard_unchanged_edit();
    assert!(!app.can_undo());
    assert!(app.can_redo());
    apply_redo(app);
    assert_eq!(app.edit_body, "Hello!");
    });
}

#[test]
fn moving_a_text_box_and_inserting_an_image_can_be_undone() {
    with_editor(|app| {
    app.begin_canvas_gesture();
    if let CanvasItem::Text(text) = &mut app.edit_canvas_items[0] {
        text.x = 80.0;
        text.y = 90.0;
    }
    app.discard_unchanged_edit();
    assert!(app.can_undo());
    apply_undo(app);
    assert_eq!(box_pos(app), (10.0, 20.0));

    app.record_edit(EditKind::Image);
    app.edit_canvas_items.push(CanvasItem::Image(ImageItem {
        id: "img-1".to_string(),
        x: 30.0,
        y: 40.0,
        path: "images/pic.png".to_string(),
        width: 240.0,
        height: 180.0,
    }));
    apply_undo(app);
    assert_eq!(app.edit_canvas_items.len(), 1);
    });
}

#[test]
fn deleting_a_box_can_be_undone() {
    with_editor(|app| {
    app.record_edit(EditKind::Canvas);
    app.edit_canvas_items.clear();
    app.active_text_block_id = None;
    app.edit_body.clear();
    apply_undo(app);
    assert_eq!(app.edit_canvas_items.len(), 1);
    assert_eq!(app.edit_body, "Hello");
    });
}

#[test]
fn adding_a_page_can_be_undone() {
    with_editor(|app| {
    app.record_edit(EditKind::Structure);
    let content = app.edit_content.as_mut().expect("open notebook");
    content.sections[0].pages.push(NotePage {
        id: "page-2".to_string(),
        name: "Page 2".to_string(),
        body: String::new(),
        images: None,
    });
    apply_undo(app);
    let pages = &app.edit_content.as_ref().expect("notebook").sections[0].pages;
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].name, "Page");
    });
}

#[test]
fn history_stays_empty_outside_edit_mode_and_clears_with_the_session() {
    with_editor(|app| {
    app.is_editing = false;
    app.record_edit(EditKind::Format);
    app.edit_body = "changed".to_string();
    assert!(!app.can_undo());

    app.is_editing = true;
    app.record_edit(EditKind::Format);
    app.edit_body_bold = vec![true];
    assert!(app.can_undo());
    app.clear_history();
    assert!(!app.can_undo());
    assert!(!app.can_redo());
    });
}

#[test]
fn restoring_does_not_record_another_step() {
    with_editor(|app| {
    app.record_edit(EditKind::typing(
        ActiveField::Body,
        Some("box-1".to_string()),
    ));
    app.edit_body = "Hello!".to_string();
    app.history.restoring = true;
    app.record_edit(EditKind::Format);
    app.history.restoring = false;
    assert_eq!(app.history.undo.len(), 1);
    });
}

#[test]
fn the_stack_keeps_the_newest_one_hundred_steps() {
    with_editor(|app| {
    for step in 0..105 {
        app.record_edit(EditKind::Format);
        app.edit_heading = format!("Page {step}");
    }
    assert_eq!(app.history.undo.len(), 100);
    apply_undo(app);
    assert_eq!(app.edit_heading, "Page 103");
    });
}
