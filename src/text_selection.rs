/// Converts a cursor/anchor pair into the normalized start-end selection range.
pub(crate) fn get_selection_range(cursor: usize, anchor: Option<usize>) -> Option<(usize, usize)> {
    if let Some(anchor) = anchor {
        if anchor != cursor {
            return Some((cursor.min(anchor), cursor.max(anchor)));
        }
    }
    None
}

use crate::constants::typography::{
    line_height_for_font_size, CANVAS_BODY_FONT_SIZE, DEFAULT_FONT_FAMILY, FontType,
};

pub(crate) use crate::font_metrics::get_char_width_for_font;

/// Estimates the rendered width of a character at a given font size with default Segoe UI metrics.
#[allow(dead_code)]
pub(crate) fn get_char_width(ch: char, font_size: f32) -> f32 {
    get_char_width_for_font(ch, font_size, FontType::from_family_name(DEFAULT_FONT_FAMILY))
}

/// Measures the estimated rendered width of a string with font-type awareness.
pub(crate) fn calculate_text_width_for_font(
    text: &str,
    font_size: f32,
    weight_multiplier: f32,
    font_type: FontType,
) -> f32 {
    let mut width = 0.0f32;
    for ch in text.chars() {
        width += get_char_width_for_font(ch, font_size, font_type) * weight_multiplier;
    }
    width
}

/// Measures the estimated rendered width of a string at a specific font size and weight multiplier.
#[allow(dead_code)]
pub(crate) fn calculate_text_width(text: &str, font_size: f32, weight_multiplier: f32) -> f32 {
    calculate_text_width_for_font(
        text,
        font_size,
        weight_multiplier,
        FontType::from_family_name(DEFAULT_FONT_FAMILY),
    )
}

/// Maps an x-position inside a single line back to the nearest character index.
#[allow(dead_code)]
pub(crate) fn calculate_line_text_offset(rel_x: f32, text: &str, font_size: f32) -> usize {
    calculate_line_text_offset_weighted(rel_x, text, font_size, 1.0)
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
        FontType::from_family_name(DEFAULT_FONT_FAMILY),
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

    let mut current_x = 0.0f32;

    for (idx, ch) in text.chars().enumerate() {
        let is_bold = bold_flags
            .and_then(|f| f.get(idx).copied())
            .unwrap_or(false);
        let mut w = get_char_width_for_font(ch, font_size, font_type);
        if is_bold {
            w *= 1.10;
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

/// Maps a click position inside a bold-aware text line to the closest character index.
#[allow(dead_code)]
pub(crate) fn calculate_line_text_offset_with_bold(
    rel_x: f32,
    text: &str,
    bold_flags: Option<&[bool]>,
    font_size: f32,
) -> usize {
    calculate_line_text_offset_with_bold_and_font(
        rel_x,
        text,
        bold_flags,
        font_size,
        FontType::from_family_name(DEFAULT_FONT_FAMILY),
    )
}

/// Computes the canvas character index taking font size and font type into account.
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

/// Computes the character index for a click inside a canvas text block.
#[allow(dead_code)]
pub(crate) fn calculate_canvas_text_offset(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    textbox_width: f32,
) -> usize {
    calculate_canvas_text_offset_with_header_and_width(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        6.0,
        18.0,
        textbox_width,
    )
}

/// Computes a canvas character offset when the text block includes a header offset.
#[allow(dead_code)]
pub(crate) fn calculate_canvas_text_offset_with_header(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    header_x_offset: f32,
    header_y_offset: f32,
    textbox_width: f32,
) -> usize {
    calculate_canvas_text_offset_with_header_and_width(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        header_x_offset,
        header_y_offset,
        textbox_width,
    )
}

/// Computes the canvas cursor offset using an explicit width and header padding model.
pub(crate) fn calculate_canvas_text_offset_with_header_and_width(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    header_x_offset: f32,
    header_y_offset: f32,
    textbox_width: f32,
) -> usize {
    calculate_canvas_text_offset_with_header_and_bold(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        None,
        header_x_offset,
        header_y_offset,
        textbox_width,
    )
}

/// Computes a canvas character offset while respecting bold formatting metadata.
pub(crate) fn calculate_canvas_text_offset_with_header_and_bold(
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
    textbox_width: f32,
) -> usize {
    calculate_canvas_text_offset_full(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        bold_flags,
        header_x_offset,
        header_y_offset,
        textbox_width,
        CANVAS_BODY_FONT_SIZE,
        FontType::from_family_name(DEFAULT_FONT_FAMILY),
    )
}

/// Calculates the drag target character index with font-size and font-type awareness.
pub(crate) fn calculate_canvas_drag_offset_full(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    bold_flags: Option<&[bool]>,
    textbox_width: f32,
    font_size: f32,
    font_type: FontType,
) -> usize {
    calculate_canvas_text_offset_full(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        bold_flags,
        6.0,
        18.0,
        textbox_width,
        font_size,
        font_type,
    )
}

/// Calculates the drag target character index for a canvas text selection gesture.
#[allow(dead_code)]
pub(crate) fn calculate_canvas_drag_offset(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    _anchor_idx: usize,
    textbox_width: f32,
) -> usize {
    calculate_canvas_drag_offset_full(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        None,
        textbox_width,
        CANVAS_BODY_FONT_SIZE,
        FontType::from_family_name(DEFAULT_FONT_FAMILY),
    )
}

/// Calculates a drag offset for a canvas text box that has explicit header offsets.
#[allow(dead_code)]
pub(crate) fn calculate_canvas_drag_offset_with_header(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    anchor_idx: usize,
    header_x_offset: f32,
    header_y_offset: f32,
    textbox_width: f32,
) -> usize {
    calculate_canvas_drag_offset_with_bold(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        None,
        anchor_idx,
        header_x_offset,
        header_y_offset,
        textbox_width,
    )
}

/// Calculates a drag offset while accounting for bold span metadata.
pub(crate) fn calculate_canvas_drag_offset_with_bold(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    bold_flags: Option<&[bool]>,
    _anchor_idx: usize,
    header_x_offset: f32,
    header_y_offset: f32,
    textbox_width: f32,
) -> usize {
    calculate_canvas_text_offset_full(
        mouse_pos,
        sidebar_open,
        pan_x,
        pan_y,
        item_x,
        item_y,
        canvas_top_y,
        text,
        bold_flags,
        header_x_offset,
        header_y_offset,
        textbox_width,
        CANVAS_BODY_FONT_SIZE,
        FontType::from_family_name(DEFAULT_FONT_FAMILY),
    )
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
        let w_14 = calculate_text_width("Hello", 14.0, 1.0);
        let w_28 = calculate_text_width("Hello", 28.0, 1.0);
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
        let w_normal = calculate_text_width(text, 11.0, 1.00);
        let w_bold = calculate_text_width(text, 11.0, 1.10);
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
        let offset_0 = calculate_line_text_offset_with_font(0.0, text, 12.0, 1.0, FontType::Monospace);
        let offset_2 = calculate_line_text_offset_with_font(w_i * 2.0, text, 12.0, 1.0, FontType::Monospace);
        let offset_4 = calculate_line_text_offset_with_font(w_i * 4.0, text, 12.0, 1.0, FontType::Monospace);
        assert_eq!(offset_0, 0);
        assert_eq!(offset_2, 2);
        assert_eq!(offset_4, 4);
    }

    #[test]
    fn test_font_family_inference_and_offsets() {
        assert_eq!(FontType::from_family_name("Consolas"), FontType::Monospace);
        assert_eq!(FontType::from_family_name("Courier New"), FontType::Monospace);
        assert_eq!(FontType::from_family_name("Arial"), FontType::Arial);
        assert_eq!(FontType::from_family_name("Times New Roman"), FontType::Serif);
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
        assert!((line_height_for_font_size(24.0) - 40.0).abs() < 0.01);

        assert!((cursor_height_for_font_size(12.0) - 16.0).abs() < 0.01);
        assert!((cursor_height_for_font_size(24.0) - 32.0).abs() < 0.01);

        assert!((selection_height_for_font_size(12.0) - 18.0).abs() < 0.01);
        assert!((selection_height_for_font_size(24.0) - 36.0).abs() < 0.01);
    }
}
