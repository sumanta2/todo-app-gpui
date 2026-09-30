//! Highlight layer for a text selection.
//!
//! The bars live in their own entity so a drag across many characters repaints the highlight
//! without rebuilding the sidebar, ribbon, and every canvas block.

use std::cell::RefCell;

use gpui::{
    canvas, fill, point, prelude::*, px, rgb, size, Bounds, Context, Entity, IntoElement, Render,
    Window,
};

use crate::app::NotesApp;
use crate::constants::typography::line_height_for_font_size;
use crate::models::ActiveField;
use crate::text::selection::{get_selection_range, prefix_x, LineHit, TextHitCache};

/// Which buffer this highlight layer follows.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HighlightKind {
    Body,
    Viewer,
}

pub(crate) struct SelectionOverlay {
    app: Entity<NotesApp>,
    kind: HighlightKind,
}

impl SelectionOverlay {
    pub(crate) fn new(app: Entity<NotesApp>, kind: HighlightKind) -> Self {
        Self { app, kind }
    }
}

impl Render for SelectionOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = self.app.read(cx);
        let (range, cache) = match self.kind {
            HighlightKind::Body => (
                get_selection_range(app.edit_body_cursor, app.edit_body_anchor),
                app.body_hit_cache.as_ref(),
            ),
            HighlightKind::Viewer => (
                get_selection_range(app.viewer_text_cursor, app.viewer_text_anchor),
                app.viewer_hit_cache.as_ref(),
            ),
        };

        let bar_h = app.canvas_selection_height();
        let caret_h = app.canvas_cursor_height();
        let show_caret = range.is_none() && (app.cursor_visible || app.is_selecting_body || app.is_selecting_viewer_text);
        let cursor = match self.kind {
            HighlightKind::Body => app.edit_body_cursor,
            HighlightKind::Viewer => app.viewer_text_cursor,
        };
        let bars = match (range, cache) {
            (Some((start, end)), Some(cache)) => selection_bars(cache, start, end),
            _ => Vec::new(),
        };
        let caret = if show_caret {
            cache.and_then(|cache| caret_origin(cache, cursor))
        } else {
            None
        };

        canvas(
            move |bounds, _, _| (bounds, bars, bar_h, caret, caret_h),
            |_, (bounds, bars, bar_h, caret, caret_h), window, _| {
                for (x, y, width) in bars {
                    if width <= 0.0 {
                        continue;
                    }
                    let rect = Bounds {
                        origin: point(bounds.origin.x + px(x), bounds.origin.y + px(y)),
                        size: size(px(width), px(bar_h)),
                    };
                    window.paint_quad(fill(rect, rgb(0x0078d4)));
                }
                if let Some((x, y)) = caret {
                    let rect = Bounds {
                        origin: point(bounds.origin.x + px(x), bounds.origin.y + px(y)),
                        size: size(px(2.0), px(caret_h)),
                    };
                    window.paint_quad(fill(rect, rgb(0x0078d4)));
                }
            },
        )
        .absolute()
        .size_full()
    }
}

/// Selection rectangles in text-local coordinates: `(x, y, width)`.
///
/// Only the lines that overlap the range are visited.
pub(crate) fn selection_bars(cache: &TextHitCache, start: usize, end: usize) -> Vec<(f32, f32, f32)> {
    let mut bars = Vec::new();
    if start >= end || cache.lines.is_empty() {
        return bars;
    }
    let line_height = line_height_for_font_size(cache.font_size);
    let first = line_index_at(&cache.lines, start);
    let last = line_index_at(&cache.lines, end.saturating_sub(1));
    for line_idx in first..=last {
        let line = &cache.lines[line_idx];
        let line_chars = line.prefix.len().saturating_sub(1);
        let line_end = line.start + line_chars;
        let y = line_idx as f32 * line_height + 1.0;
        let overlap_start = start.max(line.start);
        let overlap_end = end.min(line_end);
        if overlap_start < overlap_end {
            let loc_start = overlap_start - line.start;
            let loc_end = overlap_end - line.start;
            let x0 = prefix_x(&line.prefix, loc_start);
            let x1 = prefix_x(&line.prefix, loc_end);
            bars.push((x0, y, (x1 - x0).max(0.0)));
        } else if line_chars == 0 && start <= line.start && end > line.start {
            bars.push((0.0, y, 6.0));
        }
    }
    bars
}

fn line_index_at(lines: &[LineHit], index: usize) -> usize {
    let mut lo = 0usize;
    let mut hi = lines.len();
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if lines[mid].start <= index {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

fn caret_origin(cache: &TextHitCache, index: usize) -> Option<(f32, f32)> {
    if cache.lines.is_empty() {
        return Some((0.0, 2.0));
    }
    let line_idx = line_index_at(&cache.lines, index);
    let line = &cache.lines[line_idx];
    let local = index.saturating_sub(line.start);
    let x = prefix_x(&line.prefix, local) - 1.0;
    let y = line_idx as f32 * line_height_for_font_size(cache.font_size) + 2.0;
    Some((x, y))
}

impl NotesApp {
    fn selection_layer(&self, kind: HighlightKind, cx: &mut Context<Self>) -> Entity<SelectionOverlay> {
        let slot = match kind {
            HighlightKind::Body => &self.body_selection_overlay,
            HighlightKind::Viewer => &self.viewer_selection_overlay,
        };
        if let Some(entity) = slot.borrow().clone() {
            return entity;
        }
        let app = cx.entity();
        let entity = cx.new(move |cx| {
            cx.observe(&app, |_, _, cx| cx.notify()).detach();
            SelectionOverlay::new(app.clone(), kind)
        });
        *slot.borrow_mut() = Some(entity.clone());
        entity
    }

    pub(crate) fn body_selection_layer(&self, cx: &mut Context<Self>) -> Entity<SelectionOverlay> {
        self.selection_layer(HighlightKind::Body, cx)
    }

    pub(crate) fn viewer_selection_layer(&self, cx: &mut Context<Self>) -> Entity<SelectionOverlay> {
        self.selection_layer(HighlightKind::Viewer, cx)
    }

    /// Points later hit tests at this block. Reloads the buffer only when the block changed,
    /// so a drag inside the block that is already open does not copy the text.
    pub(crate) fn place_body_pointer(
        &mut self,
        block_id: &str,
        block_index: Option<usize>,
        fallback: (f32, f32),
    ) -> (f32, f32, bool) {
        self.pending_caret = None;
        let same = self.active_field == ActiveField::Body
            && self.active_block_index == block_index
            && self.active_text_block_id.as_deref() == Some(block_id);
        let origin = if same {
            self.body_drag_origin.unwrap_or(fallback)
        } else {
            self.sync_active_text_block();
            self.active_text_block_id = Some(block_id.to_string());
            self.active_block_index = block_index;
            self.active_field = ActiveField::Body;
            self.capture_segment(block_id, block_index)
                .unwrap_or(fallback)
        };
        self.body_drag_origin = Some(origin);
        (origin.0, origin.1, same)
    }

    /// Shows the new caret or highlight. An already-open block updates only the overlay.
    pub(crate) fn show_body_pointer(&mut self, same_block: bool, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        if same_block {
            self.refresh_body_selection(cx, false);
        } else {
            cx.notify();
        }
    }

    pub(crate) fn end_body_pointer(&mut self, cx: &mut Context<Self>) {
        self.apply_selection_drag(cx);
        if self.edit_body_anchor == Some(self.edit_body_cursor) {
            self.edit_body_anchor = None;
        }
        self.is_selecting_body = false;
        self.refresh_body_selection(cx, false);
    }

    pub(crate) fn end_viewer_pointer(&mut self, cx: &mut Context<Self>) {
        self.apply_selection_drag(cx);
        if self.viewer_text_anchor == Some(self.viewer_text_cursor) {
            self.viewer_text_anchor = None;
        }
        self.is_selecting_viewer_text = false;
        self.refresh_viewer_selection(cx, false);
    }

    /// Repaint the highlight. `also_app` also rebuilds the window, used once when a drag
    /// first becomes a real range so the caret at the anchor is cleared.
    pub(crate) fn refresh_body_selection(&self, cx: &mut Context<Self>, also_app: bool) {
        self.refresh_selection(&self.body_selection_overlay, cx, also_app);
    }

    pub(crate) fn refresh_viewer_selection(&self, cx: &mut Context<Self>, also_app: bool) {
        self.refresh_selection(&self.viewer_selection_overlay, cx, also_app);
    }

    /// Remember the latest drag point and hit-test it once on the next frame.
    ///
    /// Mouse samples arrive faster than the window can paint. Applying every sample makes the
    /// highlight fall behind the pointer. Keeping only the newest point tracks the pointer.
    pub(crate) fn queue_selection_drag(
        &mut self,
        kind: HighlightKind,
        pos: gpui::Point<gpui::Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selection_drag_sample = Some((kind, pos));
        if self.selection_drag_queued {
            return;
        }
        self.selection_drag_queued = true;
        cx.on_next_frame(window, |this, _, cx| {
            this.apply_selection_drag(cx);
        });
    }

    /// Hit-test the newest drag sample and move the highlight to that character.
    pub(crate) fn apply_selection_drag(&mut self, cx: &mut Context<Self>) {
        self.selection_drag_queued = false;
        let Some((kind, pos)) = self.selection_drag_sample.take() else {
            return;
        };
        match kind {
            HighlightKind::Body => {
                let Some((item_x, item_y)) = self.body_drag_origin else {
                    return;
                };
                let drag_idx = self.body_index_at_mouse(pos, item_x, item_y, 6.0, 18.0);
                if crate::text::selection::assign_if_changed(&mut self.edit_body_cursor, drag_idx) {
                    self.refresh_body_selection(cx, false);
                }
            }
            HighlightKind::Viewer => {
                let Some((item_x, item_y)) = self.viewer_drag_origin else {
                    return;
                };
                let drag_idx = self.viewer_index_cached(pos, item_x, item_y);
                if crate::text::selection::assign_if_changed(&mut self.viewer_text_cursor, drag_idx)
                {
                    self.refresh_viewer_selection(cx, false);
                }
            }
        }
    }

    fn refresh_selection(
        &self,
        slot: &RefCell<Option<Entity<SelectionOverlay>>>,
        cx: &mut Context<Self>,
        also_app: bool,
    ) {
        let overlay = slot.borrow().clone();
        if let Some(overlay) = overlay {
            overlay.update(cx, |_, cx| cx.notify());
            if also_app {
                cx.notify();
            }
        } else {
            cx.notify();
        }
    }
}
