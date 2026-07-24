pub(crate) fn hash_str(s: &str) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as usize
}

pub(crate) fn get_selection_range(cursor: usize, anchor: Option<usize>) -> Option<(usize, usize)> {
    if let Some(anchor) = anchor {
        if anchor != cursor {
            return Some((cursor.min(anchor), cursor.max(anchor)));
        }
    }
    None
}

pub(crate) fn replace_range(s: &mut String, start: usize, end: usize, replace_with: &str) {
    let mut chars: Vec<char> = s.chars().collect();
    if start <= end && end <= chars.len() {
        chars.drain(start..end);
        for (idx, ch) in replace_with.chars().enumerate() {
            chars.insert(start + idx, ch);
        }
        *s = chars.into_iter().collect();
    }
}

pub(crate) fn encrypt_decrypt(bytes: &[u8], key: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

fn get_char_width(ch: char) -> f32 {
    match ch {
        'i' | 'l' | '\'' | '|' | '!' | '.' | ',' | ';' | ':' | '`' => 3.2,
        't' | 'f' | 'j' | 'r' | 'I' | ' ' | '\u{00A0}' | '(' | ')' | '[' | ']' | '{' | '}' => 4.2,
        's' | 'z' | 'c' | 'v' | 'x' | 'J' | '-' => 5.4,
        'a' | 'b' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 'u' | 'y' | '0'..='9' => {
            6.2
        }
        'A' | 'B' | 'C' | 'E' | 'F' | 'G' | 'H' | 'K' | 'L' | 'N' | 'O' | 'P' | 'R' | 'S' | 'T'
        | 'U' | 'V' | 'X' | 'Y' | 'Z' => 7.5,
        'm' | 'w' | 'M' | 'W' | '@' | '%' | '#' | '&' => 9.2,
        _ => 6.2,
    }
}

pub(crate) fn calculate_line_text_offset(rel_x: f32, text: &str) -> usize {
    let char_count = text.chars().count();
    if text.is_empty() || char_count == 0 || rel_x <= 0.0 {
        return 0;
    }

    let mut current_x = 0.0f32;
    let mut selected_offset = char_count;

    for (idx, ch) in text.chars().enumerate() {
        let w = get_char_width(ch);
        let midpoint = current_x + (w / 2.0);
        let end_x = current_x + w;

        if rel_x < midpoint {
            selected_offset = idx;
            break;
        } else if rel_x < end_x {
            selected_offset = idx + 1;
            break;
        }
        current_x = end_x;
    }

    if rel_x >= current_x {
        char_count
    } else {
        selected_offset.min(char_count)
    }
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
        7.0,
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
        let mut selected_offset = line_char_count;

        for (idx, ch) in target_line.chars().enumerate() {
            let w = get_char_width(ch);
            let midpoint = current_x + (w / 2.0);
            let end_x = current_x + w;

            if rel_x < midpoint {
                selected_offset = idx;
                break;
            } else if rel_x < end_x {
                selected_offset = idx + 1;
                break;
            }
            current_x = end_x;
        }

        if rel_x >= current_x {
            line_char_count
        } else {
            selected_offset.min(line_char_count)
        }
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
        7.0,
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
