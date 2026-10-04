//! Items drawn on a page: text blocks, images, and mixed image-and-text boxes.

/// A single item drawn on the canvas editor surface.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub(crate) enum CanvasItem {
    Text(TextItem),
    Image(ImageItem),
    /// A single box that contains an image followed by text, so the text presents right
    /// after the image inside the same visual block instead of two separate floating boxes.
    Mixed(MixedItem),
}

/// Editable text block on the canvas, including its position and bold formatting spans.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct TextItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) text: String,
    pub(crate) width: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bold_spans: Vec<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) italic_spans: Vec<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) underline_spans: Vec<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) strike_spans: Vec<(usize, usize)>,
    /// Runs whose family or size differs from Calibri 11pt. Empty means the whole block uses that default.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) font_runs: Vec<FontRun>,
    /// Indent and alignment for each visual line. Missing lines use the default.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) line_layouts: Vec<LineLayout>,
}

/// A contiguous character range that shares one font family and size.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct FontRun {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) family: String,
    pub(crate) size: f32,
    /// Text color. `0` means the default body color.
    #[serde(default)]
    pub(crate) color: u32,
    /// Highlight color. `0` means no background.
    #[serde(default)]
    pub(crate) background: u32,
}

/// The four character-style span lists that belong to one text segment.
#[derive(Clone, Debug, Default)]
pub(crate) struct TextStyleSpans {
    pub(crate) bold: Vec<(usize, usize)>,
    pub(crate) italic: Vec<(usize, usize)>,
    pub(crate) underline: Vec<(usize, usize)>,
    pub(crate) strike: Vec<(usize, usize)>,
    pub(crate) font_runs: Vec<FontRun>,
    pub(crate) line_layouts: Vec<LineLayout>,
}

/// Indent level and horizontal alignment of one visual line.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LineLayout {
    #[serde(default)]
    pub(crate) indent: u8,
    /// `0` left, `1` center, `2` right.
    #[serde(default)]
    pub(crate) align: u8,
}

/// Image block stored on the canvas with its dimensions and disk path.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct ImageItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) path: String,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

/// One piece of content inside a `MixedItem`, stacked top to bottom in order.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "kind")]
pub(crate) enum ContentBlock {
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bold_spans: Vec<(usize, usize)>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        italic_spans: Vec<(usize, usize)>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        underline_spans: Vec<(usize, usize)>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        strike_spans: Vec<(usize, usize)>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        font_runs: Vec<FontRun>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        line_layouts: Vec<LineLayout>,
    },
    Image {
        path: String,
        width: f32,
        height: f32,
        /// Distance from the left edge of the text box. `0` sits the image flush left.
        #[serde(default)]
        offset_x: f32,
    },
}

/// A single combined box that stacks an image and one or more text segments, so the text
/// typed after inserting an image presents right below/after that image in the same box.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct MixedItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: Option<f32>,
    pub(crate) blocks: Vec<ContentBlock>,
}

/// Parses the JSON payload for canvas items and falls back to a single text block when needed.
pub(crate) fn load_canvas_items(body: &str) -> Vec<CanvasItem> {
    if let Ok(items) = serde_json::from_str::<Vec<CanvasItem>>(body) {
        merge_overlapping_text_and_image(items)
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
                bold_spans: Vec::new(),
                italic_spans: Vec::new(),
                underline_spans: Vec::new(),
                strike_spans: Vec::new(),
                font_runs: Vec::new(),
                line_layouts: Vec::new(),
            })]
        }
    }
}

/// Estimates a text block's on-screen height so overlap can be checked against images.
fn estimated_text_height(text: &str, font_size: f32) -> f32 {
    let line_count = text.split('\n').count().max(1) as f32;
    line_count * crate::constants::typography::line_height_for_font_size(font_size) + 10.0
}

fn rects_overlap(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

/// Auto-merges legacy Text/Image pairs that visually overlap into a single `Mixed` box
/// (image first, then text), so older notes gain the combined layout too.
fn merge_overlapping_text_and_image(items: Vec<CanvasItem>) -> Vec<CanvasItem> {
    let font_size = crate::constants::typography::CANVAS_BODY_FONT_SIZE;
    let mut images: Vec<ImageItem> = Vec::new();
    let mut texts: Vec<TextItem> = Vec::new();
    let mut others: Vec<CanvasItem> = Vec::new();

    for item in items {
        match item {
            CanvasItem::Image(i) => images.push(i),
            CanvasItem::Text(t) => texts.push(t),
            other => others.push(other),
        }
    }

    let mut used_text = vec![false; texts.len()];
    let mut result: Vec<CanvasItem> = Vec::new();

    for image in images {
        let img_w = image.width;
        let img_h = image.height;
        let match_idx = texts.iter().enumerate().position(|(idx, t)| {
            if used_text[idx] {
                return false;
            }
            let text_w = t.width.unwrap_or(250.0);
            let text_h = estimated_text_height(&t.text, font_size);
            rects_overlap(image.x, image.y, img_w, img_h, t.x, t.y, text_w, text_h)
        });

        if let Some(idx) = match_idx {
            used_text[idx] = true;
            let t = &texts[idx];
            result.push(CanvasItem::Mixed(MixedItem {
                id: image.id.clone(),
                x: image.x.min(t.x),
                y: image.y.min(t.y),
                width: t.width,
                blocks: vec![
                    ContentBlock::Image {
                        path: image.path,
                        width: img_w,
                        height: img_h,
                        offset_x: 0.0,
                    },
                    ContentBlock::Text {
                        text: t.text.clone(),
                        bold_spans: t.bold_spans.clone(),
                        italic_spans: t.italic_spans.clone(),
                        underline_spans: t.underline_spans.clone(),
                        strike_spans: t.strike_spans.clone(),
                        font_runs: t.font_runs.clone(),
                        line_layouts: t.line_layouts.clone(),
                    },
                ],
            }));
        } else {
            result.push(CanvasItem::Image(image));
        }
    }

    for (idx, t) in texts.into_iter().enumerate() {
        if !used_text[idx] {
            result.push(CanvasItem::Text(t));
        }
    }

    result.extend(others);
    result
}

/// Serializes the active canvas items back into the note page JSON body.
pub(crate) fn save_canvas_items(items: &[CanvasItem]) -> String {
    serde_json::to_string(items).unwrap_or_default()
}
