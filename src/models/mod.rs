//! Notebook document: notes, sections, pages, and which field has keyboard focus.
//!
//! Canvas block types live in `canvas` and are re-exported here.

mod canvas;

pub(crate) use canvas::{
    load_canvas_items, save_canvas_items, CanvasItem, ContentBlock, FontRun, ImageItem, LineLayout,
    MixedItem, TextItem, TextStyleSpans,
};

/// A single saved notebook item persisted to the app's JSON store.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct Note {
    pub(crate) id: String,
    pub(crate) heading: String,
    pub(crate) body: String,
    pub(crate) created_at: String,
    // Future-proof slots for screenshots, links, and markdown formatting
    pub(crate) images: Option<Vec<String>>,
    pub(crate) links: Option<Vec<String>>,
    pub(crate) format: Option<String>,
}

/// A single page within a section of a notebook document.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct NotePage {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) images: Option<Vec<String>>,
}

/// A logical section inside a note, containing one or more pages.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct NoteSection {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) pages: Vec<NotePage>,
}

/// The complete nested data model used by a note while editing.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct NoteContent {
    pub(crate) sections: Vec<NoteSection>,
}

/// Rehydrates the notebook structure from a serialized note body.
///
/// Older notes may still contain a flat body string, so this helper wraps them in a
/// default section/page structure instead of failing when the new layout is absent.
/// //
pub(crate) fn load_note_content(body: &str, legacy_images: &Option<Vec<String>>) -> NoteContent {
    if let Ok(content) = serde_json::from_str::<NoteContent>(body) {
        content
    } else {
        NoteContent {
            sections: vec![NoteSection {
                id: "default-section".to_string(),
                name: "Section 1".to_string(),
                pages: vec![NotePage {
                    id: "default-page".to_string(),
                    name: "Page 1".to_string(),
                    body: body.to_string(),
                    images: legacy_images.clone(),
                }],
            }],
        }
    }
}

/// Which UI field currently owns keyboard focus during editing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ActiveField {
    NoteHeading,
    SectionName,
    Heading,
    Body,
    Search,
}
