//! Routes keyboard input to the field that currently has focus.

use gpui::Context;

use crate::app::NotesApp;
use crate::helpers::replace_range;
use crate::models::{load_canvas_items, load_note_content, ActiveField, CanvasItem, TextItem};
use crate::text::selection::{
    get_selection_range, move_cursor_down, move_cursor_up, move_cursor_word_left,
    move_cursor_word_right,
};

impl NotesApp {
    /// Handles the global key event route for the notes application.
    ///
    /// The function decides whether the event belongs to the notebook viewer, the note editor,
    /// or the currently focused text field and then delegates the actual behavior to the
    /// correct logic path.
    pub(crate) fn handle_key(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        // Shortcut: Ctrl + Plus / Minus / 0 zoom the canvas. Ctrl+scroll and pinch do the same.
        if control && !shift && (key == "=" || key == "+" || key.eq_ignore_ascii_case("plus")) {
            self.zoom_by(1.1, (0.0, 0.0), cx);
            return;
        }
        if control && !shift && (key == "-" || key == "_" || key.eq_ignore_ascii_case("minus")) {
            self.zoom_by(1.0 / 1.1, (0.0, 0.0), cx);
            return;
        }
        if control && !shift && key == "0" {
            self.reset_canvas_zoom(cx);
            return;
        }

        if self.is_editing && control && !shift && key.eq_ignore_ascii_case("z") {
            self.undo(cx);
            return;
        }
        if self.is_editing && control && !shift && key.eq_ignore_ascii_case("y") {
            self.redo(cx);
            return;
        }
        if self.is_editing && control && shift && key.eq_ignore_ascii_case("z") {
            self.redo(cx);
            return;
        }

        // Shortcut: Ctrl + N (Create New Note)
        if control && key.eq_ignore_ascii_case("n") {
            self.create_note(cx);
            return;
        }

        // Shortcut: Ctrl + B / I / U, and Ctrl + Shift + X (strikethrough)
        if control && !shift && key.eq_ignore_ascii_case("b") {
            self.toggle_text_style(crate::app::formatting::TextStyleKind::Bold, cx);
            return;
        }
        if control && !shift && key.eq_ignore_ascii_case("i") {
            self.toggle_text_style(crate::app::formatting::TextStyleKind::Italic, cx);
            return;
        }
        if control && !shift && key.eq_ignore_ascii_case("u") {
            self.toggle_text_style(crate::app::formatting::TextStyleKind::Underline, cx);
            return;
        }
        if control && shift && key.eq_ignore_ascii_case("x") {
            self.toggle_text_style(crate::app::formatting::TextStyleKind::Strike, cx);
            return;
        }

        if self.active_field == ActiveField::Search && !control {
            if key.eq_ignore_ascii_case("escape") {
                self.active_field = ActiveField::Heading;
                cx.notify();
                return;
            }
            if key == "backspace" {
                self.title_search.pop();
                cx.notify();
                return;
            }
            if key == "space" {
                self.title_search.push(' ');
                cx.notify();
                return;
            }
            if let Some(ch) = key.chars().next() {
                if key.chars().count() == 1 && !ch.is_control() {
                    let ch = if shift { ch.to_ascii_uppercase() } else { ch };
                    self.title_search.push(ch);
                    cx.notify();
                    return;
                }
            }
            return;
        }

        // Shortcut: Escape closes the Home menu or clears the current selection.
        if key.eq_ignore_ascii_case("escape") {
            if self.section_menu_at.is_some() || self.page_menu_at.is_some() {
                self.section_menu_at = None;
                self.page_menu_at = None;
                cx.notify();
                return;
            }
            if self.section_renaming || self.page_renaming {
                self.section_renaming = false;
                self.page_renaming = false;
                self.sync_current_page_state();
                cx.notify();
                return;
            }
            if self.home_menu_open
                || self.view_menu_open
                || self.note_menu_open
                || self.zoom_menu_open
                || self.font_style_menu_open
                || self.font_size_menu_open
                || self.font_color_menu_open
                || self.bg_color_menu_open
            {
                self.home_menu_open = false;
                self.view_menu_open = false;
                self.note_menu_open = false;
                self.zoom_menu_open = false;
                self.font_style_menu_open = false;
                self.font_size_menu_open = false;
                self.font_color_menu_open = false;
                self.bg_color_menu_open = false;
                cx.notify();
                return;
            }
            self.edit_body_anchor = None;
            self.edit_heading_anchor = None;
            self.edit_section_name_anchor = None;
            self.edit_note_heading_anchor = None;
            self.pending_caret = None;
            self.viewer_text_anchor = None;
            self.viewer_active_text_block_id = None;
            self.is_selecting_viewer_text = false;
            cx.notify();
            return;
        }

        if !self.is_editing {
            if control {
                if key.eq_ignore_ascii_case("c") {
                    self.copy_selection(cx);
                } else if key.eq_ignore_ascii_case("a") {
                    if let Some(ref active_id) = self.viewer_active_text_block_id {
                        if let Some(note) = self.selected_note() {
                            let content = load_note_content(&note.body, &note.images);
                            if let Some(ref sec_id) = self.active_section_id {
                                if let Some(section) =
                                    content.sections.iter().find(|s| s.id == *sec_id)
                                {
                                    if let Some(ref page_id) = self.active_page_id {
                                        if let Some(page) =
                                            section.pages.iter().find(|p| p.id == *page_id)
                                        {
                                            let items = load_canvas_items(&page.body);
                                            if let Some(CanvasItem::Text(t)) =
                                                items.iter().find(|item| match item {
                                                    CanvasItem::Text(tx) => tx.id == *active_id,
                                                    _ => false,
                                                })
                                            {
                                                self.viewer_text_anchor = Some(0);
                                                self.viewer_text_cursor = t.text.chars().count();
                                                cx.notify();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return;
        }

        // Shortcut: Tab toggles focus through Notebook, Section, Page, and Body
        if key.eq_ignore_ascii_case("tab") {
            self.pending_caret = None;
            match self.active_field {
                ActiveField::NoteHeading => {
                    self.active_field = ActiveField::SectionName;
                }
                ActiveField::SectionName => {
                    self.active_field = ActiveField::Heading;
                }
                ActiveField::Heading => {
                    self.active_field = ActiveField::Body;
                    if self.active_text_block_id.is_none() {
                        let existing_text_block = self.edit_canvas_items.iter().find_map(|item| {
                            if let CanvasItem::Text(t) = item {
                                Some(t.id.clone())
                            } else {
                                None
                            }
                        });
                        if let Some(id) = existing_text_block {
                            self.active_text_block_id = Some(id.clone());
                            self.active_block_index = None;
                            if let Some(CanvasItem::Text(t)) =
                                self.edit_canvas_items.iter().find(|i| match i {
                                    CanvasItem::Text(tx) => tx.id == id,
                                    _ => false,
                                })
                            {
                                self.edit_body = t.text.clone();
                                let len = self.edit_body.chars().count();
                                self.load_body_styles(
                                    &crate::models::TextStyleSpans {
                                        bold: t.bold_spans.clone(),
                                        italic: t.italic_spans.clone(),
                                        underline: t.underline_spans.clone(),
                                        strike: t.strike_spans.clone(),
                                        font_runs: t.font_runs.clone(),
                                        line_layouts: t.line_layouts.clone(),
                                    },
                                    len,
                                );
                                self.edit_body_cursor = self.edit_body.chars().count();
                                self.edit_body_anchor = None;
                                self.bind_line_layouts();
                            }
                        } else {
                            let new_id = chrono::Local::now().timestamp_millis().to_string();
                            let new_text_item = TextItem {
                                id: new_id.clone(),
                                x: 50.0,
                                y: 50.0,
                                text: String::new(),
                                width: Some(250.0),
                                bold_spans: Vec::new(),
                                italic_spans: Vec::new(),
                                underline_spans: Vec::new(),
                                strike_spans: Vec::new(),
                                font_runs: Vec::new(),
                                line_layouts: Vec::new(),
                            };
                            self.edit_canvas_items.push(CanvasItem::Text(new_text_item));
                            self.active_text_block_id = Some(new_id);
                            self.active_block_index = None;
                            self.edit_body = String::new();
                            self.reset_body_styles();
                            self.edit_body_cursor = 0;
                            self.edit_body_anchor = None;
                        }
                    }
                }
                ActiveField::Body => {
                    if self.active_field == ActiveField::Body {
                        self.touch_body_layout();
                    }
                    self.sync_active_text_block();
                    self.active_field = ActiveField::NoteHeading;
                }
                ActiveField::Search => {
                    self.active_field = ActiveField::NoteHeading;
                }
            }
            cx.notify();
            return;
        }

        if control && !shift {
            if key.eq_ignore_ascii_case("c") {
                self.copy_selection(cx);
                return;
            } else if key.eq_ignore_ascii_case("x") {
                self.cut_selection(cx);
                return;
            } else if key.eq_ignore_ascii_case("v") {
                self.paste_clipboard(cx);
                return;
            }
        }

        if records_text_edit(event, self.active_field) {
            self.record_edit(crate::app::history::EditKind::typing(
                self.active_field,
                self.active_text_block_id.clone(),
            ));
        }

        if self.pending_caret.is_some()
            && self.active_field == ActiveField::Body
            && self.active_text_block_id.is_none()
            && key_opens_pending_box(event, cx)
        {
            self.materialize_pending_caret();
        }

        // Get active field refs
        let (text, cursor, anchor, _is_selecting, is_multiline) = match self.active_field {
            ActiveField::NoteHeading => (
                &mut self.edit_note_heading,
                &mut self.edit_note_heading_cursor,
                &mut self.edit_note_heading_anchor,
                &mut self.is_selecting_note_heading,
                false,
            ),
            ActiveField::SectionName => (
                &mut self.edit_section_name,
                &mut self.edit_section_name_cursor,
                &mut self.edit_section_name_anchor,
                &mut self.is_selecting_section_name,
                false,
            ),
            ActiveField::Heading => (
                &mut self.edit_heading,
                &mut self.edit_heading_cursor,
                &mut self.edit_heading_anchor,
                &mut self.is_selecting_heading,
                false,
            ),
            ActiveField::Body => (
                &mut self.edit_body,
                &mut self.edit_body_cursor,
                &mut self.edit_body_anchor,
                &mut self.is_selecting_body,
                true,
            ),
            ActiveField::Search => return,
        };
        let typing_family = self.typing_font_family;
        let typing_size = self.typing_font_size_px;

        // Navigation
        if key.eq_ignore_ascii_case("left") {
            let next_pos = if control {
                move_cursor_word_left(text, *cursor)
            } else if !shift {
                if let Some((start, _)) = get_selection_range(*cursor, *anchor) {
                    start
                } else if *cursor > 0 {
                    *cursor - 1
                } else {
                    0
                }
            } else if *cursor > 0 {
                *cursor - 1
            } else {
                0
            };

            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
            } else {
                *anchor = None;
            }
            *cursor = next_pos;
            cx.notify();
            return;
        } else if key.eq_ignore_ascii_case("right") {
            let len = text.chars().count();
            let next_pos = if control {
                move_cursor_word_right(text, *cursor)
            } else if !shift {
                if let Some((_, end)) = get_selection_range(*cursor, *anchor) {
                    end
                } else if *cursor < len {
                    *cursor + 1
                } else {
                    len
                }
            } else if *cursor < len {
                *cursor + 1
            } else {
                len
            };

            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
            } else {
                *anchor = None;
            }
            *cursor = next_pos;
            cx.notify();
            return;
        } else if key.eq_ignore_ascii_case("up") {
            if is_multiline {
                let next_pos = move_cursor_up(text, *cursor);
                if shift {
                    if anchor.is_none() {
                        *anchor = Some(*cursor);
                    }
                } else {
                    *anchor = None;
                }
                *cursor = next_pos;
                cx.notify();
            }
            return;
        } else if key.eq_ignore_ascii_case("down") {
            if is_multiline {
                let next_pos = move_cursor_down(text, *cursor);
                if shift {
                    if anchor.is_none() {
                        *anchor = Some(*cursor);
                    }
                } else {
                    *anchor = None;
                }
                *cursor = next_pos;
                cx.notify();
            }
            return;
        } else if key.eq_ignore_ascii_case("home") {
            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
            } else {
                *anchor = None;
            }
            *cursor = 0;
            cx.notify();
            return;
        } else if key.eq_ignore_ascii_case("end") {
            let len = text.chars().count();
            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
            } else {
                *anchor = None;
            }
            *cursor = len;
            cx.notify();
            return;
        }

        // Editing Shortcuts (Select All, Copy, Cut, Paste)
        if control {
            if key.eq_ignore_ascii_case("a") {
                *anchor = Some(0);
                *cursor = text.chars().count();
                cx.notify();
                return;
            }
        }

        // Enter key inserts newline in multiline fields
        if key.eq_ignore_ascii_case("enter") {
            if is_multiline {
                let caret = *cursor;
                let font_color = crate::text::styles::color_for_insert(
                    &mut self.font_color_pinned,
                    &mut self.font_color_pin_at,
                    self.typing_font_color,
                    &self.edit_body_font_color,
                    caret,
                );
                let bg_color = crate::text::styles::color_for_insert(
                    &mut self.bg_color_pinned,
                    &mut self.bg_color_pin_at,
                    self.typing_bg_color,
                    &self.edit_body_bg_color,
                    caret,
                );
                if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                    replace_range(text, start, end, "\n");
                    if is_multiline {
                        crate::text::styles::drain_style_set(
                            [
                                &mut self.edit_body_bold,
                                &mut self.edit_body_italic,
                                &mut self.edit_body_underline,
                                &mut self.edit_body_strike,
                            ],
                            start,
                            end,
                        );
                        crate::text::styles::drain_font_values(
                            &mut self.edit_body_font_family,
                            &mut self.edit_body_font_size,
                            start,
                            end,
                        );
                        crate::text::styles::drain_color_values(
                            &mut self.edit_body_font_color,
                            &mut self.edit_body_bg_color,
                            start,
                            end,
                        );
                    }
                    crate::text::styles::insert_style_set(
                        [
                            &mut self.edit_body_bold,
                            &mut self.edit_body_italic,
                            &mut self.edit_body_underline,
                            &mut self.edit_body_strike,
                        ],
                        start,
                        1,
                        [false; 4],
                    );
                    crate::text::styles::insert_font_values(
                        &mut self.edit_body_font_family,
                        &mut self.edit_body_font_size,
                        start,
                        1,
                        typing_family,
                        typing_size,
                    );
                    crate::text::styles::insert_color_values(
                        &mut self.edit_body_font_color,
                        &mut self.edit_body_bg_color,
                        start,
                        1,
                        font_color,
                        bg_color,
                    );
                    *cursor = start + 1;
                    *anchor = None;
                    crate::text::styles::advance_color_pin(
                        self.font_color_pinned,
                        &mut self.font_color_pin_at,
                        *cursor,
                    );
                    crate::text::styles::advance_color_pin(
                        self.bg_color_pinned,
                        &mut self.bg_color_pin_at,
                        *cursor,
                    );
                } else {
                    let mut chars: Vec<char> = text.chars().collect();
                    let insert_at = *cursor;
                    chars.insert(*cursor, '\n');
                    *text = chars.into_iter().collect();
                    if is_multiline {
                        crate::text::styles::insert_style_set(
                            [
                                &mut self.edit_body_bold,
                                &mut self.edit_body_italic,
                                &mut self.edit_body_underline,
                                &mut self.edit_body_strike,
                            ],
                            insert_at,
                            1,
                            [false; 4],
                        );
                        crate::text::styles::insert_font_values(
                            &mut self.edit_body_font_family,
                            &mut self.edit_body_font_size,
                            insert_at,
                            1,
                            typing_family,
                            typing_size,
                        );
                        crate::text::styles::insert_color_values(
                            &mut self.edit_body_font_color,
                            &mut self.edit_body_bg_color,
                            insert_at,
                            1,
                            font_color,
                            bg_color,
                        );
                    }
                    *cursor += 1;
                    crate::text::styles::advance_color_pin(
                        self.font_color_pinned,
                        &mut self.font_color_pin_at,
                        *cursor,
                    );
                    crate::text::styles::advance_color_pin(
                        self.bg_color_pinned,
                        &mut self.bg_color_pin_at,
                        *cursor,
                    );
                }
                if self.active_field == ActiveField::Body {
                    self.touch_body_layout();
                }
                self.sync_active_text_block();
                self.schedule_autosave(cx);
                cx.notify();
            }
            return;
        }

        // Backspace and Delete
        if key.eq_ignore_ascii_case("backspace") {
            if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                replace_range(text, start, end, "");
                if is_multiline {
                    crate::text::styles::drain_style_set(
                        [
                            &mut self.edit_body_bold,
                            &mut self.edit_body_italic,
                            &mut self.edit_body_underline,
                            &mut self.edit_body_strike,
                        ],
                        start,
                        end,
                    );
                    crate::text::styles::drain_font_values(
                        &mut self.edit_body_font_family,
                        &mut self.edit_body_font_size,
                        start,
                        end,
                    );
                    crate::text::styles::drain_color_values(
                        &mut self.edit_body_font_color,
                        &mut self.edit_body_bg_color,
                        start,
                        end,
                    );
                }
                *cursor = start;
                *anchor = None;
            } else if *cursor > 0 {
                let mut chars: Vec<char> = text.chars().collect();
                chars.remove(*cursor - 1);
                if is_multiline {
                    crate::text::styles::remove_style_set(
                        [
                            &mut self.edit_body_bold,
                            &mut self.edit_body_italic,
                            &mut self.edit_body_underline,
                            &mut self.edit_body_strike,
                        ],
                        *cursor - 1,
                    );
                    crate::text::styles::remove_font_value(
                        &mut self.edit_body_font_family,
                        &mut self.edit_body_font_size,
                        *cursor - 1,
                    );
                    crate::text::styles::remove_color_value(
                        &mut self.edit_body_font_color,
                        &mut self.edit_body_bg_color,
                        *cursor - 1,
                    );
                }
                *text = chars.into_iter().collect();
                *cursor -= 1;
            }
            if self.active_field == ActiveField::Body {
                self.touch_body_layout();
            }
            self.sync_active_text_block();
            self.schedule_autosave(cx);
            self.collapse_empty_active_text_box();
            cx.notify();
            return;
        } else if key.eq_ignore_ascii_case("delete") {
            if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                replace_range(text, start, end, "");
                if is_multiline {
                    crate::text::styles::drain_style_set(
                        [
                            &mut self.edit_body_bold,
                            &mut self.edit_body_italic,
                            &mut self.edit_body_underline,
                            &mut self.edit_body_strike,
                        ],
                        start,
                        end,
                    );
                    crate::text::styles::drain_font_values(
                        &mut self.edit_body_font_family,
                        &mut self.edit_body_font_size,
                        start,
                        end,
                    );
                    crate::text::styles::drain_color_values(
                        &mut self.edit_body_font_color,
                        &mut self.edit_body_bg_color,
                        start,
                        end,
                    );
                }
                *cursor = start;
                *anchor = None;
            } else {
                let len = text.chars().count();
                if *cursor < len {
                    let mut chars: Vec<char> = text.chars().collect();
                    chars.remove(*cursor);
                    if is_multiline {
                        crate::text::styles::remove_style_set(
                            [
                                &mut self.edit_body_bold,
                                &mut self.edit_body_italic,
                                &mut self.edit_body_underline,
                                &mut self.edit_body_strike,
                            ],
                            *cursor,
                        );
                        crate::text::styles::remove_font_value(
                            &mut self.edit_body_font_family,
                            &mut self.edit_body_font_size,
                            *cursor,
                        );
                        crate::text::styles::remove_color_value(
                            &mut self.edit_body_font_color,
                            &mut self.edit_body_bg_color,
                            *cursor,
                        );
                    }
                    *text = chars.into_iter().collect();
                }
            }
            if self.active_field == ActiveField::Body {
                self.touch_body_layout();
            }
            self.sync_active_text_block();
            self.schedule_autosave(cx);
            self.collapse_empty_active_text_box();
            cx.notify();
            return;
        }

        // Character typing
        if !control && !event.keystroke.modifiers.platform {
            if let Some(character) = event.keystroke.key_char.as_deref() {
                let cleaned_char = if is_multiline {
                    character.to_owned()
                } else {
                    character.replace("\n", " ")
                };
                let char_len = cleaned_char.chars().count();
                let caret = *cursor;
                let font_color = if is_multiline {
                    crate::text::styles::color_for_insert(
                        &mut self.font_color_pinned,
                        &mut self.font_color_pin_at,
                        self.typing_font_color,
                        &self.edit_body_font_color,
                        caret,
                    )
                } else {
                    0
                };
                let bg_color = if is_multiline {
                    crate::text::styles::color_for_insert(
                        &mut self.bg_color_pinned,
                        &mut self.bg_color_pin_at,
                        self.typing_bg_color,
                        &self.edit_body_bg_color,
                        caret,
                    )
                } else {
                    0
                };
                let inherited = if is_multiline {
                    let prev = |flags: &[bool]| {
                        *cursor > 0 && *cursor - 1 < flags.len() && flags[*cursor - 1]
                    };
                    [
                        prev(&self.edit_body_bold),
                        prev(&self.edit_body_italic),
                        prev(&self.edit_body_underline),
                        prev(&self.edit_body_strike),
                    ]
                } else {
                    [false; 4]
                };

                if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                    replace_range(text, start, end, &cleaned_char);
                    if is_multiline {
                        crate::text::styles::drain_style_set(
                            [
                                &mut self.edit_body_bold,
                                &mut self.edit_body_italic,
                                &mut self.edit_body_underline,
                                &mut self.edit_body_strike,
                            ],
                            start,
                            end,
                        );
                        crate::text::styles::drain_font_values(
                            &mut self.edit_body_font_family,
                            &mut self.edit_body_font_size,
                            start,
                            end,
                        );
                        crate::text::styles::drain_color_values(
                            &mut self.edit_body_font_color,
                            &mut self.edit_body_bg_color,
                            start,
                            end,
                        );
                    }
                    crate::text::styles::insert_style_set(
                        [
                            &mut self.edit_body_bold,
                            &mut self.edit_body_italic,
                            &mut self.edit_body_underline,
                            &mut self.edit_body_strike,
                        ],
                        start,
                        char_len,
                        inherited,
                    );
                    if is_multiline {
                        crate::text::styles::insert_font_values(
                            &mut self.edit_body_font_family,
                            &mut self.edit_body_font_size,
                            start,
                            char_len,
                            typing_family,
                            typing_size,
                        );
                        crate::text::styles::insert_color_values(
                            &mut self.edit_body_font_color,
                            &mut self.edit_body_bg_color,
                            start,
                            char_len,
                            font_color,
                            bg_color,
                        );
                    }
                    *cursor = start + char_len;
                    if is_multiline {
                        crate::text::styles::advance_color_pin(
                            self.font_color_pinned,
                            &mut self.font_color_pin_at,
                            *cursor,
                        );
                        crate::text::styles::advance_color_pin(
                            self.bg_color_pinned,
                            &mut self.bg_color_pin_at,
                            *cursor,
                        );
                    }
                    *anchor = None;
                } else {
                    let mut chars: Vec<char> = text.chars().collect();
                    let insert_at = *cursor;
                    for (idx, ch) in cleaned_char.chars().enumerate() {
                        chars.insert(*cursor + idx, ch);
                    }
                    *text = chars.into_iter().collect();
                    if is_multiline {
                        crate::text::styles::insert_style_set(
                            [
                                &mut self.edit_body_bold,
                                &mut self.edit_body_italic,
                                &mut self.edit_body_underline,
                                &mut self.edit_body_strike,
                            ],
                            insert_at,
                            char_len,
                            inherited,
                        );
                        crate::text::styles::insert_font_values(
                            &mut self.edit_body_font_family,
                            &mut self.edit_body_font_size,
                            insert_at,
                            char_len,
                            typing_family,
                            typing_size,
                        );
                        crate::text::styles::insert_color_values(
                            &mut self.edit_body_font_color,
                            &mut self.edit_body_bg_color,
                            insert_at,
                            char_len,
                            font_color,
                            bg_color,
                        );
                    }
                    *cursor += char_len;
                    if is_multiline {
                        crate::text::styles::advance_color_pin(
                            self.font_color_pinned,
                            &mut self.font_color_pin_at,
                            *cursor,
                        );
                        crate::text::styles::advance_color_pin(
                            self.bg_color_pinned,
                            &mut self.bg_color_pin_at,
                            *cursor,
                        );
                    }
                }
                if self.active_field == ActiveField::Body {
                    self.touch_body_layout();
                }
                self.sync_active_text_block();
                self.schedule_autosave(cx);
                cx.notify();
            }
        }
    }
}

/// True when this key inserts or deletes characters in the focused field.
fn records_text_edit(event: &gpui::KeyDownEvent, field: crate::models::ActiveField) -> bool {
    let key = event.keystroke.key.as_str();
    let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
    if key.eq_ignore_ascii_case("backspace") || key.eq_ignore_ascii_case("delete") {
        return true;
    }
    if key.eq_ignore_ascii_case("enter") {
        return !control && field == crate::models::ActiveField::Body;
    }
    !control && event.keystroke.key_char.is_some()
}

/// True when this key should turn a blinking caret into a real text box.
fn key_opens_pending_box(event: &gpui::KeyDownEvent, cx: &mut Context<NotesApp>) -> bool {
    let key = event.keystroke.key.as_str();
    let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
    if control && key.eq_ignore_ascii_case("v") {
        return cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .is_some_and(|text| !text.is_empty());
    }
    if key.eq_ignore_ascii_case("enter") {
        return !control;
    }
    !control && !event.keystroke.modifiers.platform && event.keystroke.key_char.is_some()
}
