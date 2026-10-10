//! Small helpers that are not specific to text layout: stable ids, string edits, and image bytes.

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

/// XOR-encrypts or decrypts `bytes` in place with a repeating key.
///
/// The same call encrypts and decrypts. Working in the existing buffer avoids a second
/// full-size copy of each image.
pub(crate) fn encrypt_decrypt_in_place(bytes: &mut [u8], key: &[u8]) {
    if key.is_empty() {
        return;
    }
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}
