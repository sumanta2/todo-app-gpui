//! Cut, copy, and paste for the active note field. The Home ribbon and the keyboard shortcuts share these.

use gpui::Context;

use crate::app::NotesApp;
use crate::helpers::replace_range;
use crate::models::{load_canvas_items, load_note_content, ActiveField, CanvasItem, ContentBlock};
use crate::text::selection::get_selection_range;

impl NotesApp {
    /// True when the active field, or the viewer selection, has a non-empty range.
    pub(crate) fn has_clipboard_selection(&self) -> bool {
        if self.is_editing {
            self.active_selection().is_some()
        } else {
            self.viewer_selected_text().is_some()
        }
    }

    /// Copies the current selection. In view mode this reads the open page; while editing it reads the focused field.
    pub(crate) fn copy_selection(&mut self, cx: &mut Context<Self>) {
        let selected = if self.is_editing {
            self.active_selection().map(|(_, _, text, _)| text)
        } else {
            self.viewer_selected_text()
        };
        if let Some(selected) = selected {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected));
        }
    }

    /// Deletes the selected image, the selected text, or the character after the caret.
    pub(crate) fn delete_forward(&mut self, cx: &mut Context<Self>) {
        if self.delete_selected_image() {
            self.schedule_autosave(cx);
            cx.notify();
            return;
        }
        if !self.is_editing {
            return;
        }
        let multiline = self.active_field == ActiveField::Body;
        if let Some((start, end, _, _)) = self.active_selection() {
            self.replace_active_range(start, end, "", multiline, 0, 0);
            self.place_active_cursor(start);
        } else {
            let cursor = self.active_cursor();
            let len = self.active_text().0.chars().count();
            if cursor >= len {
                return;
            }
            self.replace_active_range(cursor, cursor + 1, "", multiline, 0, 0);
            self.place_active_cursor(cursor);
        }
        self.after_clipboard_edit(cx);
    }

    /// Removes the selected inline or standalone image. Returns false when nothing is selected.
    fn delete_selected_image(&mut self) -> bool {
        let Some((item_id, block_idx)) = self.selected_inline_image.clone() else {
            return false;
        };
        let mut remove_item = false;
        let mut shift_active = false;
        let mut clear_editor = false;
        let found = self.edit_canvas_items.iter_mut().any(|item| match item {
            CanvasItem::Image(image) if image.id == item_id => {
                remove_item = true;
                true
            }
            CanvasItem::Mixed(mixed) if mixed.id == item_id => {
                let is_image = mixed
                    .blocks
                    .get(block_idx)
                    .is_some_and(|block| matches!(block, ContentBlock::Image { .. }));
                if is_image {
                    mixed.blocks.remove(block_idx);
                    shift_active = true;
                    if mixed.blocks.is_empty() {
                        remove_item = true;
                        clear_editor = true;
                    }
                }
                true
            }
            _ => false,
        });
        if !found {
            self.selected_inline_image = None;
            return false;
        }
        if shift_active && self.active_text_block_id.as_deref() == Some(item_id.as_str()) {
            if let Some(active) = self.active_block_index {
                if active > block_idx {
                    self.active_block_index = Some(active - 1);
                }
            }
        }
        if remove_item {
            self.edit_canvas_items.retain(|item| match item {
                CanvasItem::Image(image) => image.id != item_id,
                CanvasItem::Mixed(mixed) => mixed.id != item_id,
                CanvasItem::Text(_) => true,
            });
        }
        if clear_editor && self.active_text_block_id.as_deref() == Some(item_id.as_str()) {
            self.active_text_block_id = None;
            self.active_block_index = None;
            self.edit_body.clear();
            self.reset_body_styles();
            self.edit_body_cursor = 0;
        }
        self.selected_inline_image = None;
        self.inline_image_gesture = None;
        true
    }

    /// Copies the selection and removes it from the focused field.
    pub(crate) fn cut_selection(&mut self, cx: &mut Context<Self>) {
        if !self.is_editing {
            return;
        }
        let Some((start, end, selected, multiline)) = self.active_selection() else {
            return;
        };
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected));
        self.replace_active_range(start, end, "", multiline, 0, 0);
        self.after_clipboard_edit(cx);
    }

    /// Pastes an image from the clipboard, or inserts clipboard text at the caret.
    pub(crate) fn paste_clipboard(&mut self, cx: &mut Context<Self>) {
        if !self.is_editing {
            return;
        }
        if let Some(clipboard) = cx.read_from_clipboard() {
            for entry in clipboard.entries() {
                if let gpui::ClipboardEntry::Image(img) = entry {
                    self.attach_image_to_note(img, cx);
                    return;
                }
            }
        }
        let Some(clipboard_text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        if clipboard_text.is_empty() {
            return;
        }
        if self.pending_caret.is_some()
            && self.active_field == ActiveField::Body
            && self.active_text_block_id.is_none()
        {
            self.materialize_pending_caret();
        }

        let multiline = self.active_field == ActiveField::Body;
        let caret = self.edit_body_cursor;
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
        let cleaned = if multiline {
            clipboard_text
        } else {
            clipboard_text.replace('\n', " ")
        };
        let paste_len = cleaned.chars().count();
        if let Some((start, end, _, _)) = self.active_selection() {
            self.replace_active_range(start, end, &cleaned, multiline, font_color, bg_color);
            self.place_active_cursor(start + paste_len);
        } else {
            let insert_at = self.active_cursor();
            self.insert_active_text(insert_at, &cleaned, multiline, font_color, bg_color);
            self.place_active_cursor(insert_at + paste_len);
        }
        if multiline {
            let cursor = self.edit_body_cursor;
            crate::text::styles::advance_color_pin(
                self.font_color_pinned,
                &mut self.font_color_pin_at,
                cursor,
            );
            crate::text::styles::advance_color_pin(
                self.bg_color_pinned,
                &mut self.bg_color_pin_at,
                cursor,
            );
        }
        self.after_clipboard_edit(cx);
    }

    fn active_selection(&self) -> Option<(usize, usize, String, bool)> {
        let (text, cursor, anchor, multiline) = self.active_text();
        let (start, end) = get_selection_range(cursor, anchor)?;
        let selected: String = text
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect();
        if selected.is_empty() {
            None
        } else {
            Some((start, end, selected, multiline))
        }
    }

    fn active_text(&self) -> (&str, usize, Option<usize>, bool) {
        match self.active_field {
            ActiveField::NoteHeading => (
                self.edit_note_heading.as_str(),
                self.edit_note_heading_cursor,
                self.edit_note_heading_anchor,
                false,
            ),
            ActiveField::SectionName => (
                self.edit_section_name.as_str(),
                self.edit_section_name_cursor,
                self.edit_section_name_anchor,
                false,
            ),
            ActiveField::Heading => (
                self.edit_heading.as_str(),
                self.edit_heading_cursor,
                self.edit_heading_anchor,
                false,
            ),
            ActiveField::Body => (
                self.edit_body.as_str(),
                self.edit_body_cursor,
                self.edit_body_anchor,
                true,
            ),
        }
    }

    fn active_cursor(&self) -> usize {
        match self.active_field {
            ActiveField::NoteHeading => self.edit_note_heading_cursor,
            ActiveField::SectionName => self.edit_section_name_cursor,
            ActiveField::Heading => self.edit_heading_cursor,
            ActiveField::Body => self.edit_body_cursor,
        }
    }

    fn place_active_cursor(&mut self, cursor: usize) {
        match self.active_field {
            ActiveField::NoteHeading => {
                self.edit_note_heading_cursor = cursor;
                self.edit_note_heading_anchor = None;
            }
            ActiveField::SectionName => {
                self.edit_section_name_cursor = cursor;
                self.edit_section_name_anchor = None;
            }
            ActiveField::Heading => {
                self.edit_heading_cursor = cursor;
                self.edit_heading_anchor = None;
            }
            ActiveField::Body => {
                self.edit_body_cursor = cursor;
                self.edit_body_anchor = None;
            }
        }
    }

    fn replace_active_range(
        &mut self,
        start: usize,
        end: usize,
        replacement: &str,
        multiline: bool,
        font_color: u32,
        bg_color: u32,
    ) {
        let paste_len = replacement.chars().count();
        match self.active_field {
            ActiveField::NoteHeading => {
                replace_range(&mut self.edit_note_heading, start, end, replacement)
            }
            ActiveField::SectionName => {
                replace_range(&mut self.edit_section_name, start, end, replacement)
            }
            ActiveField::Heading => replace_range(&mut self.edit_heading, start, end, replacement),
            ActiveField::Body => {
                replace_range(&mut self.edit_body, start, end, replacement);
                if multiline {
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
                    if paste_len > 0 {
                        self.insert_body_paste_styles(start, paste_len, font_color, bg_color);
                    }
                }
            }
        }
    }

    fn insert_active_text(
        &mut self,
        insert_at: usize,
        text: &str,
        multiline: bool,
        font_color: u32,
        bg_color: u32,
    ) {
        let paste_len = text.chars().count();
        let target = match self.active_field {
            ActiveField::NoteHeading => &mut self.edit_note_heading,
            ActiveField::SectionName => &mut self.edit_section_name,
            ActiveField::Heading => &mut self.edit_heading,
            ActiveField::Body => &mut self.edit_body,
        };
        let mut chars: Vec<char> = target.chars().collect();
        for (idx, ch) in text.chars().enumerate() {
            chars.insert(insert_at + idx, ch);
        }
        *target = chars.into_iter().collect();
        if multiline && self.active_field == ActiveField::Body && paste_len > 0 {
            self.insert_body_paste_styles(insert_at, paste_len, font_color, bg_color);
        }
    }

    fn insert_body_paste_styles(
        &mut self,
        at: usize,
        count: usize,
        font_color: u32,
        bg_color: u32,
    ) {
        crate::text::styles::insert_style_set(
            [
                &mut self.edit_body_bold,
                &mut self.edit_body_italic,
                &mut self.edit_body_underline,
                &mut self.edit_body_strike,
            ],
            at,
            count,
            [false; 4],
        );
        crate::text::styles::insert_font_values(
            &mut self.edit_body_font_family,
            &mut self.edit_body_font_size,
            at,
            count,
            self.typing_font_family,
            self.typing_font_size_px,
        );
        crate::text::styles::insert_color_values(
            &mut self.edit_body_font_color,
            &mut self.edit_body_bg_color,
            at,
            count,
            font_color,
            bg_color,
        );
    }

    fn after_clipboard_edit(&mut self, cx: &mut Context<Self>) {
        if self.active_field == ActiveField::Body {
            self.touch_body_layout();
        }
        self.sync_active_text_block();
        self.schedule_autosave(cx);
        self.collapse_empty_active_text_box();
        cx.notify();
    }

    fn viewer_selected_text(&self) -> Option<String> {
        let active_id = self.viewer_active_text_block_id.as_ref()?;
        let (start, end) = get_selection_range(self.viewer_text_cursor, self.viewer_text_anchor)?;
        let note = self.selected_note()?;
        let content = load_note_content(&note.body, &note.images);
        let section = content
            .sections
            .iter()
            .find(|section| self.active_section_id.as_ref() == Some(&section.id))?;
        let page = section
            .pages
            .iter()
            .find(|page| self.active_page_id.as_ref() == Some(&page.id))?;
        let items = load_canvas_items(&page.body);
        let text = segment_text(&items, active_id)?;
        let selected: String = text
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect();
        if selected.is_empty() {
            None
        } else {
            Some(selected)
        }
    }
}

/// Text of a plain block, or of a mixed-box segment whose id is `{item}::{block}`.
fn segment_text(items: &[CanvasItem], id: &str) -> Option<String> {
    if let Some((mixed_id, index)) = id.rsplit_once("::") {
        if let Ok(index) = index.parse::<usize>() {
            for item in items {
                if let CanvasItem::Mixed(mixed) = item {
                    if mixed.id == mixed_id {
                        if let Some(ContentBlock::Text { text, .. }) = mixed.blocks.get(index) {
                            return Some(text.clone());
                        }
                    }
                }
            }
        }
    }
    for item in items {
        if let CanvasItem::Text(text) = item {
            if text.id == id {
                return Some(text.text.clone());
            }
        }
    }
    None
}
