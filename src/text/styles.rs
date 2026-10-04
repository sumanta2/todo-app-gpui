//! Bold, italic, underline, and strikethrough flags, spans, and styled runs.

/// Converts bold ranges into a boolean flag array indexed by character position.
pub(crate) fn spans_to_bool_vec(spans: &[(usize, usize)], len: usize) -> Vec<bool> {
    let mut flags = vec![false; len];
    for &(start, end) in spans {
        for i in start..end.min(len) {
            flags[i] = true;
        }
    }
    flags
}

/// Converts a bold-flag vector back into contiguous character ranges.
pub(crate) fn bool_vec_to_spans(flags: &[bool]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut in_span = false;
    let mut span_start = 0;

    for (idx, &is_bold) in flags.iter().enumerate() {
        if is_bold && !in_span {
            in_span = true;
            span_start = idx;
        } else if !is_bold && in_span {
            in_span = false;
            spans.push((span_start, idx));
        }
    }

    if in_span {
        spans.push((span_start, flags.len()));
    }

    spans
}

/// Splits text and its bold-flag vector at a character index into a "before" and "after" half.
///
/// Used when an image is inserted at the cursor position inside a text block: the text before
/// the cursor stays in place, the image is inserted next, and the text after the cursor becomes
/// its own segment that presents right after the image.
pub(crate) fn split_text_and_bold_at(
    text: &str,
    bold_flags: &[bool],
    cursor: usize,
) -> (String, Vec<bool>, String, Vec<bool>) {
    let chars: Vec<char> = text.chars().collect();
    let idx = cursor.min(chars.len());

    let before: String = chars[..idx].iter().collect();
    let after: String = chars[idx..].iter().collect();

    let before_flags = bold_flags
        .get(..idx.min(bold_flags.len()))
        .unwrap_or(&[])
        .to_vec();
    let after_flags = if idx < bold_flags.len() {
        bold_flags[idx..].to_vec()
    } else {
        Vec::new()
    };

    (before, before_flags, after, after_flags)
}

/// A run of text whose bold, italic, underline, strikethrough, font, and size stay the same.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StyledRun {
    pub(crate) text: String,
    pub(crate) is_bold: bool,
    pub(crate) is_italic: bool,
    pub(crate) is_underline: bool,
    pub(crate) is_strike: bool,
    pub(crate) font_family: u8,
    pub(crate) font_size: f32,
    /// `0` means the default body text color.
    pub(crate) font_color: u32,
    /// `0` means no highlight.
    pub(crate) bg_color: u32,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// Copies the flags that fall inside one visual line, padding or trimming to `len` characters.
pub(crate) fn slice_flags(flags: &[bool], start: usize, end: usize, len: usize) -> Vec<bool> {
    let mut sliced = if start < flags.len() {
        flags[start..end.min(flags.len())].to_vec()
    } else {
        Vec::new()
    };
    sliced.resize(len, false);
    sliced
}

/// Drops a character range from each style buffer.
pub(crate) fn drain_style_set(sets: [&mut Vec<bool>; 4], start: usize, end: usize) {
    for flags in sets {
        let start = start.min(flags.len());
        let end = end.min(flags.len());
        if start < end {
            flags.drain(start..end);
        }
    }
}

/// Removes one character's flags from each style buffer.
pub(crate) fn remove_style_set(sets: [&mut Vec<bool>; 4], at: usize) {
    for flags in sets {
        if at < flags.len() {
            flags.remove(at);
        }
    }
}

/// Inserts `count` flags into each style buffer. `values` is bold, italic, underline, strike.
pub(crate) fn insert_style_set(
    sets: [&mut Vec<bool>; 4],
    at: usize,
    count: usize,
    values: [bool; 4],
) {
    for (flags, value) in sets.into_iter().zip(values) {
        for i in 0..count {
            let pos = (at + i).min(flags.len());
            flags.insert(pos, value);
        }
    }
}

fn flag_at(flags: &[bool], idx: usize) -> bool {
    flags.get(idx).copied().unwrap_or(false)
}

fn family_at(families: &[u8], idx: usize) -> u8 {
    families.get(idx).copied().unwrap_or(0)
}

fn size_at(sizes: &[f32], idx: usize) -> f32 {
    sizes
        .get(idx)
        .copied()
        .filter(|size| *size > 0.0)
        .unwrap_or(crate::constants::typography::CANVAS_BODY_FONT_SIZE)
}

fn same_size(left: f32, right: f32) -> bool {
    (left - right).abs() < 0.05
}

/// Inserts the ribbon's current font into each character of a new range.
pub(crate) fn insert_font_values(
    families: &mut Vec<u8>,
    sizes: &mut Vec<f32>,
    at: usize,
    count: usize,
    family: u8,
    size: f32,
) {
    if count == 0 {
        return;
    }
    if families.len() < at {
        families.resize(at, 0);
    }
    if sizes.len() < at {
        sizes.resize(at, crate::constants::typography::CANVAS_BODY_FONT_SIZE);
    }
    let at = at.min(families.len()).min(sizes.len());
    for i in 0..count {
        families.insert(at + i, family);
        sizes.insert(at + i, size);
    }
}

/// Drops font values for a deleted character range.
pub(crate) fn drain_font_values(
    families: &mut Vec<u8>,
    sizes: &mut Vec<f32>,
    start: usize,
    end: usize,
) {
    let start_f = start.min(families.len());
    let end_f = end.min(families.len());
    if start_f < end_f {
        families.drain(start_f..end_f);
    }
    let start_s = start.min(sizes.len());
    let end_s = end.min(sizes.len());
    if start_s < end_s {
        sizes.drain(start_s..end_s);
    }
}

/// Copies color values for one visual line, padding with `0`.
pub(crate) fn slice_colors(values: &[u32], start: usize, end: usize, len: usize) -> Vec<u32> {
    let mut sliced = if start < values.len() {
        values[start..end.min(values.len())].to_vec()
    } else {
        Vec::new()
    };
    sliced.resize(len, 0);
    sliced
}

fn color_at(colors: &[u32], idx: usize) -> u32 {
    colors.get(idx).copied().unwrap_or(0)
}

/// Inserts a text color and highlight into each character of a new range.
pub(crate) fn insert_color_values(
    colors: &mut Vec<u32>,
    backgrounds: &mut Vec<u32>,
    at: usize,
    count: usize,
    color: u32,
    background: u32,
) {
    if count == 0 {
        return;
    }
    if colors.len() < at {
        colors.resize(at, 0);
    }
    if backgrounds.len() < at {
        backgrounds.resize(at, 0);
    }
    let at = at.min(colors.len()).min(backgrounds.len());
    for i in 0..count {
        colors.insert(at + i, color);
        backgrounds.insert(at + i, background);
    }
}

/// Drops color values for a deleted character range.
pub(crate) fn drain_color_values(
    colors: &mut Vec<u32>,
    backgrounds: &mut Vec<u32>,
    start: usize,
    end: usize,
) {
    let start_c = start.min(colors.len());
    let end_c = end.min(colors.len());
    if start_c < end_c {
        colors.drain(start_c..end_c);
    }
    let start_b = start.min(backgrounds.len());
    let end_b = end.min(backgrounds.len());
    if start_b < end_b {
        backgrounds.drain(start_b..end_b);
    }
}

/// Removes the color value for one deleted character.
pub(crate) fn remove_color_value(colors: &mut Vec<u32>, backgrounds: &mut Vec<u32>, at: usize) {
    if at < colors.len() {
        colors.remove(at);
    }
    if at < backgrounds.len() {
        backgrounds.remove(at);
    }
}

/// Color stamped onto newly typed characters.
///
/// A ribbon choice stays in force while the caret has not moved. Otherwise the character
/// before the caret supplies the color.
pub(crate) fn color_for_insert(
    pinned: &mut bool,
    pin_at: &mut Option<usize>,
    typing: u32,
    colors: &[u32],
    cursor: usize,
) -> u32 {
    if *pinned && *pin_at == Some(cursor) {
        typing
    } else {
        *pinned = false;
        *pin_at = None;
        if cursor > 0 {
            colors.get(cursor - 1).copied().unwrap_or(typing)
        } else {
            typing
        }
    }
}

/// Keeps a ribbon color choice attached to the caret after an insert.
pub(crate) fn advance_color_pin(pinned: bool, pin_at: &mut Option<usize>, cursor: usize) {
    if pinned {
        *pin_at = Some(cursor);
    }
}

/// Removes the font value for one deleted character.
pub(crate) fn remove_font_value(families: &mut Vec<u8>, sizes: &mut Vec<f32>, at: usize) {
    if at < families.len() {
        families.remove(at);
    }
    if at < sizes.len() {
        sizes.remove(at);
    }
}

/// Splits parallel font buffers at a character index.
pub(crate) fn split_font_at(
    families: &[u8],
    sizes: &[f32],
    cursor: usize,
) -> (Vec<u8>, Vec<f32>, Vec<u8>, Vec<f32>) {
    let family_idx = cursor.min(families.len());
    let size_idx = cursor.min(sizes.len());
    (
        families[..family_idx].to_vec(),
        sizes[..size_idx].to_vec(),
        families[family_idx..].to_vec(),
        sizes[size_idx..].to_vec(),
    )
}

/// Expands saved font runs into one family index and one pixel size per character.
pub(crate) fn font_runs_to_vecs(
    runs: &[crate::models::FontRun],
    len: usize,
) -> (Vec<u8>, Vec<f32>, Vec<u32>, Vec<u32>) {
    let mut families = vec![0u8; len];
    let mut sizes = vec![crate::constants::typography::CANVAS_BODY_FONT_SIZE; len];
    let mut colors = vec![0u32; len];
    let mut backgrounds = vec![0u32; len];
    for run in runs {
        let family = crate::constants::typography::font_style_index(&run.family);
        for i in run.start..run.end.min(len) {
            families[i] = family;
            sizes[i] = run.size;
            colors[i] = run.color;
            backgrounds[i] = run.background;
        }
    }
    (families, sizes, colors, backgrounds)
}

/// Collapses per-character font values into runs. All-default text is stored as an empty list.
pub(crate) fn font_vecs_to_runs(
    families: &[u8],
    sizes: &[f32],
    colors: &[u32],
    backgrounds: &[u32],
) -> Vec<crate::models::FontRun> {
    let len = families
        .len()
        .max(sizes.len())
        .max(colors.len())
        .max(backgrounds.len());
    if len == 0 {
        return Vec::new();
    }
    let default_size = crate::constants::typography::CANVAS_BODY_FONT_SIZE;
    let mut runs = Vec::new();
    let mut start = 0usize;
    let mut current_family = family_at(families, 0);
    let mut current_size = size_at(sizes, 0);
    let mut current_color = color_at(colors, 0);
    let mut current_bg = color_at(backgrounds, 0);
    for idx in 1..len {
        let family = family_at(families, idx);
        let size = size_at(sizes, idx);
        let color = color_at(colors, idx);
        let background = color_at(backgrounds, idx);
        if family != current_family
            || !same_size(size, current_size)
            || color != current_color
            || background != current_bg
        {
            push_font_run(
                &mut runs,
                start,
                idx,
                current_family,
                current_size,
                current_color,
                current_bg,
                default_size,
            );
            start = idx;
            current_family = family;
            current_size = size;
            current_color = color;
            current_bg = background;
        }
    }
    push_font_run(
        &mut runs,
        start,
        len,
        current_family,
        current_size,
        current_color,
        current_bg,
        default_size,
    );
    runs
}

fn push_font_run(
    runs: &mut Vec<crate::models::FontRun>,
    start: usize,
    end: usize,
    family: u8,
    size: f32,
    color: u32,
    background: u32,
    default_size: f32,
) {
    if start >= end
        || (family == 0 && same_size(size, default_size) && color == 0 && background == 0)
    {
        return;
    }
    runs.push(crate::models::FontRun {
        start,
        end,
        family: crate::constants::typography::font_style_name(family).to_string(),
        size,
        color,
        background,
    });
}

/// Splits text wherever bold, italic, underline, strikethrough, font, or size changes.
pub(crate) fn split_text_into_full_runs(
    text: &str,
    bold: &[bool],
    italic: &[bool],
    underline: &[bool],
    strike: &[bool],
    font_family: &[u8],
    font_size: &[f32],
    font_color: &[u32],
    bg_color: &[u32],
) -> Vec<StyledRun> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut runs = Vec::new();
    let mut current_bold = flag_at(bold, 0);
    let mut current_italic = flag_at(italic, 0);
    let mut current_underline = flag_at(underline, 0);
    let mut current_strike = flag_at(strike, 0);
    let mut current_family = family_at(font_family, 0);
    let mut current_size = size_at(font_size, 0);
    let mut current_color = color_at(font_color, 0);
    let mut current_bg = color_at(bg_color, 0);
    let mut current_chars = Vec::new();
    let mut run_start = 0;

    for (idx, &ch) in chars.iter().enumerate() {
        let is_bold = flag_at(bold, idx);
        let is_italic = flag_at(italic, idx);
        let is_underline = flag_at(underline, idx);
        let is_strike = flag_at(strike, idx);
        let family = family_at(font_family, idx);
        let size = size_at(font_size, idx);
        let color = color_at(font_color, idx);
        let background = color_at(bg_color, idx);
        if is_bold != current_bold
            || is_italic != current_italic
            || is_underline != current_underline
            || is_strike != current_strike
            || family != current_family
            || !same_size(size, current_size)
            || color != current_color
            || background != current_bg
        {
            runs.push(StyledRun {
                text: current_chars.into_iter().collect(),
                is_bold: current_bold,
                is_italic: current_italic,
                is_underline: current_underline,
                is_strike: current_strike,
                font_family: current_family,
                font_size: current_size,
                font_color: current_color,
                bg_color: current_bg,
                start: run_start,
                end: idx,
            });
            current_bold = is_bold;
            current_italic = is_italic;
            current_underline = is_underline;
            current_strike = is_strike;
            current_family = family;
            current_size = size;
            current_color = color;
            current_bg = background;
            current_chars = Vec::new();
            run_start = idx;
        }
        current_chars.push(ch);
    }

    if !current_chars.is_empty() {
        runs.push(StyledRun {
            text: current_chars.into_iter().collect(),
            is_bold: current_bold,
            is_italic: current_italic,
            is_underline: current_underline,
            is_strike: current_strike,
            font_family: current_family,
            font_size: current_size,
            font_color: current_color,
            bg_color: current_bg,
            start: run_start,
            end: chars.len(),
        });
    }

    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_spans_roundtrip() {
        let spans = vec![(2, 5), (8, 11)];
        let flags = spans_to_bool_vec(&spans, 15);
        assert_eq!(
            flags,
            vec![
                false, false, true, true, true, false, false, false, true, true, true, false,
                false, false, false
            ]
        );
        let reconstructed = bool_vec_to_spans(&flags);
        assert_eq!(reconstructed, spans);
    }

    #[test]
    fn test_split_text_into_styled_runs() {
        let text = "Hello world!";
        let mut flags = vec![false; 12];
        // Make "world" bold (indices 6..11)
        for i in 6..11 {
            flags[i] = true;
        }

        let runs = split_text_into_full_runs(text, &flags, &[], &[], &[], &[], &[], &[], &[]);
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "Hello ");
        assert_eq!(runs[0].is_bold, false);
        assert_eq!(runs[1].text, "world");
        assert_eq!(runs[1].is_bold, true);
        assert_eq!(runs[2].text, "!");
        assert_eq!(runs[2].is_bold, false);
    }
}
