use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClipboardItem, Context, FocusHandle, Focusable, KeyDownEvent,
};
use std::ops::Range;

#[derive(Clone)]
#[allow(dead_code)]
pub struct TextEditor {
    pub text: String,
    pub cursor: usize,
    pub anchor: Option<usize>,
    pub focus_handle: FocusHandle,
    pub is_selecting: bool,
    pub cursor_visible: bool,
}

#[allow(dead_code)]
impl TextEditor {
    pub fn new(text: String, cx: &mut Context<impl Focusable>) -> Self {
        let cursor = text.chars().count();
        Self {
            text,
            cursor,
            anchor: None,
            focus_handle: cx.focus_handle(),
            is_selecting: false,
            cursor_visible: true,
        }
    }

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

    pub fn selected_text(&self) -> Option<String> {
        let range = self.selection()?;
        let chars: Vec<char> = self.text.chars().collect();
        if range.end <= chars.len() {
            Some(chars[range.start..range.end].iter().collect())
        } else {
            None
        }
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.chars().count();
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

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

    pub fn move_up(&mut self, extend_selection: bool) {
        let next_pos = crate::helpers::move_cursor_up(&self.text, self.cursor);
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = next_pos;
    }

    pub fn move_down(&mut self, extend_selection: bool) {
        let next_pos = crate::helpers::move_cursor_down(&self.text, self.cursor);
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = next_pos;
    }

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

    pub fn render_editor(&self, is_focused: bool) -> AnyElement {
        let text = self.text.as_str();
        let cursor_idx = self.cursor;
        let selection_range = self.selection();

        if text.is_empty() {
            return div()
                .flex()
                .items_center()
                .min_h(px(20.0))
                .child(if is_focused {
                    div()
                        .w(px(2.0))
                        .h(px(16.0))
                        .bg(if self.cursor_visible {
                            rgb(0x0078d4)
                        } else {
                            rgba(0x00000000)
                        })
                        .flex_shrink_0()
                        .mr(px(-2.0))
                } else {
                    div()
                })
                .child(div().text_color(rgb(0x606060)).child("Type note..."))
                .into_any_element();
        }

        let logical_lines: Vec<&str> = text.split('\n').collect();
        let mut line_rows: Vec<AnyElement> = Vec::new();
        let mut global_offset = 0;

        for (line_idx, line) in logical_lines.iter().enumerate() {
            let line_len = line.chars().count();
            let is_last_line = line_idx == logical_lines.len() - 1;
            let line_start = global_offset;
            let line_end = global_offset + line_len;

            let mut row = div()
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

                        let before_str: String = line_chars[..loc_start].iter().collect();
                        let sel_str: String = line_chars[loc_start..loc_end].iter().collect();
                        let after_str: String = line_chars[loc_end..].iter().collect();

                        if !before_str.is_empty() {
                            row = row.child(
                                div()
                                    .flex_shrink_0()
                                    .text_color(rgb(0xd4d4d4))
                                    .child(before_str.replace(' ', "\u{00A0}")),
                            );
                        }
                        if !sel_str.is_empty() {
                            row = row.child(
                                div()
                                    .flex_shrink_0()
                                    .text_color(rgb(0xffffff))
                                    .bg(rgb(0x0078d4))
                                    .rounded(px(2.0))
                                    .child(sel_str.replace(' ', "\u{00A0}")),
                            );
                        }
                        if !after_str.is_empty() {
                            row = row.child(
                                div()
                                    .flex_shrink_0()
                                    .text_color(rgb(0xd4d4d4))
                                    .child(after_str.replace(' ', "\u{00A0}")),
                            );
                        }
                    } else if line_len == 0 && sel.start <= line_start && sel.end > line_start {
                        row = row.child(
                            div()
                                .w(px(6.0))
                                .h(px(14.0))
                                .bg(rgb(0x0078d4))
                                .rounded(px(2.0)),
                        );
                    }
                }
            } else {
                // Render line with caret if line contains cursor and focused
                let is_cursor_in_line = if is_last_line {
                    cursor_idx >= line_start && cursor_idx <= line_end
                } else {
                    cursor_idx >= line_start && cursor_idx <= line_end
                };

                if is_focused && is_cursor_in_line && selection_range.is_none() {
                    let local_cursor = cursor_idx.saturating_sub(line_start).min(line_len);
                    let line_chars: Vec<char> = line.chars().collect();
                    let before_str: String = line_chars[..local_cursor].iter().collect();
                    let after_str: String = line_chars[local_cursor..].iter().collect();

                    if !before_str.is_empty() {
                        row = row.child(
                            div()
                                .text_color(rgb(0xd4d4d4))
                                .child(before_str.replace(' ', "\u{00A0}")),
                        );
                    }
                    row = row.child(
                        div()
                            .w(px(2.0))
                            .h(px(16.0))
                            .bg(if self.cursor_visible {
                                rgb(0x0078d4)
                            } else {
                                rgba(0x00000000)
                            })
                            .flex_shrink_0()
                            .mr(px(-2.0)),
                    );
                    if !after_str.is_empty() {
                        row = row.child(
                            div()
                                .text_color(rgb(0xd4d4d4))
                                .child(after_str.replace(' ', "\u{00A0}")),
                        );
                    }
                } else {
                    if !line.is_empty() {
                        row = row.child(
                            div()
                                .text_color(rgb(0xd4d4d4))
                                .child(line.replace(' ', "\u{00A0}")),
                        );
                    }
                }
            }

            line_rows.push(row.into_any_element());
            global_offset += line_len + 1; // +1 for \n
        }

        div()
            .flex()
            .flex_col()
            .children(line_rows)
            .into_any_element()
    }
}
