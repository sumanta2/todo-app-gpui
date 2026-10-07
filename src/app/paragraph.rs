//! Line indent and alignment for the text box, and the same commands for a selected image.

use gpui::Context;

use crate::app::NotesApp;
use crate::constants::typography::{INDENT_STEP, MAX_INDENT_LEVEL};
use crate::models::{CanvasItem, ContentBlock, LineLayout};
use crate::text::selection::get_selection_range;

impl NotesApp {
    /// Increases the indent of the selected lines, or nudges a selected image right.
    pub(crate) fn increase_indent(&mut self, cx: &mut Context<Self>) {
        if self.is_editing {
            self.record_edit(crate::app::history::EditKind::Format);
        }
        if self.nudge_selected_image(INDENT_STEP) {
            self.schedule_autosave(cx);
            cx.notify();
        } else {
            self.change_line_indent(1, cx);
        }
        if self.is_editing {
            self.discard_unchanged_edit();
        }
    }

    /// Decreases the indent of the selected lines, or nudges a selected image left.
    pub(crate) fn decrease_indent(&mut self, cx: &mut Context<Self>) {
        if self.is_editing {
            self.record_edit(crate::app::history::EditKind::Format);
        }
        if self.nudge_selected_image(-INDENT_STEP) {
            self.schedule_autosave(cx);
            cx.notify();
        } else {
            self.change_line_indent(-1, cx);
        }
        if self.is_editing {
            self.discard_unchanged_edit();
        }
    }

    /// Aligns the selected lines, or a selected image, to the left, center, or right.
    pub(crate) fn align_content(&mut self, align: u8, cx: &mut Context<Self>) {
        if self.is_editing {
            self.record_edit(crate::app::history::EditKind::Format);
        }
        if self.align_selected_image(align) {
            self.schedule_autosave(cx);
            cx.notify();
        } else {
            self.change_line_align(align, cx);
        }
        if self.is_editing {
            self.discard_unchanged_edit();
        }
    }

    /// Alignment of the caret line, or of the selected image when one is active.
    pub(crate) fn current_align(&self) -> u8 {
        if let Some(align) = self.selected_image_align() {
            return align;
        }
        if !self.is_editing || self.active_field != crate::models::ActiveField::Body {
            return 0;
        }
        let line = caret_line(
            &self.edit_body,
            self.edit_body_cursor,
            self.edit_body_anchor,
        );
        self.edit_body_line_layouts
            .get(line)
            .map(|layout| layout.align)
            .unwrap_or(0)
    }

    /// Keeps line layouts lined up with the body after typing inserts or removes line breaks.
    pub(crate) fn reconcile_line_layouts(&mut self) {
        if self.line_layout_anchor == self.edit_body {
            return;
        }
        let (start, end, inserted) = simple_edit(&self.line_layout_anchor, &self.edit_body);
        splice_line_layouts(
            &mut self.edit_body_line_layouts,
            &self.line_layout_anchor,
            start,
            end,
            &inserted,
        );
        self.line_layout_anchor = self.edit_body.clone();
    }

    /// Marks the current body text as the baseline for line-layout tracking.
    pub(crate) fn bind_line_layouts(&mut self) {
        self.line_layout_anchor = self.edit_body.clone();
    }

    /// Splits line layouts the same way an image splits a text block.
    pub(crate) fn split_line_layouts_at(
        &self,
        cursor: usize,
    ) -> (Vec<LineLayout>, Vec<LineLayout>) {
        let line = line_index(&self.edit_body, cursor);
        let layouts = &self.edit_body_line_layouts;
        let before = (0..=line).map(|idx| layout_at(layouts, idx)).collect();
        let after_count = self.edit_body.split('\n').count().saturating_sub(line);
        let after = (0..after_count)
            .map(|idx| layout_at(layouts, line + idx))
            .collect();
        (before, after)
    }

    fn change_line_indent(&mut self, delta: i32, cx: &mut Context<Self>) {
        if !self.is_editing || self.active_field != crate::models::ActiveField::Body {
            return;
        }
        self.touch_affected_lines(|layout| {
            let next = layout.indent as i32 + delta;
            layout.indent = next.clamp(0, MAX_INDENT_LEVEL as i32) as u8;
        });
        self.touch_body_layout();
        self.sync_active_text_block();
        self.schedule_autosave(cx);
        cx.notify();
    }

    fn change_line_align(&mut self, align: u8, cx: &mut Context<Self>) {
        if !self.is_editing || self.active_field != crate::models::ActiveField::Body {
            return;
        }
        self.touch_affected_lines(|layout| {
            layout.align = align.min(2);
        });
        self.touch_body_layout();
        self.sync_active_text_block();
        self.schedule_autosave(cx);
        cx.notify();
    }

    fn touch_affected_lines(&mut self, mut change: impl FnMut(&mut LineLayout)) {
        let lines = affected_lines(
            &self.edit_body,
            self.edit_body_cursor,
            self.edit_body_anchor,
        );
        let line_count = self.edit_body.split('\n').count();
        if self.edit_body_line_layouts.len() < line_count {
            self.edit_body_line_layouts
                .resize(line_count, LineLayout::default());
        }
        for line in lines {
            if let Some(layout) = self.edit_body_line_layouts.get_mut(line) {
                change(layout);
            }
        }
    }

    fn nudge_selected_image(&mut self, delta: f32) -> bool {
        let Some((item_id, block_idx)) = self.selected_inline_image.clone() else {
            return false;
        };
        let Some(item) = self.edit_canvas_items.iter_mut().find(|item| match item {
            CanvasItem::Mixed(mixed) => mixed.id == item_id,
            _ => false,
        }) else {
            return false;
        };
        let CanvasItem::Mixed(mixed) = item else {
            return false;
        };
        let box_w = mixed.width.unwrap_or(250.0);
        let Some(ContentBlock::Image {
            width, offset_x, ..
        }) = mixed.blocks.get_mut(block_idx)
        else {
            return false;
        };
        let max_off = (box_w - *width).max(0.0);
        *offset_x = (*offset_x + delta).clamp(0.0, max_off);
        true
    }

    fn align_selected_image(&mut self, align: u8) -> bool {
        let Some((item_id, block_idx)) = self.selected_inline_image.clone() else {
            return false;
        };
        let box_w = self.edit_canvas_items.iter().find_map(|item| match item {
            CanvasItem::Mixed(mixed) if mixed.id == item_id => Some(mixed.width.unwrap_or(250.0)),
            _ => None,
        });
        let Some(box_w) = box_w else {
            return false;
        };
        let is_image = self.edit_canvas_items.iter().any(|item| match item {
            CanvasItem::Mixed(mixed) if mixed.id == item_id => mixed
                .blocks
                .get(block_idx)
                .is_some_and(|block| matches!(block, ContentBlock::Image { .. })),
            _ => false,
        });
        if !is_image {
            return false;
        }
        self.align_inline_image(&item_id, block_idx, align as i32, box_w);
        true
    }

    fn selected_image_align(&self) -> Option<u8> {
        let (item_id, block_idx) = self.selected_inline_image.as_ref()?;
        let mixed = self.edit_canvas_items.iter().find_map(|item| match item {
            CanvasItem::Mixed(mixed) if mixed.id == *item_id => Some(mixed),
            _ => None,
        })?;
        let ContentBlock::Image {
            width, offset_x, ..
        } = mixed.blocks.get(*block_idx)?
        else {
            return None;
        };
        let box_w = mixed.width.unwrap_or(250.0);
        let max_off = (box_w - *width).max(0.0);
        if max_off < 1.0 || *offset_x <= 1.0 {
            Some(0)
        } else if (*offset_x - max_off).abs() <= 1.0 {
            Some(2)
        } else if (*offset_x - max_off / 2.0).abs() <= 1.0 {
            Some(1)
        } else {
            Some(0)
        }
    }
}

fn affected_lines(text: &str, cursor: usize, anchor: Option<usize>) -> Vec<usize> {
    let (start, end) = get_selection_range(cursor, anchor).unwrap_or((cursor, cursor));
    let start_line = line_index(text, start);
    let end_line = if end > start {
        line_index(text, end - 1)
    } else {
        start_line
    };
    (start_line..=end_line).collect()
}

fn caret_line(text: &str, cursor: usize, anchor: Option<usize>) -> usize {
    affected_lines(text, cursor, anchor)
        .first()
        .copied()
        .unwrap_or(0)
}

fn line_index(text: &str, cursor: usize) -> usize {
    text.chars().take(cursor).filter(|ch| *ch == '\n').count()
}

fn layout_at(layouts: &[LineLayout], index: usize) -> LineLayout {
    layouts.get(index).copied().unwrap_or_default()
}

fn simple_edit(old: &str, new: &str) -> (usize, usize, String) {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    let mut prefix = 0;
    while prefix < old_chars.len()
        && prefix < new_chars.len()
        && old_chars[prefix] == new_chars[prefix]
    {
        prefix += 1;
    }
    let mut old_end = old_chars.len();
    let mut new_end = new_chars.len();
    while old_end > prefix && new_end > prefix && old_chars[old_end - 1] == new_chars[new_end - 1] {
        old_end -= 1;
        new_end -= 1;
    }
    let inserted: String = new_chars[prefix..new_end].iter().collect();
    (prefix, old_end, inserted)
}

fn splice_line_layouts(
    layouts: &mut Vec<LineLayout>,
    old_text: &str,
    start: usize,
    end: usize,
    inserted: &str,
) {
    let old_lines = old_text.split('\n').count();
    if layouts.len() < old_lines {
        layouts.resize(old_lines, LineLayout::default());
    }
    let start_line = line_index(old_text, start);
    let end_line = line_index(old_text, end);
    let remove = end_line.saturating_sub(start_line);
    let insert = inserted.chars().filter(|ch| *ch == '\n').count();
    let inherited = layout_at(layouts, start_line);
    let remove_at = (start_line + 1).min(layouts.len());
    let remove_end = (remove_at + remove).min(layouts.len());
    if remove_at < remove_end {
        layouts.drain(remove_at..remove_end);
    }
    for offset in 0..insert {
        let at = (remove_at + offset).min(layouts.len());
        layouts.insert(at, inherited);
    }
}

pub(crate) fn saved_line_layouts(layouts: &[LineLayout]) -> Vec<LineLayout> {
    if layouts
        .iter()
        .all(|layout| layout.indent == 0 && layout.align == 0)
    {
        Vec::new()
    } else {
        layouts.to_vec()
    }
}
