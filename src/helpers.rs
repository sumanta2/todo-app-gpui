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
