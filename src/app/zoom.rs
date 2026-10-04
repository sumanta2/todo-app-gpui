//! Canvas magnification. The stored note stays in unscaled coordinates.

use gpui::Context;

use crate::app::NotesApp;

/// Zoom choices shown in the View ribbon. `1.0` is 100%.
pub(crate) const ZOOM_OPTIONS: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 4.0;

impl NotesApp {
    pub(crate) fn zoom_percent_label(&self) -> String {
        format!("{}%", (self.canvas_zoom * 100.0).round() as i32)
    }

    /// Screen X of a canvas point, including pan and zoom.
    pub(crate) fn place_x(&self, x: f32) -> f32 {
        x * self.canvas_zoom + self.pan_x
    }

    /// Screen Y of a canvas point, including pan and zoom.
    pub(crate) fn place_y(&self, y: f32) -> f32 {
        y * self.canvas_zoom + self.pan_y
    }

    /// Scales a canvas length into screen pixels.
    pub(crate) fn scaled(&self, value: f32) -> f32 {
        value * self.canvas_zoom
    }

    /// Sets an absolute zoom, keeping the canvas point under `anchor` fixed on screen.
    /// `anchor` is in canvas-widget coordinates (after the sidebar and ribbon).
    pub(crate) fn set_canvas_zoom(
        &mut self,
        zoom: f32,
        anchor: (f32, f32),
        cx: &mut Context<Self>,
    ) {
        let old = self.canvas_zoom.max(0.05);
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let canvas_x = (anchor.0 - self.pan_x) / old;
        let canvas_y = (anchor.1 - self.pan_y) / old;
        self.canvas_zoom = zoom;
        self.pan_x = anchor.0 - canvas_x * zoom;
        self.pan_y = anchor.1 - canvas_y * zoom;
        self.zoom_menu_open = false;
        cx.notify();
    }

    pub(crate) fn zoom_by(&mut self, factor: f32, anchor: (f32, f32), cx: &mut Context<Self>) {
        self.set_canvas_zoom(self.canvas_zoom * factor, anchor, cx);
    }

    pub(crate) fn reset_canvas_zoom(&mut self, cx: &mut Context<Self>) {
        self.set_canvas_zoom(1.0, (0.0, 0.0), cx);
    }

    /// Ctrl+wheel and touchpad pinch zoom the canvas. Other scrolls pan it.
    pub(crate) fn handle_canvas_scroll(
        &mut self,
        event: &gpui::ScrollWheelEvent,
        cx: &mut Context<Self>,
    ) {
        let dy = match event.delta {
            gpui::ScrollDelta::Pixels(point) => point.y.as_f32(),
            gpui::ScrollDelta::Lines(point) => point.y * 20.0,
        };
        if event.modifiers.control || event.modifiers.platform {
            if dy.abs() < 0.01 {
                return;
            }
            let factor = if dy > 0.0 { 1.1 } else { 1.0 / 1.1 };
            let anchor = (
                event.position.x.as_f32() - self.layout_sidebar_w(),
                event.position.y.as_f32() - self.canvas_top_y,
            );
            self.zoom_by(factor, anchor, cx);
            return;
        }
        match event.delta {
            gpui::ScrollDelta::Pixels(point) => {
                self.pan_x = (self.pan_x + point.x.as_f32()).min(0.0);
                self.pan_y = (self.pan_y + point.y.as_f32()).min(0.0);
            }
            gpui::ScrollDelta::Lines(point) => {
                self.pan_x = (self.pan_x + point.x * 20.0).min(0.0);
                self.pan_y = (self.pan_y + point.y * 20.0).min(0.0);
            }
        }
        cx.notify();
    }
}
