//! `notes.json` persistence and the encrypted image files stored beside it.

use gpui::Context;
use std::fs;
use std::path::PathBuf;

use crate::app::NotesApp;
use crate::helpers::encrypt_decrypt;
use crate::text::styles::split_text_and_bold_at;
use crate::models::{ActiveField, CanvasItem, ContentBlock, ImageItem, MixedItem, Note};

/// Persistent storage helpers for the app's notes and image files.
///
/// The application keeps all user data under a single app-specific folder:
/// - Windows: `%APPDATA%/todo-app-gpui/notes.json`  or C:\Users\gorai\AppData\Roaming\todo-app-gpui\notes.json
/// - Unix-like systems: `$HOME/todo-app-gpui/notes.json`
///
/// Notes are stored as a JSON file, while attached images are saved as separate files in
/// a sibling `images` folder. The image bytes are encrypted before being written to disk,
/// and decrypted again when they are loaded back into the editor.
impl NotesApp {
    /// Builds the location of the app's JSON storage file in the user data directory.
    ///
    /// The function tries to place data in the OS user data area, preferring the platform
    /// standard directory (`APPDATA` on Windows, then `HOME`/`USERPROFILE` on Unix-like
    /// systems). If no environment variable is available, it falls back to the current working
    /// directory and still keeps the same folder layout.
    pub(crate) fn get_storage_path() -> PathBuf {
        let mut path = if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile)
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        path.push("todo-app-gpui");
        path.push("notes.json");
        path
    }

    /// Loads the full note list from disk.
    ///
    /// This reads the JSON file created by `save_notes()` and converts it back into a
    /// `Vec<Note>`. If the file does not exist or the content is invalid JSON, the function
    /// returns `None` so the app can continue with an empty/default state.
    pub(crate) fn load_notes() -> Option<Vec<Note>> {
        let path = Self::get_storage_path();
        if path.exists() {
            let content = fs::read_to_string(path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    /// Saves the current in-memory notes to the storage file.
    ///
    /// The data is written as pretty-printed JSON so it remains human-readable and easy to
    /// inspect manually if needed. The parent directory is created automatically before writing.
    pub(crate) fn save_notes(&self) {
        let path = Self::get_storage_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&self.notes) {
            let _ = fs::write(path, content);
        }
    }

    /// Attaches an image to the currently selected note and persists it to disk.
    ///
    /// The function:
    /// 1. determines the file extension from the image format,
    /// 2. creates the app-level `images` folder beside `notes.json`,
    /// 3. writes the image under a unique filename based on the note ID and timestamp,
    /// 4. encrypts the raw bytes before storing them,
    /// 5. pushes a corresponding `ImageItem` into the canvas state so it can be rendered later.
    pub(crate) fn attach_image_to_note(&mut self, img: &gpui::Image, cx: &mut Context<Self>) {
        if let Some(ref selected_id) = self.selected_note_id {
            let ext = match img.format {
                gpui::ImageFormat::Png => "png",
                gpui::ImageFormat::Jpeg => "jpg",
                gpui::ImageFormat::Webp => "webp",
                gpui::ImageFormat::Gif => "gif",
                gpui::ImageFormat::Bmp => "bmp",
                _ => "png",
            };
            let mut path = Self::get_storage_path().parent().unwrap().to_path_buf();
            path.push("images");
            let _ = fs::create_dir_all(&path);
            let timestamp = chrono::Local::now().timestamp_millis();
            let filename = format!("{}_{}.{}", selected_id, timestamp, ext);
            path.push(filename);

            // Scramble bytes using XOR cipher before writing to disk.
            // The selected note ID is part of the key so each note has its own encryption scope.
            let key = format!("notes-security-key-{}", selected_id).into_bytes();
            let encrypted_bytes = encrypt_decrypt(&img.bytes, &key);

            if fs::write(&path, &encrypted_bytes).is_ok() {
                if let Some(path_str) = path.to_str() {
                    let path_str = path_str.to_owned();
                    self.edit_images.push(path_str.clone());

                    let inserted_inline = self.insert_image_into_active_text(path_str.clone());
                    if !inserted_inline {
                        let new_id = chrono::Local::now().timestamp_millis().to_string();
                        let new_image_item = ImageItem {
                            id: new_id,
                            x: -self.pan_x + 50.0,
                            y: -self.pan_y + 50.0,
                            path: path_str,
                            width: 240.0,
                            height: 180.0,
                        };
                        self.edit_canvas_items
                            .push(CanvasItem::Image(new_image_item));
                    }
                    cx.notify();
                }
            }
        }
    }

    /// Splits the currently focused text block at the cursor and inserts a new image between
    /// the two halves, so the remaining text presents right after the image in the same box.
    ///
    /// Returns `false` when there is no active text block to insert into (for example, when
    /// nothing is focused), in which case the caller should fall back to a standalone image box.
    fn insert_image_into_active_text(&mut self, image_path: String) -> bool {
        if self.active_field != ActiveField::Body {
            return false;
        }
        let Some(active_id) = self.active_text_block_id.clone() else {
            return false;
        };

        let cursor = self.edit_body_cursor;
        let (before_text, before_bold, after_text, after_bold) =
            split_text_and_bold_at(&self.edit_body, &self.edit_body_bold, cursor);
        let (_, before_italic, _, after_italic) =
            split_text_and_bold_at(&self.edit_body, &self.edit_body_italic, cursor);
        let (_, before_underline, _, after_underline) =
            split_text_and_bold_at(&self.edit_body, &self.edit_body_underline, cursor);
        let (_, before_strike, _, after_strike) =
            split_text_and_bold_at(&self.edit_body, &self.edit_body_strike, cursor);

        let mut replacement = Vec::new();
        if !before_text.is_empty() {
            replacement.push(ContentBlock::Text {
                text: before_text,
                bold_spans: crate::text::styles::bool_vec_to_spans(&before_bold),
                italic_spans: crate::text::styles::bool_vec_to_spans(&before_italic),
                underline_spans: crate::text::styles::bool_vec_to_spans(&before_underline),
                strike_spans: crate::text::styles::bool_vec_to_spans(&before_strike),
            });
        }
        replacement.push(ContentBlock::Image {
            path: image_path,
            width: 240.0,
            height: 180.0,
        });
        let after_pos = replacement.len();
        replacement.push(ContentBlock::Text {
            text: after_text.clone(),
            bold_spans: crate::text::styles::bool_vec_to_spans(&after_bold),
            italic_spans: crate::text::styles::bool_vec_to_spans(&after_italic),
            underline_spans: crate::text::styles::bool_vec_to_spans(&after_underline),
            strike_spans: crate::text::styles::bool_vec_to_spans(&after_strike),
        });

        let Some(item) = self
            .edit_canvas_items
            .iter_mut()
            .find(|i| match i {
                CanvasItem::Text(t) => t.id == active_id,
                CanvasItem::Mixed(m) => m.id == active_id,
                CanvasItem::Image(_) => false,
            })
        else {
            return false;
        };

        let new_active_block_index = match item {
            CanvasItem::Text(t) => {
                let mixed = MixedItem {
                    id: t.id.clone(),
                    x: t.x,
                    y: t.y,
                    width: t.width,
                    blocks: replacement,
                };
                *item = CanvasItem::Mixed(mixed);
                after_pos
            }
            CanvasItem::Mixed(m) => {
                let idx = self.active_block_index.unwrap_or(0).min(m.blocks.len());
                let end = (idx + 1).min(m.blocks.len());
                m.blocks.splice(idx..end, replacement);
                idx + after_pos
            }
            CanvasItem::Image(_) => return false,
        };

        self.active_block_index = Some(new_active_block_index);
        self.edit_body = after_text;
        self.edit_body_bold = after_bold;
        self.edit_body_italic = after_italic;
        self.edit_body_underline = after_underline;
        self.edit_body_strike = after_strike;
        self.edit_body_cursor = 0;
        self.edit_body_anchor = None;
        true
    }

    /// Loads an image from disk and decrypts it back into a `gpui::Image`.
    ///
    /// The function reads the file from the stored path, uses the same note-based key to
    /// decrypt the bytes, infers the image format from the extension, and creates a valid
    /// image object for the editor. Returns `None` if the file cannot be read or if the
    /// selected note ID is not available.
    pub(crate) fn decrypt_image(&self, path_str: &str) -> Option<gpui::Image> {
        let path = PathBuf::from(path_str);
        if let Some(ref selected_id) = self.selected_note_id {
            if let Ok(encrypted_bytes) = fs::read(&path) {
                let key = format!("notes-security-key-{}", selected_id).into_bytes();
                let decrypted_bytes = encrypt_decrypt(&encrypted_bytes, &key);

                let ext = path.extension()?.to_str()?.to_lowercase();
                let format = match ext.as_str() {
                    "jpg" | "jpeg" => gpui::ImageFormat::Jpeg,
                    "webp" => gpui::ImageFormat::Webp,
                    "gif" => gpui::ImageFormat::Gif,
                    "bmp" => gpui::ImageFormat::Bmp,
                    _ => gpui::ImageFormat::Png,
                };

                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                path_str.hash(&mut hasher);
                let id = hasher.finish();

                return Some(gpui::Image {
                    format,
                    bytes: decrypted_bytes,
                    id,
                });
            }
        }
        None
    }
}
