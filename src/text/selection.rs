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

use crate::constants::typography::{line_height_for_font_size, FontType, WEIGHT_BOLD};

use crate::text::metrics::effective_weight_multiplier;
pub(crate) use crate::text::metrics::get_char_width_for_font;

/// Measures the estimated rendered width of a string with font-type awareness.
pub(crate) fn calculate_text_width_for_font(
    text: &str,
    font_size: f32,
    weight_multiplier: f32,
    font_type: FontType,
) -> f32 {
    let weight_multiplier = effective_weight_multiplier(weight_multiplier, font_type);
    let mut width = 0.0f32;
    for ch in text.chars() {
        width += get_char_width_for_font(ch, font_size, font_type) * weight_multiplier;
    }
    width
}

/// Maps a click position inside a line to closest character index with font-type awareness.
pub(crate) fn calculate_line_text_offset_with_font(
    rel_x: f32,
    text: &str,
    font_size: f32,
    weight_multiplier: f32,
    font_type: FontType,
) -> usize {
    let char_count = text.chars().count();
    if text.is_empty() || char_count == 0 || rel_x <= 0.0 {
        return 0;
    }

    let weight_multiplier = effective_weight_multiplier(weight_multiplier, font_type);
    let mut current_x = 0.0f32;

    for (idx, ch) in text.chars().enumerate() {
        let w = get_char_width_for_font(ch, font_size, font_type) * weight_multiplier;
        let midpoint = current_x + (w / 2.0);
        let end_x = current_x + w;

        if rel_x < midpoint {
            return idx;
        } else if rel_x < end_x {
            return idx + 1;
        }
        current_x = end_x;
    }

    char_count
}

/// Maps a click position inside a line to the closest character index with a uniform weight multiplier.
#[cfg(test)]
pub(crate) fn calculate_line_text_offset_weighted(
    rel_x: f32,
    text: &str,
    font_size: f32,
    weight_multiplier: f32,
) -> usize {
    calculate_line_text_offset_with_font(
        rel_x,
        text,
        font_size,
        weight_multiplier,
        FontType::Calibri,
    )
}

/// Maps a click position inside a bold-aware line to character index with font-type awareness.
pub(crate) fn calculate_line_text_offset_with_bold_and_font(
    rel_x: f32,
    text: &str,
    bold_flags: Option<&[bool]>,
    font_size: f32,
    font_type: FontType,
) -> usize {
    let char_count = text.chars().count();
    if text.is_empty() || char_count == 0 || rel_x <= 0.0 {
        return 0;
    }

    let bold_multiplier = effective_weight_multiplier(WEIGHT_BOLD, font_type);
    let mut current_x = 0.0f32;

    for (idx, ch) in text.chars().enumerate() {
        let is_bold = bold_flags
            .and_then(|f| f.get(idx).copied())
            .unwrap_or(false);
        let mut w = get_char_width_for_font(ch, font_size, font_type);
        if is_bold {
            w *= bold_multiplier;
        }
        let midpoint = current_x + (w / 2.0);
        let end_x = current_x + w;

        if rel_x < midpoint {
            return idx;
        } else if rel_x < end_x {
            return idx + 1;
        }
        current_x = end_x;
    }

    char_count
}

/// Computes the canvas character index taking font size and font type into account.
#[cfg(test)]
pub(crate) fn calculate_canvas_text_offset_full(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    bold_flags: Option<&[bool]>,
    header_x_offset: f32,
    header_y_offset: f32,
    _textbox_width: f32,
    font_size: f32,
    font_type: FontType,
) -> usize {
    let sidebar_w = if sidebar_open { 220.0 } else { 44.0 };
    let rel_x = mouse_pos.x.as_f32() - sidebar_w - pan_x - item_x - header_x_offset;
    let rel_y = mouse_pos.y.as_f32() - canvas_top_y - pan_y - item_y - header_y_offset;

    let total_char_count = text.chars().count();
    if text.is_empty() || total_char_count == 0 {
        return 0;
    }

    let logical_lines: Vec<&str> = text.split('\n').collect();
    let num_lines = logical_lines.len();

    let line_height = line_height_for_font_size(font_size);
    let target_idx = if rel_y < 0.0 {
        0
    } else {
        let raw = (rel_y / line_height).floor() as usize;
        raw.min(num_lines - 1)
    };

    let target_line = logical_lines[target_idx];
    let mut line_start_global = 0;
    for i in 0..target_idx {
        line_start_global += logical_lines[i].chars().count() + 1;
    }

    let target_line_len = target_line.chars().count();
    let line_flags = bold_flags.and_then(|flags| {
        if line_start_global < flags.len() {
            let end = (line_start_global + target_line_len).min(flags.len());
            Some(&flags[line_start_global..end])
        } else {
            None
        }
    });

    let local_char_offset = calculate_line_text_offset_with_bold_and_font(
        rel_x,
        target_line,
        line_flags,
        font_size,
        font_type,
    );

    (line_start_global + local_char_offset).min(total_char_count)
}

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
/// Built when the text, font, or bold flags change. Pointer moves binary-search it
/// instead of measuring every character again.
#[derive(Clone, Debug)]
pub(crate) struct TextHitCache {
    pub(crate) font_size: f32,
    pub(crate) font_type: FontType,
    pub(crate) byte_len: usize,
    pub(crate) bold_len: usize,
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

/// Advance widths for one line, using the same bold metric as hit testing.
pub(crate) fn line_prefix(
    text: &str,
    bold_flags: Option<&[bool]>,
    font_size: f32,
    font_type: FontType,
) -> Vec<f32> {
    let bold_multiplier = effective_weight_multiplier(WEIGHT_BOLD, font_type);
    let mut prefix = Vec::with_capacity(text.chars().count() + 1);
    let mut x = 0.0f32;
    prefix.push(0.0);
    for (idx, ch) in text.chars().enumerate() {
        let is_bold = bold_flags
            .and_then(|flags| flags.get(idx).copied())
            .unwrap_or(false);
        let mut w = get_char_width_for_font(ch, font_size, font_type);
        if is_bold {
            w *= bold_multiplier;
        }
        x += w;
        prefix.push(x);
    }
    prefix
}

/// Widest logical line, using the same advances as the caret.
///
/// The text box must be at least this wide or the line wraps and the caret, which follows
/// logical lines, lands on the wrong row after Enter.
pub(crate) fn max_text_advance(
    text: &str,
    bold_flags: Option<&[bool]>,
    font_size: f32,
    font_type: FontType,
) -> f32 {
    let mut max_w = 0.0f32;
    let mut start = 0usize;
    for line in text.split('\n') {
        let line_len = line.chars().count();
        let flags = bold_flags.and_then(|all| {
            if start < all.len() {
                let end = (start + line_len).min(all.len());
                Some(&all[start..end])
            } else {
                None
            }
        });
        let prefix = line_prefix(line, flags, font_size, font_type);
        max_w = max_w.max(prefix.last().copied().unwrap_or(0.0));
        start += line_len + 1;
    }
    max_w
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

/// Builds per-line prefix sums for `text`. `stamp` is stored so callers can tell the cache is current.
pub(crate) fn build_text_hit_cache(
    text: &str,
    bold_flags: Option<&[bool]>,
    font_size: f32,
    font_type: FontType,
    stamp: u64,
) -> TextHitCache {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for line in text.split('\n') {
        let line_len = line.chars().count();
        let flags = bold_flags.and_then(|all| {
            if start < all.len() {
                let end = (start + line_len).min(all.len());
                Some(&all[start..end])
            } else {
                None
            }
        });
        lines.push(LineHit {
            start,
            prefix: line_prefix(line, flags, font_size, font_type),
        });
        start += line_len + 1;
    }

    TextHitCache {
        font_size,
        font_type,
        byte_len: text.len(),
        bold_len: bold_flags.map(|flags| flags.len()).unwrap_or(0),
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
    fn test_calculate_line_text_offset_scales_with_font_size() {
        let text = "Hello World";
        // At 14px, clicking at x = 0 should be index 0
        assert_eq!(calculate_line_text_offset_weighted(0.0, text, 14.0, 1.0), 0);

        // At 14px vs 28px, the same character should be at double the x offset
        let w_14 = calculate_text_width_for_font("Hello", 14.0, 1.0, FontType::Calibri);
        let w_28 = calculate_text_width_for_font("Hello", 28.0, 1.0, FontType::Calibri);
        assert!((w_28 - w_14 * 2.0).abs() < 0.001);

        // Clicking right after "Hello" should return 5 for both font sizes
        assert_eq!(
            calculate_line_text_offset_weighted(w_14, text, 14.0, 1.0),
            5
        );
        assert_eq!(
            calculate_line_text_offset_weighted(w_28, text, 28.0, 1.0),
            5
        );
    }

    #[test]
    fn test_calculate_line_text_offset_with_weight() {
        let text = "Section Title";
        let w_normal = calculate_text_width_for_font(text, 11.0, 1.00, FontType::Calibri);
        let w_bold = calculate_text_width_for_font(text, 11.0, 1.10, FontType::Calibri);
        assert!(w_bold > w_normal);

        // Clicking beyond the end returns full length
        assert_eq!(
            calculate_line_text_offset_weighted(999.0, text, 11.0, 1.10),
            text.len()
        );
    }

    #[test]
    fn test_monospace_font_uniform_character_width() {
        let w_i = get_char_width_for_font('i', 12.0, FontType::Monospace);
        let w_m = get_char_width_for_font('m', 12.0, FontType::Monospace);
        let w_space = get_char_width_for_font(' ', 12.0, FontType::Monospace);
        let w_w = get_char_width_for_font('W', 12.0, FontType::Monospace);

        // In monospace, every character has identical width
        assert!((w_i - w_m).abs() < 0.0001);
        assert!((w_i - w_space).abs() < 0.0001);
        assert!((w_i - w_w).abs() < 0.0001);

        // Clicking exactly at 3.5 chars should map to index 3 or 4 accurately
        let text = "iiii";
        let offset_0 =
            calculate_line_text_offset_with_font(0.0, text, 12.0, 1.0, FontType::Monospace);
        let offset_2 =
            calculate_line_text_offset_with_font(w_i * 2.0, text, 12.0, 1.0, FontType::Monospace);
        let offset_4 =
            calculate_line_text_offset_with_font(w_i * 4.0, text, 12.0, 1.0, FontType::Monospace);
        assert_eq!(offset_0, 0);
        assert_eq!(offset_2, 2);
        assert_eq!(offset_4, 4);
    }

    #[test]
    fn test_font_family_inference_and_offsets() {
        assert_eq!(FontType::from_family_name("Consolas"), FontType::Monospace);
        assert_eq!(
            FontType::from_family_name("Courier New"),
            FontType::Monospace
        );
        assert_eq!(FontType::from_family_name("Arial"), FontType::Arial);
        assert_eq!(
            FontType::from_family_name("Times New Roman"),
            FontType::Serif
        );
        assert_eq!(FontType::from_family_name("Segoe UI"), FontType::SegoeUI);

        let text = "mm";
        let w_mono = calculate_text_width_for_font(text, 12.0, 1.0, FontType::Monospace);
        let w_segoe = calculate_text_width_for_font(text, 12.0, 1.0, FontType::SegoeUI);
        // 'm' is wider in proportional fonts than monospace
        assert!(w_segoe > w_mono);
    }

    #[test]
    fn test_canvas_text_offset_full_multiline_font_scaling() {
        let text = "Line 1\nLine 2";
        // At 12px font size, line height is 20px (6.0 + 18.0 header offset)
        // Click on Line 2 (rel_y = 25.0)
        let pos_line2 = gpui::Point {
            x: gpui::px(220.0 + 10.0 + 6.0), // sidebar=220, item_x=10, header_x=6
            y: gpui::px(0.0 + 10.0 + 18.0 + 25.0), // canvas_top=0, item_y=10, header_y=18, rel_y=25 (line 2)
        };
        let idx_12 = calculate_canvas_text_offset_full(
            pos_line2,
            true,
            0.0,
            0.0,
            10.0,
            10.0,
            0.0,
            text,
            None,
            6.0,
            18.0,
            250.0,
            12.0,
            FontType::SegoeUI,
        );
        // Line 1 is 6 chars + '\n' (7), so Line 2 starts at index 7
        assert_eq!(idx_12, 7);

        // At 24px font size, line height is 40px
        // rel_y = 25.0 is still in Line 1 because line 1 spans 0..40px!
        let idx_24 = calculate_canvas_text_offset_full(
            pos_line2,
            true,
            0.0,
            0.0,
            10.0,
            10.0,
            0.0,
            text,
            None,
            6.0,
            18.0,
            250.0,
            24.0,
            FontType::SegoeUI,
        );
        assert_eq!(idx_24, 0); // Still in Line 1!
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

    #[test]
    fn test_hit_cache_matches_linear_walk() {
        let text = "Hello\nBold line stays put";
        let mut bold = vec![false; text.chars().count()];
        for flag in bold.iter_mut().skip(6).take(4) {
            *flag = true;
        }
        let cache = build_text_hit_cache(text, Some(&bold), 14.0, FontType::Calibri, 1);
        let line_height = line_height_for_font_size(14.0);

        for line_i in 0..4 {
            let rel_y = line_i as f32 * line_height + 1.0;
            for step in 0..80 {
                let rel_x = step as f32 * 1.7;
                let from_cache = cache_index_at(&cache, rel_x, rel_y);
                let from_walk = calculate_canvas_text_offset_full(
                    gpui::Point {
                        x: gpui::px(220.0 + rel_x + 6.0),
                        y: gpui::px(rel_y + 18.0),
                    },
                    true,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    text,
                    Some(&bold),
                    6.0,
                    18.0,
                    250.0,
                    14.0,
                    FontType::Calibri,
                );
                assert_eq!(from_cache, from_walk, "rel_x={rel_x} rel_y={rel_y}");
            }
        }
    }
}
