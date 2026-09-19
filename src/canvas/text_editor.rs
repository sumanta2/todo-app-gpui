use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClipboardItem, Context, FocusHandle, Focusable, KeyDownEvent,
};
use std::ops::Range;

#[derive(Clone)]
#[allow(dead_code)]
pub struct TextEditor {
    pub text: String,
    pub bold_flags: Vec<bool>,
    pub cursor: usize,
    pub anchor: Option<usize>,
    pub focus_handle: FocusHandle,
    pub is_selecting: bool,
    pub cursor_visible: bool,
}

#[allow(dead_code)]
impl TextEditor {
    /// Creates a new editor wrapper around a text value and binds it to a GPUI focus handle.
    pub fn new(text: String, cx: &mut Context<impl Focusable>) -> Self {
        let cursor = text.chars().count();
        Self {
            bold_flags: vec![false; cursor],
            text,
            cursor,
            anchor: None,
            focus_handle: cx.focus_handle(),
            is_selecting: false,
            cursor_visible: true,
        }
    }

    /// Returns the current selection as a character range when the anchor and cursor differ.
    pub fn selection(&self) -> Option<Range<usize>> {
        if let Some(anchor) = self.anchor {
            if anchor != self.cursor {
                let start = anchor.min(self.cursor);
                let end = anchor.max(self.cursor);
                return Some(start..end);
            }
        }
        None
    }

    /// Extracts the currently selected text from the editor buffer if a selection exists.
    pub fn selected_text(&self) -> Option<String> {
        let range = self.selection()?;
        let chars: Vec<char> = self.text.chars().collect();
        if range.end <= chars.len() {
            Some(chars[range.start..range.end].iter().collect())
        } else {
            None
        }
    }

    /// Selects the entire text content from beginning to end of the current buffer.
    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.chars().count();
    }

    /// Clears the current selection anchor without moving the cursor.
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Inserts a string at the active cursor position, replacing the current selection if one exists.
    pub fn insert_str(&mut self, s: &str) {
        let chars: Vec<char> = self.text.chars().collect();
        let total_chars = chars.len();

        let (start, end) = if let Some(range) = self.selection() {
            (range.start, range.end)
        } else {
            let pos = self.cursor.min(total_chars);
            (pos, pos)
        };

        let mut new_chars = Vec::with_capacity(total_chars - (end - start) + s.chars().count());
        new_chars.extend_from_slice(&chars[..start]);
        new_chars.extend(s.chars());
        new_chars.extend_from_slice(&chars[end..]);

        self.text = new_chars.into_iter().collect();
        self.cursor = start + s.chars().count();
        self.anchor = None;
    }

    /// Removes the selected content or the character immediately before the cursor.
    pub fn backspace(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        let total_chars = chars.len();

        if let Some(range) = self.selection() {
            let mut new_chars = Vec::with_capacity(total_chars - (range.end - range.start));
            new_chars.extend_from_slice(&chars[..range.start]);
            new_chars.extend_from_slice(&chars[range.end..]);
            self.text = new_chars.into_iter().collect();
            self.cursor = range.start;
            self.anchor = None;
        } else if self.cursor > 0 && self.cursor <= total_chars {
            let mut new_chars = Vec::with_capacity(total_chars - 1);
            new_chars.extend_from_slice(&chars[..self.cursor - 1]);
            new_chars.extend_from_slice(&chars[self.cursor..]);
            self.text = new_chars.into_iter().collect();
            self.cursor -= 1;
            self.anchor = None;
        }
    }

    /// Deletes the selected content or the character directly after the cursor.
    pub fn delete_forward(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        let total_chars = chars.len();

        if let Some(range) = self.selection() {
            let mut new_chars = Vec::with_capacity(total_chars - (range.end - range.start));
            new_chars.extend_from_slice(&chars[..range.start]);
            new_chars.extend_from_slice(&chars[range.end..]);
            self.text = new_chars.into_iter().collect();
            self.cursor = range.start;
            self.anchor = None;
        } else if self.cursor < total_chars {
            let mut new_chars = Vec::with_capacity(total_chars - 1);
            new_chars.extend_from_slice(&chars[..self.cursor]);
            new_chars.extend_from_slice(&chars[self.cursor + 1..]);
            self.text = new_chars.into_iter().collect();
            self.anchor = None;
        }
    }

    /// Moves the cursor one character to the left and optionally extends the active selection.
    pub fn move_left(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        } else {
            if let Some(range) = self.selection() {
                self.cursor = range.start;
            } else if self.cursor > 0 {
                self.cursor -= 1;
            }
            self.anchor = None;
        }
    }

    /// Moves the cursor one character to the right and optionally extends the active selection.
    pub fn move_right(&mut self, extend_selection: bool) {
        let total_chars = self.text.chars().count();
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
            if self.cursor < total_chars {
                self.cursor += 1;
            }
        } else {
            if let Some(range) = self.selection() {
                self.cursor = range.end;
            } else if self.cursor < total_chars {
                self.cursor += 1;
            }
            self.anchor = None;
        }
    }

    /// Moves the cursor to the previous visual line while preserving or extending the selection.
    pub fn move_up(&mut self, extend_selection: bool) {
        let next_pos = crate::text_selection::move_cursor_up(&self.text, self.cursor);
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = next_pos;
    }

    /// Moves the cursor to the next visual line while preserving or extending the selection.
    pub fn move_down(&mut self, extend_selection: bool) {
        let next_pos = crate::text_selection::move_cursor_down(&self.text, self.cursor);
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = next_pos;
    }

    /// Handles a key event, including clipboard shortcuts, cursor movement, and text insertion.
    pub fn handle_key<T: Focusable>(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<T>,
    ) -> bool {
        self.cursor_visible = true;
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        if control {
            if key.eq_ignore_ascii_case("a") {
                self.select_all();
                cx.notify();
                return true;
            } else if key.eq_ignore_ascii_case("c") {
                if let Some(selected) = self.selected_text() {
                    cx.write_to_clipboard(ClipboardItem::new_string(selected));
                }
                return true;
            } else if key.eq_ignore_ascii_case("x") {
                if let Some(selected) = self.selected_text() {
                    cx.write_to_clipboard(ClipboardItem::new_string(selected));
                    self.backspace();
                    cx.notify();
                }
                return true;
            } else if key.eq_ignore_ascii_case("v") {
                if let Some(item) = cx.read_from_clipboard() {
                    if let Some(text) = item.text() {
                        self.insert_str(&text);
                        cx.notify();
                    }
                }
                return true;
            }
        }

        match key {
            "left" => {
                self.move_left(shift);
                cx.notify();
                true
            }
            "right" => {
                self.move_right(shift);
                cx.notify();
                true
            }
            "up" => {
                self.move_up(shift);
                cx.notify();
                true
            }
            "down" => {
                self.move_down(shift);
                cx.notify();
                true
            }
            "backspace" => {
                self.backspace();
                cx.notify();
                true
            }
            "delete" => {
                self.delete_forward();
                cx.notify();
                true
            }
            "enter" => {
                self.insert_str("\n");
                cx.notify();
                true
            }
            _ => {
                if let Some(ref ch) = event.keystroke.key_char {
                    self.insert_str(ch.as_str());
                    cx.notify();
                    true
                } else if key.chars().count() == 1 && !control {
                    self.insert_str(key);
                    cx.notify();
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Renders the editor in a compact line-oriented form for a focused or unfocused state.
    pub fn render_editor(&self, is_focused: bool) -> AnyElement {
        self.render_editor_with_line_wrapper(is_focused, |_, _, _, row| row.into_any_element())
    }

    /// Renders the full text editor while allowing a caller to wrap each visual line.
    pub fn render_editor_with_line_wrapper<F>(
        &self,
        is_focused: bool,
        mut line_wrapper: F,
    ) -> AnyElement
    where
        F: FnMut(usize, usize, &str, gpui::Div) -> AnyElement,
    {
        let text = self.text.as_str();
        let cursor_idx = self.cursor;
        let selection_range = self.selection();

        if text.is_empty() {
            let row = div()
                .relative()
                .flex()
                .items_center()
                .min_h(px(20.0))
                .child(if is_focused {
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(2.0))
                        .w(px(2.0))
                        .h(px(16.0))
                        .bg(if self.cursor_visible {
                            rgb(0x0078d4)
                        } else {
                            rgba(0x00000000)
                        })
                } else {
                    div()
                })
                .child(div().text_color(rgb(0x606060)).child("Type note..."));
            return line_wrapper(0, 0, "", row).into_any_element();
        }

        let logical_lines: Vec<&str> = text.split('\n').collect();
        let mut line_rows: Vec<AnyElement> = Vec::new();
        let mut global_offset = 0;

        for (line_idx, line) in logical_lines.iter().enumerate() {
            let line_len = line.chars().count();
            let is_last_line = line_idx == logical_lines.len() - 1;
            let line_start = global_offset;
            let line_end = global_offset + line_len;

            let line_bold_flags = if line_start < self.bold_flags.len() {
                let end = line_end.min(self.bold_flags.len());
                let mut f = self.bold_flags[line_start..end].to_vec();
                if f.len() < line_len {
                    f.resize(line_len, false);
                }
                f
            } else {
                vec![false; line_len]
            };

            let mut row = div()
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .min_h(px(20.0));

            let has_sel_overlap = if let Some(ref sel) = selection_range {
                let sel_overlap_start = sel.start.max(line_start);
                let sel_overlap_end = sel.end.min(line_end);
                sel_overlap_start < sel_overlap_end || (line_len == 0 && sel.start <= line_start && sel.end > line_start)
            } else {
                false
            };

            if has_sel_overlap {
                if let Some(ref sel) = selection_range {
                    let sel_overlap_start = sel.start.max(line_start);
                    let sel_overlap_end = sel.end.min(line_end);

                    if sel_overlap_start < sel_overlap_end {
                        let line_chars: Vec<char> = line.chars().collect();
                        let loc_start = sel_overlap_start.saturating_sub(line_start).min(line_chars.len());
                        let loc_end = sel_overlap_end.saturating_sub(line_start).min(line_chars.len());

                        let before_prefix: String = line_chars[..loc_start].iter().collect();
                        let sel_content: String = line_chars[loc_start..loc_end].iter().collect();

                        let before_flags = &line_bold_flags[..loc_start.min(line_bold_flags.len())];
                        let before_runs = crate::helpers::split_text_into_styled_runs(&before_prefix, before_flags);
                        let mut before_ghosts = Vec::new();
                        for run in &before_runs {
                            let disp = run.text.replace(' ', "\u{00A0}");
                            let el = div()
                                .text_color(rgba(0x00000000))
                                .font_weight(if run.is_bold {
                                    gpui::FontWeight::BOLD
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .child(disp);
                            before_ghosts.push(el.into_any_element());
                        }

                        let sel_flags = &line_bold_flags[loc_start.min(line_bold_flags.len())..loc_end.min(line_bold_flags.len())];
                        let sel_runs = crate::helpers::split_text_into_styled_runs(&sel_content, sel_flags);
                        let mut sel_ghosts = Vec::new();
                        for run in &sel_runs {
                            let disp = run.text.replace(' ', "\u{00A0}");
                            let el = div()
                                .text_color(rgba(0x00000000))
                                .font_weight(if run.is_bold {
                                    gpui::FontWeight::BOLD
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .child(disp);
                            sel_ghosts.push(el.into_any_element());
                        }

                        row = row.child(
                            div()
                                .absolute()
                                .left(px(0.0))
                                .top(px(1.0))
                                .flex()
                                .flex_row()
                                .items_center()
                                .children(before_ghosts)
                                .child(
                                    div()
                                        .bg(rgb(0x0078d4))
                                        .rounded(px(2.0))
                                        .h(px(18.0))
                                        .flex()
                                        .items_center()
                                        .children(sel_ghosts),
                                ),
                        );
                    } else if line_len == 0 && sel.start <= line_start && sel.end > line_start {
                        row = row.child(
                            div()
                                .absolute()
                                .left(px(0.0))
                                .top(px(1.0))
                                .w(px(6.0))
                                .h(px(18.0))
                                .bg(rgb(0x0078d4))
                                .rounded(px(2.0)),
                        );
                    }
                }
            }

            // Render line text as styled runs with bold support
            let runs = crate::helpers::split_text_into_styled_runs(line, &line_bold_flags);
            let mut line_elements = Vec::new();
            if runs.is_empty() {
                line_elements.push(
                    div()
                        .text_color(rgb(0xd4d4d4))
                        .child("\u{00A0}")
                        .into_any_element(),
                );
            } else {
                for run in &runs {
                    let disp = run.text.replace(' ', "\u{00A0}");
                    let el = div()
                        .text_color(if run.is_bold {
                            rgb(0xffffff)
                        } else {
                            rgb(0xd4d4d4)
                        })
                        .font_weight(if run.is_bold {
                            gpui::FontWeight::BOLD
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(disp);
                    line_elements.push(el.into_any_element());
                }
            }

            row = row.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .children(line_elements),
            );

            // If focused and cursor is in this line, render cursor overlay
            let is_cursor_in_line = if is_last_line {
                cursor_idx >= line_start && cursor_idx <= line_end
            } else {
                cursor_idx >= line_start && cursor_idx <= line_end
            };

            if is_focused && is_cursor_in_line && selection_range.is_none() {
                let local_cursor = cursor_idx.saturating_sub(line_start).min(line_len);
                let line_chars: Vec<char> = line.chars().collect();
                let before_prefix: String = line_chars[..local_cursor].iter().collect();
                let before_flags = &line_bold_flags[..local_cursor.min(line_bold_flags.len())];
                let prefix_runs = crate::helpers::split_text_into_styled_runs(&before_prefix, before_flags);

                let mut ghost_elements = Vec::new();
                for run in &prefix_runs {
                    let disp = run.text.replace(' ', "\u{00A0}");
                    let el = div()
                        .text_color(rgba(0x00000000))
                        .font_weight(if run.is_bold {
                            gpui::FontWeight::BOLD
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(disp);
                    ghost_elements.push(el.into_any_element());
                }

                row = row.child(
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(2.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .children(ghost_elements)
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(16.0))
                                .bg(if self.cursor_visible {
                                    rgb(0x0078d4)
                                } else {
                                    rgba(0x00000000)
                                })
                                .flex_shrink_0()
                                .ml(px(-1.0)),
                        ),
                );
            }

            let row = line_wrapper(line_idx, line_start, line, row);
            line_rows.push(row);
            global_offset += line_len + 1; // +1 for \n
        }

        div()
            .flex()
            .flex_col()
            .children(line_rows)
            .into_any_element()
    }
}
