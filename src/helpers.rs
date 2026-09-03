pub(crate) fn hash_str(s: &str) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as usize
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

pub(crate) fn spans_to_bool_vec(spans: &[(usize, usize)], len: usize) -> Vec<bool> {
    let mut flags = vec![false; len];
    for &(start, end) in spans {
        for i in start..end.min(len) {
            flags[i] = true;
        }
    }
    flags
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StyledRun {
    pub(crate) text: String,
    pub(crate) is_bold: bool,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn split_text_into_styled_runs(text: &str, flags: &[bool]) -> Vec<StyledRun> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut runs = Vec::new();
    let mut current_bold = flags.first().copied().unwrap_or(false);
    let mut current_chars = Vec::new();
    let mut run_start = 0;

    for (idx, &ch) in chars.iter().enumerate() {
        let is_bold = flags.get(idx).copied().unwrap_or(false);
        if is_bold != current_bold {
            runs.push(StyledRun {
                text: current_chars.into_iter().collect(),
                is_bold: current_bold,
                start: run_start,
                end: idx,
            });
            current_bold = is_bold;
            current_chars = Vec::new();
            run_start = idx;
        }
        current_chars.push(ch);
    }

    if !current_chars.is_empty() {
        runs.push(StyledRun {
            text: current_chars.into_iter().collect(),
            is_bold: current_bold,
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


