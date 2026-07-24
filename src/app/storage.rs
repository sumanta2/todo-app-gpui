use gpui::Context;
use std::fs;
use std::path::PathBuf;

use crate::app::NotesApp;
use crate::helpers::encrypt_decrypt;
use crate::models::{CanvasItem, ImageItem, Note};

impl NotesApp {
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

    pub(crate) fn load_notes() -> Option<Vec<Note>> {
        let path = Self::get_storage_path();
        if path.exists() {
            let content = fs::read_to_string(path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    pub(crate) fn save_notes(&self) {
        let path = Self::get_storage_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&self.notes) {
            let _ = fs::write(path, content);
        }
    }

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

            // Scramble bytes using XOR cipher before writing to disk
            let key = format!("notes-security-key-{}", selected_id).into_bytes();
            let encrypted_bytes = encrypt_decrypt(&img.bytes, &key);

            if fs::write(&path, &encrypted_bytes).is_ok() {
                if let Some(path_str) = path.to_str() {
                    let new_id = chrono::Local::now().timestamp_millis().to_string();
                    let new_image_item = ImageItem {
                        id: new_id,
                        x: -self.pan_x + 50.0,
                        y: -self.pan_y + 50.0,
                        path: path_str.to_owned(),
                        width: 240.0,
                        height: 180.0,
                    };
                    self.edit_canvas_items
                        .push(CanvasItem::Image(new_image_item));
                    // Keep self.edit_images in sync as well
                    self.edit_images.push(path_str.to_owned());
                    cx.notify();
                }
            }
        }
    }

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
