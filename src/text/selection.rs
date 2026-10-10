//! Selection ranges, line widths, canvas hit testing, and cursor movement.

/// Converts a cursor/anchor pair into the normalized start-end selection range.
pub(crate) fn get_selection_range(cursor: usize, anchor: Option<usize>) -> Option<(usize, usize)> {
    if let Some(anchor) = anchor {
        if anchor != cursor {
            return Some((cursor.min(anchor), cursor.max(anchor)));
        }
    }
    None
}

use crate::constants::typography::line_height_for_font_size;

/// One visual line: the buffer index where it starts, and the x position before each character.
///
/// `prefix[i]` is the advance at character `i`. `prefix[len]` is the x at the end of the line.
#[derive(Clone, Debug)]
pub(crate) struct LineHit {
    pub(crate) start: usize,
    pub(crate) prefix: Vec<f32>,
}

/// Prefix sums of character advances for a whole text buffer.
///
/// Built when the text, font, or zoom changes. Pointer moves binary-search it
/// instead of shaping every character again.
#[derive(Clone, Debug)]
pub(crate) struct TextHitCache {
    pub(crate) font_size: f32,
    pub(crate) zoom: f32,
    pub(crate) content_hash: u64,
    pub(crate) stamp: u64,
    pub(crate) lines: Vec<LineHit>,
    pub(crate) total_chars: usize,
}

/// Writes `next` into `slot` and reports whether the value changed.
#[inline]
pub(crate) fn assign_if_changed(slot: &mut usize, next: usize) -> bool {
    if *slot != next {
        *slot = next;
        true
    } else {
        false
    }
}

/// X where a line's glyphs start, in the same units as character advances.
///
/// `content_width` is the flex row width. Center and right alignment consume the free
/// space after indent; left alignment only shifts by the indent.
pub(crate) fn aligned_line_origin(
    content_width: f32,
    layout: crate::models::LineLayout,
    text_advance: f32,
) -> f32 {
    let indent = layout.indent as f32 * crate::constants::typography::INDENT_STEP;
    let slack = content_width - indent - text_advance;
    match layout.align {
        1 => indent + slack * 0.5,
        2 => indent + slack,
        _ => indent,
    }
}

/// X position of a character index inside a line prefix. The index may sit at the end of the line.
#[inline]
pub(crate) fn prefix_x(prefix: &[f32], local_index: usize) -> f32 {
    if prefix.is_empty() {
        return 0.0;
    }
    prefix[local_index.min(prefix.len() - 1)]
}

/// Closest character index for an x position, matching the midpoint rule of the linear walk.
pub(crate) fn index_in_prefix(prefix: &[f32], rel_x: f32) -> usize {
    let char_count = prefix.len().saturating_sub(1);
    if char_count == 0 || rel_x <= 0.0 {
        return 0;
    }
    if rel_x >= prefix[char_count] {
        return char_count;
    }

    let mut lo = 0usize;
    let mut hi = char_count;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if prefix[mid + 1] <= rel_x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo >= char_count {
        return char_count;
    }
    let start = prefix[lo];
    let end = prefix[lo + 1];
    let midpoint = start + (end - start) / 2.0;
    if rel_x < midpoint {
        lo
    } else {
        (lo + 1).min(char_count)
    }
}

/// Widest line in a cache built by [`build_text_hit_cache`].
pub(crate) fn cache_max_advance(cache: &TextHitCache) -> f32 {
    cache
        .lines
        .iter()
        .filter_map(|line| line.prefix.last().copied())
        .fold(0.0f32, f32::max)
}

fn slice_range<T>(values: &[T], start: usize, len: usize) -> &[T] {
    if start >= values.len() {
        &[]
    } else {
        let end = (start + len).min(values.len());
        &values[start..end]
    }
}

pub(crate) fn content_hash(
    text: &str,
    bold: &[bool],
    italic: &[bool],
    families: &[u8],
    sizes: &[f32],
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    bold.hash(&mut hasher);
    italic.hash(&mut hasher);
    families.hash(&mut hasher);
    for size in sizes {
        size.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

/// Builds per-line caret positions from GPUI's shaper. `stamp` lets callers tell the cache is current.
pub(crate) fn build_text_hit_cache(
    window: &gpui::Window,
    text: &str,
    bold: &[bool],
    italic: &[bool],
    families: &[u8],
    sizes: &[f32],
    font_size: f32,
    zoom: f32,
    stamp: u64,
) -> TextHitCache {
    let zoom = zoom.max(0.05);
    let mut lines = Vec::new();
    let mut start = 0usize;
    for line in text.split('\n') {
        let line_len = line.chars().count();
        lines.push(LineHit {
            start,
            prefix: crate::text::shaping::shaped_line_prefix(
                window,
                line,
                slice_range(bold, start, line_len),
                slice_range(italic, start, line_len),
                slice_range(families, start, line_len),
                slice_range(sizes, start, line_len),
                zoom,
            ),
        });
        start += line_len + 1;
    }

    TextHitCache {
        font_size,
        zoom,
        content_hash: content_hash(text, bold, italic, families, sizes),
        stamp,
        lines,
        total_chars: text.chars().count(),
    }
}

/// Line under a y position inside the text block. `rel_y` is local to the first line.
pub(crate) fn cache_line_at_y(cache: &TextHitCache, rel_y: f32) -> usize {
    if cache.lines.is_empty() {
        return 0;
    }
    let line_height = line_height_for_font_size(cache.font_size);
    let last = cache.lines.len() - 1;
    if rel_y < 0.0 {
        0
    } else {
        ((rel_y / line_height).floor() as usize).min(last)
    }
}

/// Maps a point inside the text block to a character index using cached line advances.
pub(crate) fn cache_index_at(cache: &TextHitCache, rel_x: f32, rel_y: f32) -> usize {
    if cache.lines.is_empty() {
        return 0;
    }
    let line_idx = cache_line_at_y(cache, rel_y);
    let line = &cache.lines[line_idx];
    let local = index_in_prefix(&line.prefix, rel_x);
    (line.start + local).min(cache.total_chars)
}

/// Character index on one cached line. `rel_x` is already local to that line.
pub(crate) fn cache_index_on_line(cache: &TextHitCache, line_start: usize, rel_x: f32) -> usize {
    if let Some(line) = cache.lines.iter().find(|line| line.start == line_start) {
        let local = index_in_prefix(&line.prefix, rel_x);
        return (line.start + local).min(cache.total_chars);
    }
    0
}

/// Window point to the text block's local origin, matching `calculate_canvas_text_offset_full`.
pub(crate) fn canvas_text_rel(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_w: f32,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    header_x_offset: f32,
    header_y_offset: f32,
    zoom: f32,
) -> (f32, f32) {
    let zoom = if zoom <= 0.05 { 1.0 } else { zoom };
    // Header chrome stays a fixed screen size, so it is removed before the zoom division.
    let rel_x = (mouse_pos.x.as_f32() - sidebar_w - pan_x - header_x_offset) / zoom - item_x;
    let rel_y = (mouse_pos.y.as_f32() - canvas_top_y - pan_y - header_y_offset) / zoom - item_y;
    (rel_x, rel_y)
}

/// Moves a cursor upward by one visual line while keeping the same column whenever possible.
pub(crate) fn move_cursor_up(text: &str, cursor: usize) -> usize {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut current_offset = 0;
    let mut current_line_idx = 0;
    let mut col_offset = 0;

    for (idx, line) in lines.iter().enumerate() {
        let line_len = line.chars().count();
        if cursor >= current_offset && cursor <= current_offset + line_len {
            current_line_idx = idx;
            col_offset = cursor - current_offset;
            break;
        }
        current_offset += line_len + 1;
    }

    if current_line_idx > 0 {
        let target_line_idx = current_line_idx - 1;
        let mut target_offset = 0;
        for i in 0..target_line_idx {
            target_offset += lines[i].chars().count() + 1;
        }
        let target_line_len = lines[target_line_idx].chars().count();
        target_offset + col_offset.min(target_line_len)
    } else {
        0
    }
}

/// Moves a cursor downward by one visual line while keeping the same column whenever possible.
pub(crate) fn move_cursor_down(text: &str, cursor: usize) -> usize {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut current_offset = 0;
    let mut current_line_idx = lines.len() - 1;
    let mut col_offset = 0;

    for (idx, line) in lines.iter().enumerate() {
        let line_len = line.chars().count();
        if cursor >= current_offset && cursor <= current_offset + line_len {
            current_line_idx = idx;
            col_offset = cursor - current_offset;
            break;
        }
        current_offset += line_len + 1;
    }

    if current_line_idx + 1 < lines.len() {
        let target_line_idx = current_line_idx + 1;
        let mut target_offset = 0;
        for i in 0..target_line_idx {
            target_offset += lines[i].chars().count() + 1;
        }
        let target_line_len = lines[target_line_idx].chars().count();
        target_offset + col_offset.min(target_line_len)
    } else {
        text.chars().count()
    }
}

/// Moves the cursor to the previous word boundary in the text value.
pub(crate) fn move_cursor_word_left(text: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut pos = cursor;
    while pos > 0 && !chars[pos - 1].is_alphanumeric() {
        pos -= 1;
    }
    while pos > 0 && chars[pos - 1].is_alphanumeric() {
        pos -= 1;
    }
    pos
}

/// Moves the cursor to the next word boundary in the text value.
pub(crate) fn move_cursor_word_right(text: &str, cursor: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if cursor >= len {
        return len;
    }
    let mut pos = cursor;
    while pos < len && chars[pos].is_alphanumeric() {
        pos += 1;
    }
    while pos < len && !chars[pos].is_alphanumeric() {
        pos += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_in_prefix_uses_character_midpoints() {
        let prefix = vec![0.0, 10.0, 30.0, 36.0];
        assert_eq!(index_in_prefix(&prefix, 0.0), 0);
        assert_eq!(index_in_prefix(&prefix, 4.9), 0);
        assert_eq!(index_in_prefix(&prefix, 5.0), 1);
        assert_eq!(index_in_prefix(&prefix, 19.9), 1);
        assert_eq!(index_in_prefix(&prefix, 20.0), 2);
        assert_eq!(index_in_prefix(&prefix, 36.0), 3);
        assert_eq!(index_in_prefix(&prefix, 80.0), 3);
        assert_eq!(index_in_prefix(&[0.0], 12.0), 0);
    }

    #[test]
    fn test_cache_line_at_y_follows_line_height() {
        let lines = vec![
            LineHit {
                start: 0,
                prefix: vec![0.0],
            },
            LineHit {
                start: 7,
                prefix: vec![0.0],
            },
        ];
        let at_12 = TextHitCache {
            font_size: 12.0,
            zoom: 1.0,
            content_hash: 0,
            stamp: 0,
            lines: lines.clone(),
            total_chars: 13,
        };
        let at_24 = TextHitCache {
            font_size: 24.0,
            ..at_12.clone()
        };
        // 12px lines are 20px tall, so y=25 is the second line.
        assert_eq!(cache_line_at_y(&at_12, 25.0), 1);
        // 24px lines are 32px tall, so y=25 is still the first line.
        assert_eq!(cache_line_at_y(&at_24, 25.0), 0);
    }

    #[test]
    fn test_dynamic_font_metrics_helpers() {
        use crate::constants::typography::{
            cursor_height_for_font_size, line_height_for_font_size, selection_height_for_font_size,
        };
        assert!((line_height_for_font_size(12.0) - 20.0).abs() < 0.01);
        assert!((line_height_for_font_size(24.0) - 32.0).abs() < 0.01);

        assert!((cursor_height_for_font_size(12.0) - 16.0).abs() < 0.01);
        assert!((cursor_height_for_font_size(24.0) - 28.0).abs() < 0.01);

        assert!((selection_height_for_font_size(12.0) - 18.0).abs() < 0.01);
        assert!((selection_height_for_font_size(24.0) - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_aligned_line_origin_centers_and_right_aligns() {
        use crate::models::LineLayout;

        let left = LineLayout {
            indent: 0,
            align: 0,
        };
        let center = LineLayout {
            indent: 0,
            align: 1,
        };
        let right = LineLayout {
            indent: 0,
            align: 2,
        };
        let indented = LineLayout {
            indent: 1,
            align: 1,
        };

        assert!((aligned_line_origin(200.0, left, 40.0) - 0.0).abs() < 0.01);
        assert!((aligned_line_origin(200.0, center, 40.0) - 80.0).abs() < 0.01);
        assert!((aligned_line_origin(200.0, right, 40.0) - 160.0).abs() < 0.01);
        // Indent 24, then center the remaining slack: 24 + (200 - 24 - 40) / 2 = 92.
        assert!((aligned_line_origin(200.0, indented, 40.0) - 92.0).abs() < 0.01);
    }
}
