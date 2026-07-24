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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct NotePage {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) images: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct NoteSection {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) pages: Vec<NotePage>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct NoteContent {
    pub(crate) sections: Vec<NoteSection>,
}

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub(crate) enum CanvasItem {
    Text(TextItem),
    Image(ImageItem),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct TextItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) text: String,
    pub(crate) width: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct ImageItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) path: String,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

pub(crate) fn load_canvas_items(body: &str) -> Vec<CanvasItem> {
    if let Ok(items) = serde_json::from_str::<Vec<CanvasItem>>(body) {
        items
    } else {
        if body.trim().is_empty() {
            Vec::new()
        } else {
            vec![CanvasItem::Text(TextItem {
                id: "default-text-block".to_string(),
                x: 20.0,
                y: 20.0,
                text: body.to_string(),
                width: Some(500.0),
            })]
        }
    }
}

pub(crate) fn save_canvas_items(items: &[CanvasItem]) -> String {
    serde_json::to_string(items).unwrap_or_default()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ActiveField {
    NoteHeading,
    SectionName,
    Heading,
    Body,
}
