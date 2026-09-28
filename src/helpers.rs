/// Creates a stable hash for a string so UI elements can use a deterministic ID.
pub(crate) fn hash_str(s: &str) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as usize
}

/// Replaces a character range within a string while preserving UTF-8 character boundaries.
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

/// XOR-encrypts and decrypts byte slices using a repeating key.
///
/// The app uses this helper to store image data with a lightweight obfuscation layer
/// before writing it to disk.
pub(crate) fn encrypt_decrypt(bytes: &[u8], key: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

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

    let before_flags = bold_flags.get(..idx.min(bold_flags.len())).unwrap_or(&[]).to_vec();
    let after_flags = if idx < bold_flags.len() {
        bold_flags[idx..].to_vec()
    } else {
        Vec::new()
    };

    (before, before_flags, after, after_flags)
}

/// A run of text whose bold, italic, underline, and strikethrough state stays the same.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StyledRun {
    pub(crate) text: String,
    pub(crate) is_bold: bool,
    pub(crate) is_italic: bool,
    pub(crate) is_underline: bool,
    pub(crate) is_strike: bool,
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
pub(crate) fn insert_style_set(sets: [&mut Vec<bool>; 4], at: usize, count: usize, values: [bool; 4]) {
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

/// Splits a text string into segments whose bold state is consistent across each run.
pub(crate) fn split_text_into_styled_runs(text: &str, flags: &[bool]) -> Vec<StyledRun> {
    split_text_into_full_runs(text, flags, &[], &[], &[])
}

/// Splits text wherever bold, italic, underline, or strikethrough changes.
pub(crate) fn split_text_into_full_runs(
    text: &str,
    bold: &[bool],
    italic: &[bool],
    underline: &[bool],
    strike: &[bool],
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
    let mut current_chars = Vec::new();
    let mut run_start = 0;

    for (idx, &ch) in chars.iter().enumerate() {
        let is_bold = flag_at(bold, idx);
        let is_italic = flag_at(italic, idx);
        let is_underline = flag_at(underline, idx);
        let is_strike = flag_at(strike, idx);
        if is_bold != current_bold
            || is_italic != current_italic
            || is_underline != current_underline
            || is_strike != current_strike
        {
            runs.push(StyledRun {
                text: current_chars.into_iter().collect(),
                is_bold: current_bold,
                is_italic: current_italic,
                is_underline: current_underline,
                is_strike: current_strike,
                start: run_start,
                end: idx,
            });
            current_bold = is_bold;
            current_italic = is_italic;
            current_underline = is_underline;
            current_strike = is_strike;
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

        let runs = split_text_into_styled_runs(text, &flags);
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "Hello ");
        assert_eq!(runs[0].is_bold, false);
        assert_eq!(runs[1].text, "world");
        assert_eq!(runs[1].is_bold, true);
        assert_eq!(runs[2].text, "!");
        assert_eq!(runs[2].is_bold, false);
    }
}


