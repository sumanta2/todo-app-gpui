use gpui::Context;

use crate::app::NotesApp;
use crate::helpers::{get_selection_range, replace_range};
use crate::models::{load_canvas_items, load_note_content, ActiveField, CanvasItem, TextItem};

impl NotesApp {
    pub(crate) fn handle_key(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        // Shortcut: Ctrl + N (Create New Note)
        if control && key.eq_ignore_ascii_case("n") {
            self.create_note(cx);
            return;
        }

        // Shortcut: Escape (Discard changes or close menu)
        if key.eq_ignore_ascii_case("escape") {
            if self.is_editing {
                self.cancel_edit(cx);
            } else {
                self.viewer_text_anchor = None;
                self.viewer_active_text_block_id = None;
                self.is_selecting_viewer_text = false;
            }
            cx.notify();
            return;
        }

        if !self.is_editing {
            if control {
                if key.eq_ignore_ascii_case("c") {
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
                                                if let Some((start, end)) = get_selection_range(
                                                    self.viewer_text_cursor,
                                                    self.viewer_text_anchor,
                                                ) {
                                                    let selected_chars: String = t
                                                        .text
                                                        .chars()
                                                        .skip(start)
                                                        .take(end - start)
                                                        .collect();
                                                    cx.write_to_clipboard(
                                                        gpui::ClipboardItem::new_string(
                                                            selected_chars,
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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

        // Shortcut: Ctrl + Enter (Save changes)
        if control && key.eq_ignore_ascii_case("enter") {
            self.save_edit(cx);
            return;
        }

        // Shortcut: Tab toggles focus through Notebook, Section, Page, and Body
        if key.eq_ignore_ascii_case("tab") {
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
                            if let Some(CanvasItem::Text(t)) =
                                self.edit_canvas_items.iter().find(|i| match i {
                                    CanvasItem::Text(tx) => tx.id == id,
                                    _ => false,
                                })
                            {
                                self.edit_body = t.text.clone();
                                self.edit_body_cursor = self.edit_body.chars().count();
                                self.edit_body_anchor = None;
                            }
                        } else {
                            let new_id = chrono::Local::now().timestamp_millis().to_string();
                            let new_text_item = TextItem {
                                id: new_id.clone(),
                                x: 50.0,
                                y: 50.0,
                                text: String::new(),
                                width: Some(250.0),
                            };
                            self.edit_canvas_items.push(CanvasItem::Text(new_text_item));
                            self.active_text_block_id = Some(new_id);
                            self.edit_body = String::new();
                            self.edit_body_cursor = 0;
                            self.edit_body_anchor = None;
                        }
                    }
                }
                ActiveField::Body => {
                    self.sync_active_text_block();
                    self.active_field = ActiveField::NoteHeading;
                }
            }
            cx.notify();
            return;
        }

        // Shortcut: Ctrl + V image paste check (before borrowing self fields mutably)
        if control && key.eq_ignore_ascii_case("v") {
            let mut pasted_image = false;
            if let Some(clipboard) = cx.read_from_clipboard() {
                for entry in clipboard.entries() {
                    if let gpui::ClipboardEntry::Image(img) = entry {
                        self.attach_image_to_note(img, cx);
                        pasted_image = true;
                        break;
                    }
                }
            }
            if pasted_image {
                return;
            }
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
        };

        // Navigation
        if key.eq_ignore_ascii_case("left") {
            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
                if *cursor > 0 {
                    *cursor -= 1;
                }
            } else {
                if let Some((start, _)) = get_selection_range(*cursor, *anchor) {
                    *cursor = start;
                } else if *cursor > 0 {
                    *cursor -= 1;
                }
                *anchor = None;
            }
            cx.notify();
            return;
        } else if key.eq_ignore_ascii_case("right") {
            let len = text.chars().count();
            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
                if *cursor < len {
                    *cursor += 1;
                }
            } else {
                if let Some((_, end)) = get_selection_range(*cursor, *anchor) {
                    *cursor = end;
                } else if *cursor < len {
                    *cursor += 1;
                }
                *anchor = None;
            }
            cx.notify();
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
            } else if key.eq_ignore_ascii_case("c") {
                if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                    let selected_chars: String =
                        text.chars().skip(start).take(end - start).collect();
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected_chars));
                }
                return;
            } else if key.eq_ignore_ascii_case("x") {
                if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                    let selected_chars: String =
                        text.chars().skip(start).take(end - start).collect();
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected_chars));
                    replace_range(text, start, end, "");
                    *cursor = start;
                    *anchor = None;
                    self.sync_active_text_block();
                    cx.notify();
                }
                return;
            } else if key.eq_ignore_ascii_case("v") {
                if let Some(clipboard_text) = cx.read_from_clipboard().and_then(|item| item.text())
                {
                    let cleaned_text = if is_multiline {
                        clipboard_text
                    } else {
                        clipboard_text.replace("\n", " ")
                    };
                    if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                        replace_range(text, start, end, &cleaned_text);
                        *cursor = start + cleaned_text.chars().count();
                        *anchor = None;
                    } else {
                        let mut chars: Vec<char> = text.chars().collect();
                        for (idx, ch) in cleaned_text.chars().enumerate() {
                            chars.insert(*cursor + idx, ch);
                        }
                        *text = chars.into_iter().collect();
                        *cursor += cleaned_text.chars().count();
                    }
                    self.sync_active_text_block();
                    cx.notify();
                }
                return;
            }
        }

        // Enter key inserts newline in multiline fields
        if key.eq_ignore_ascii_case("enter") {
            if is_multiline {
                if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                    replace_range(text, start, end, "\n");
                    *cursor = start + 1;
                    *anchor = None;
                } else {
                    let mut chars: Vec<char> = text.chars().collect();
                    chars.insert(*cursor, '\n');
                    *text = chars.into_iter().collect();
                    *cursor += 1;
                }
                self.sync_active_text_block();
                cx.notify();
            }
            return;
        }

        // Backspace and Delete
        if key.eq_ignore_ascii_case("backspace") {
            if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                replace_range(text, start, end, "");
                *cursor = start;
                *anchor = None;
            } else if *cursor > 0 {
                let mut chars: Vec<char> = text.chars().collect();
                chars.remove(*cursor - 1);
                *text = chars.into_iter().collect();
                *cursor -= 1;
            }
            self.sync_active_text_block();
            cx.notify();
            return;
        } else if key.eq_ignore_ascii_case("delete") {
            if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                replace_range(text, start, end, "");
                *cursor = start;
                *anchor = None;
            } else {
                let len = text.chars().count();
                if *cursor < len {
                    let mut chars: Vec<char> = text.chars().collect();
                    chars.remove(*cursor);
                    *text = chars.into_iter().collect();
                }
            }
            self.sync_active_text_block();
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
                if let Some((start, end)) = get_selection_range(*cursor, *anchor) {
                    replace_range(text, start, end, &cleaned_char);
                    *cursor = start + cleaned_char.chars().count();
                    *anchor = None;
                } else {
                    let mut chars: Vec<char> = text.chars().collect();
                    for (idx, ch) in cleaned_char.chars().enumerate() {
                        chars.insert(*cursor + idx, ch);
                    }
                    *text = chars.into_iter().collect();
                    *cursor += cleaned_char.chars().count();
                }
                self.sync_active_text_block();
                cx.notify();
            }
        }
    }
}
