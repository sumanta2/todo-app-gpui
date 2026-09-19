/// Converts a cursor/anchor pair into the normalized start-end selection range.
pub(crate) fn get_selection_range(cursor: usize, anchor: Option<usize>) -> Option<(usize, usize)> {
    if let Some(anchor) = anchor {
        if anchor != cursor {
            return Some((cursor.min(anchor), cursor.max(anchor)));
        }
    }
    None
}

/// Estimates the rendered width of a character at a given font size.
///
/// This is used by the editor and canvas selection logic to map pointer positions back to
/// character indices with a simple approximation model.
pub(crate) fn get_char_width(ch: char, font_size: f32) -> f32 {
    let base = match ch {
        // Very narrow characters (~3.15px - 3.25px)
        'i' | 'l' | '\'' | '|' | '!' | '.' | ',' | ';' | ':' | '`' => 3.18,
        'I' | 'j' => 3.28,
        ' ' | '\u{00A0}' => 3.55,

        // Narrow characters (~4.35px - 5.15px)
        't' | 'f' | 'r' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_' | '+' | '*' | '=' => 4.35,
        'J' | 's' | 'z' => 5.15,

        // Lowercase mid-width (~5.60px - 6.00px)
        'c' | 'v' | 'x' | '?' | '/' | '\\' => 5.65,
        'k' => 6.00,

        // Standard lowercase (~6.15px - 6.35px)
        'e' => 6.15,
        'a' => 6.22,
        'b' | 'd' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' | 'y' => 6.35,

        // Standard digits (~6.85px)
        '0'..='9' => 6.85,

        // Narrow uppercase (~6.30px - 7.15px)
        'L' | 'S' => 6.30,
        'F' => 6.50,
        'E' | 'T' | 'Z' => 6.75,
        'P' | 'Y' => 7.15,

        // Standard uppercase (~7.65px - 8.45px)
        'B' | 'K' | 'R' | 'V' | 'X' => 7.65,
        'A' | 'C' => 7.75,
        'G' => 8.05,
        'D' | 'U' => 8.18,
        'H' | 'N' => 8.42,
        'O' | 'Q' => 8.48,

        // Wide characters (~8.95px - 11.00px)
        'w' => 8.95,
        '@' | '%' | '&' | '#' | '$' => 9.10,
        'm' => 9.80,
        'M' => 10.05,
        'W' => 11.00,

        // Default average
        _ => 6.28,
    };
    base * (font_size / 12.0)
}

/// Measures the estimated rendered width of a string at a specific font size and weight multiplier.
pub(crate) fn calculate_text_width(text: &str, font_size: f32, weight_multiplier: f32) -> f32 {
    let mut width = 0.0f32;
    for ch in text.chars() {
        width += get_char_width(ch, font_size) * weight_multiplier;
    }
    width
}

/// Maps an x-position inside a single line back to the nearest character index.
#[allow(dead_code)]
pub(crate) fn calculate_line_text_offset(rel_x: f32, text: &str, font_size: f32) -> usize {
    calculate_line_text_offset_weighted(rel_x, text, font_size, 1.0)
}

/// Maps a click position inside a line to the closest character index with a uniform weight multiplier.
pub(crate) fn calculate_line_text_offset_weighted(
    rel_x: f32,
    text: &str,
    font_size: f32,
    weight_multiplier: f32,
) -> usize {
    let char_count = text.chars().count();
    if text.is_empty() || char_count == 0 || rel_x <= 0.0 {
        return 0;
    }

    let mut current_x = 0.0f32;

    for (idx, ch) in text.chars().enumerate() {
        let w = get_char_width(ch, font_size) * weight_multiplier;
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
pub(crate) fn calculate_line_text_offset_with_bold(
    rel_x: f32,
    text: &str,
    bold_flags: Option<&[bool]>,
    font_size: f32,
) -> usize {
    let char_count = text.chars().count();
    if text.is_empty() || char_count == 0 || rel_x <= 0.0 {
        return 0;
    }

    let mut current_x = 0.0f32;

    for (idx, ch) in text.chars().enumerate() {
        let is_bold = bold_flags.and_then(|f| f.get(idx).copied()).unwrap_or(false);
        let mut w = get_char_width(ch, font_size);
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

/// Computes the character index for a click inside a canvas text block.
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
    _textbox_width: f32,
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

    let line_height = 20.0f32;
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

    let local_char_offset = calculate_line_text_offset_with_bold(rel_x, target_line, line_flags, 12.0);

    (line_start_global + local_char_offset).min(total_char_count)
}

/// Calculates the drag target character index for a canvas text selection gesture.
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
    calculate_canvas_text_offset_with_header_and_bold(
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
        assert_eq!(calculate_line_text_offset_weighted(w_14, text, 14.0, 1.0), 5);
        assert_eq!(calculate_line_text_offset_weighted(w_28, text, 28.0, 1.0), 5);
    }

    #[test]
    fn test_calculate_line_text_offset_with_weight() {
        let text = "Section Title";
        let w_normal = calculate_text_width(text, 11.0, 1.00);
        let w_bold = calculate_text_width(text, 11.0, 1.10);
        assert!(w_bold > w_normal);

        // Clicking beyond the end returns full length
        assert_eq!(calculate_line_text_offset_weighted(999.0, text, 11.0, 1.10), text.len());
    }
}

