# Undo and redo

Undo and redo apply to the notebook that is open in the editor. The stacks live on `NotesApp.history` for that edit session. They are not written to `notes.json`.

Read this before changing notebook content, the snapshot, or the buttons. The code is split on purpose:

| File | Job |
| --- | --- |
| [src/app/history/kind.rs](src/app/history/kind.rs) | `EditKind` and which edits merge into one step |
| [src/app/history/snapshot.rs](src/app/history/snapshot.rs) | Copy the open notebook, then put that copy back |
| [src/app/history/mod.rs](src/app/history/mod.rs) | Undo and redo stacks, `record_edit`, `undo`, `redo` |
| [src/app/history/tests.rs](src/app/history/tests.rs) | Regression tests. They do not open a window or write `notes.json` |

`NotesApp` in [src/app/mod.rs](src/app/mod.rs) owns the `history` field. [src/app/editing.rs](src/app/editing.rs) clears it when an edit session starts. [src/app/notebook.rs](src/app/notebook.rs) clears it when the open note is deleted.

## What the user sees

- Title bar, top left: curved arrow icons. A dark arrow can be clicked. A gray arrow cannot.
- Home ribbon: the same Undo and Redo icons beside the clipboard buttons.
- Ctrl+Z undoes. Ctrl+Y and Ctrl+Shift+Z redo.
- `can_undo` and `can_redo` are true only while editing and that stack has a step.

Each step is one snapshot of the document as it was *before* the change. Undo swaps the live document with the previous snapshot and keeps the live document on the redo stack. Redo does the opposite. After either one, the app marks the note dirty and uses the existing autosave.

The stack keeps the newest 100 steps. Older steps are dropped.

## What one snapshot holds

`capture` in `snapshot.rs` flushes the active text box with `sync_active_text_block`, then copies:

- Note id and notebook name, plus the caret in that name
- Every section and page (`NoteContent`). The open page's name, body, and image paths are written from the live editor, so the copy matches what is on screen
- Section name, page name, and their carets
- Focused field, active text box, and caret
- Body text and its bold, italic, underline, strikethrough, font, color, highlight, and line layout
- Canvas items (text boxes, images, mixed boxes) and their positions and sizes
- Which inline image is selected

The caret is restored, but it is not part of "did the document change?". Two snapshots match when the note id, notebook name, section/page content, and canvas items match. A click that only moves the caret is not a step.

Zoom, pan, search text, menus, sidebar width, and the ribbon's "next character" font, size, and color are not in the snapshot. Choosing a font or color with nothing selected changes what the next typed characters look like. Those characters are undone with the typing step. The ribbon choice itself is not its own step.

Image files stay on disk. Undo puts the canvas path back. Do not delete the file inside undo or redo, or a later redo cannot show the image.

## How a step is recorded

Call `record_edit` *before* the mutation, then change the document.

```rust
self.record_edit(crate::app::history::EditKind::Format);
// then change the text, canvas, or names
```

`record_edit` does nothing when the note is not being edited, and nothing while a snapshot is being restored (`history.restoring`). Do not call it from `undo`, `redo`, or `snapshot::restore`.

On a new step it:

1. Copies the document as it is now onto the undo stack.
2. Clears redo. The previous redo entries are held only until this step is known to be real.
3. Remembers the `EditKind` and the time.

Typing in the same field, within one second, does not copy again. The snapshot already on the stack is the text from before the burst, and the live editor has the latest characters. Undo restores the whole burst in one step. The first keystroke that opens a box starts with no block id. The following keys in that same box still merge with it.

These start a new step: a pause longer than one second, a different field, a different text box, paste, formatting, an image insert, deleting a box, or a canvas drag.

## Canvas drag, resize, and zoom

A gesture is one step for the whole mouse drag.

1. On mouse down, call `begin_canvas_gesture`. That records `EditKind::CanvasGesture` before the item moves.
2. Mouse move updates the item. Do not call `record_edit` again.
3. On mouse up, call `end_canvas_gesture`. If the position and size match the snapshot, the checkpoint is dropped and redo is put back. If they differ, the step stays and the note is saved.

`end_canvas_gesture` must read the drag flags *before* they are cleared. [src/canvas/editor.rs](src/canvas/editor.rs) does that.

`discard_unchanged_edit` is the same check without saving. Use it after a command that might change nothing, such as a failed image write. If the document matches the snapshot just pushed, that checkpoint is removed and redo is restored. If the document changed, the checkpoint stays and redo stays empty.

## Where `record_edit` is called today

| Change | Kind | File |
| --- | --- | --- |
| Typing and backspace in the notebook name, section name, page name, or canvas body | `EditKind::typing` | [src/app/keyboard.rs](src/app/keyboard.rs) |
| Cut, paste, and the Delete command | `Clipboard` | [src/app/clipboard.rs](src/app/clipboard.rs) |
| Bold, italic, underline, strikethrough, and painting a text or highlight color on a selection | `Format` | [src/app/formatting.rs](src/app/formatting.rs) |
| Indent and alignment, including nudging a selected image | `Format` | [src/app/paragraph.rs](src/app/paragraph.rs) |
| Inserting an image | `Image` | [src/app/storage.rs](src/app/storage.rs) |
| Deleting a text box, image, or mixed box | `Canvas` | [src/canvas/blocks.rs](src/canvas/blocks.rs) |
| Move, resize, or inline-image move and zoom | `CanvasGesture` | [src/canvas/blocks.rs](src/canvas/blocks.rs) and [src/canvas/editor.rs](src/canvas/editor.rs) |
| Add or delete a section or page, and only when that change will actually happen | `Structure` | [src/app/outline.rs](src/app/outline.rs) |

Keyboard shortcuts are handled in `keyboard.rs` before the key is treated as text.

## Adding a new document edit

1. Pick an existing `EditKind`, or add a variant in `kind.rs`. Only `Typing` merges. A new kind is its own step unless you teach `merges_with` otherwise.
2. Call `record_edit` before the first write to notebook content. If the command can no-op, call it only when the change will happen, or call `discard_unchanged_edit` afterward.
3. If the new data is part of the document and is not already inside `NoteContent` or the canvas items, add it to `EditSnapshot` in `snapshot.rs`. Set it in `capture` and put it back in `restore`. A field that is only on `NotesApp` will be lost on undo until both sides are updated.
4. Do not record zoom, pan, selection, menus, or search.
5. Add a test in `tests.rs` that changes the value, undoes, and checks the old value. Redo should put the new value back.

`Structure` is the kind for a new content change that is not typing, formatting, clipboard, image, or a canvas gesture.

Creating or deleting a whole notebook is not an undo step. Switching notes clears the stacks, so edits from the previous note cannot be undone. Switching sections or pages is navigation, not a step. Undo of an edit still returns to the section and page stored in that snapshot.

Pressing Tab from the page name into an empty canvas creates a blank text box and does not record it. Typing into that box is recorded.

## Tests

```text
cargo test --bin todo-app-gpui history::tests
```

The tests build an editor in memory. They call the same `record_edit`, snapshot, and stack code the buttons use, then restore without autosave so `notes.json` is left alone. When a test fails, its name is the behavior that broke: canvas text, names, typing merge, bold, redo being cleared, an unmoved drag, moving a box, inserting an image, deleting a box, adding a page, edit mode, restore, and the 100-step cap.
