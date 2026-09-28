# NotesApp Canvas Conversion Flow

This note explains how the app converts a saved note body into canvas items, and why only `Text` and `Image` are kept in the canvas model.

## 1) Top-level note structure

The note model is defined in [src/models/mod.rs](src/models/mod.rs):

```rust
pub(crate) struct Note {
    pub(crate) id: String,
    pub(crate) heading: String,
    pub(crate) body: String,
    pub(crate) created_at: String,
    pub(crate) images: Option<Vec<String>>,
    pub(crate) links: Option<Vec<String>>,
    pub(crate) format: Option<String>,
}
```


The important part is that Note.body is a JSON String, not a direct in-memory canvas object array.


---

## 2) The note body is parsed as notebook content

The helper `load_note_content` in [src/models/mod.rs](src/models/mod.rs) does this:

```rust
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
```

This means the app expects the saved `note.body` to contain nested notebook JSON:

```json
{
  "sections": [
    {
      "id": "...
      "name": "...",
      "pages": [
        {
          "id": "...",
          "name": "...",
          "body": "...",
          "images": []
        }
      ]
    }
  ]
}
```

If the JSON is not in this form, the app falls back to a default section/page wrapper.

---

## 3) Each page `body` is then parsed as canvas items

The canvas layer uses this enum in [src/models/canvas.rs](src/models/canvas.rs):

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub(crate) enum CanvasItem {
    Text(TextItem),
    Image(ImageItem),
}
```

And the actual parser is:

```rust
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
                bold_spans: Vec::new(),
            })]
        }
    }
}
```

This is the real conversion step:

`page.body: String -> Vec<CanvasItem>`

The `page.body` field is expected to be a JSON array of canvas objects, not the full note JSON or arbitrary metadata.

---

## 4) The actual canvas item shape

The text item model is:

```rust
pub(crate) struct TextItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) text: String,
    pub(crate) width: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bold_spans: Vec<(usize, usize)>,
}
```

The image item model is:

```rust
pub(crate) struct ImageItem {
    pub(crate) id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) path: String,
    pub(crate) width: f32,
    pub(crate) height: f32,
}
```

So only the fields that belong to these two models are kept.

---

## 5) Example of saved JSON that matches this model

The sample data shows the shape used by the app. For a page, the `body` is a JSON array like:

```json
[
  {
    "type": "Text",
    "id": "1784486189041",
    "x": 9.0,
    "y": 58.800003,
    "text": "Hello How are You...",
    "width": 706.0,
    "bold_spans": [[68, 99]]
  },
  {
    "type": "Image",
    "id": "1784486313374",
    "x": 11.999985,
    "y": 127.000015,
    "path": "C:\\Users\\gorai\\AppData\\Roaming\\todo-app-gpui\\images\\1784484680064_1784486313354.bmp",
    "width": 490.0,
    "height": 361.0
  }
]
```

This is exactly what `serde_json` can deserialize into `Vec<CanvasItem>`.

---

## 6) Where the app loads and saves this data

### Load flow

In [src/app/editing.rs](src/app/editing.rs):

```rust
if let Some(page) = section.pages.iter().find(|p| p.id == page_id) {
    self.edit_canvas_items = load_canvas_items(&page.body);
    self.edit_images = page.images.clone().unwrap_or_default();
}
```

This is the moment when the page JSON is converted to canvas items.

### Save flow

Also in [src/app/editing.rs](src/app/editing.rs):

```rust
self.edit_canvas_items.retain(|item| match item {
    CanvasItem::Text(t) => !t.text.trim().is_empty(),
    CanvasItem::Image(_) => true,
});

page.body = save_canvas_items(&self.edit_canvas_items);
```

And `save_canvas_items` is:

```rust
pub(crate) fn save_canvas_items(items: &[CanvasItem]) -> String {
    serde_json::to_string(items).unwrap_or_default()
}
```

So the current canvas item array is serialized back to JSON and saved into `page.body`.

---

## 7) Why extra JSON fields are lost

The app only knows how to deserialize into the types declared in [src/models/canvas.rs](src/models/canvas.rs):

- `TextItem`
- `ImageItem`

If your raw JSON contains additional metadata that is not represented by those structs, it is ignored during deserialization unless the model is expanded.

Example:

```json
{
  "type": "Text",
  "id": "123",
  "x": 10,
  "y": 20,
  "text": "abc",
  "width": 300,
  "bold_spans": [[0, 2]],
  "custom_meta": {
    "author": "user",
    "category": "notes"
  }
}
```

`custom_meta` will not be loaded unless you add a matching field to `TextItem` or a wrapper struct.

---

## 8) The practical conclusion

The design is intentionally narrowed:

- `Note.body` stores a notebook document structure
- `Page.body` stores a JSON array of `CanvasItem`
- `CanvasItem` only supports `Text` and `Image`
- any richer metadata must either be:
  - preserved in the outer note/page structure,
  - added to the `TextItem` / `ImageItem` structs,
  - or converted into a new item type and enum variant

This is why the app can display canvas text and images cleanly, but only with data explicitly defined in the `CanvasItem` model.

---

## 9) Key files involved

- [src/models/mod.rs](src/models/mod.rs)
- [src/models/canvas.rs](src/models/canvas.rs)
- [src/app/editing.rs](src/app/editing.rs)
- [src/views/detail_pane.rs](src/views/detail_pane.rs)
- [CleanedNotesJsonSample.json](CleanedNotesJsonSample.json)
- [OriginalNotesJsonSample.json](OriginalNotesJsonSample.json)

This is the actual conversion boundary in the project.
