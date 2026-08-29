pub(crate) fn get_selection_range(cursor: usize, anchor: Option<usize>) -> Option<(usize, usize)> {
    if let Some(anchor) = anchor {
        if anchor != cursor {
            return Some((cursor.min(anchor), cursor.max(anchor)));
        }
    }
    None
}

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

pub(crate) fn calculate_line_text_offset(rel_x: f32, text: &str, font_size: f32) -> usize {
    let char_count = text.chars().count();
    if text.is_empty() || char_count == 0 || rel_x <= 0.0 {
        return 0;
    }

    let mut current_x = 0.0f32;

    for (idx, ch) in text.chars().enumerate() {
        let w = get_char_width(ch, font_size);
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
    let line_char_count = target_line.chars().count();

    let local_char_offset = if rel_x <= 0.0 {
        0
    } else {
        let mut current_x = 0.0f32;
        let mut found = None;

        for (idx, ch) in target_line.chars().enumerate() {
            let w = get_char_width(ch, 12.0);
            let midpoint = current_x + (w / 2.0);
            let end_x = current_x + w;

            if rel_x < midpoint {
                found = Some(idx);
                break;
            } else if rel_x < end_x {
                found = Some(idx + 1);
                break;
            }
            current_x = end_x;
        }

        found.unwrap_or(line_char_count)
    };

    let mut global_idx = 0;
    for i in 0..target_idx {
        global_idx += logical_lines[i].chars().count() + 1;
    }
    global_idx += local_char_offset;

    global_idx.min(total_char_count)
}

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

pub(crate) fn calculate_canvas_drag_offset_with_header(
    mouse_pos: gpui::Point<gpui::Pixels>,
    sidebar_open: bool,
    pan_x: f32,
    pan_y: f32,
    item_x: f32,
    item_y: f32,
    canvas_top_y: f32,
    text: &str,
    _anchor_idx: usize,
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
