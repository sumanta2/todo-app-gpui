#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod constants;
mod font_metrics;
mod helpers;
mod models;
mod text_selection;
mod views;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, Focusable, IntoElement, Render, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

use crate::app::NotesApp;

impl Render for NotesApp {
    /// Builds the main application layout for the notes editor.
    ///
    /// The layout contains the left sidebar for note selection and the right detail pane
    /// where note content, page management, and canvas editing are rendered.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = self.render_sidebar(window, cx);
        let detail_pane = self.render_detail_pane(window, cx);

        div()
            .id("notes-app")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _, cx| {
                this.handle_key(event, cx);
            }))
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xd4d4d4))
            .font_family(self.font_family.as_str())
            .child(sidebar)
            .child(detail_pane)
    }
}

/// Application startup flow:
///
/// 1. `cx.new(NotesApp::new)` creates the app state and initializes persisted note data,
///    active selection, focus handles, and editor-specific flags.
/// 2. The `Render for NotesApp` implementation is responsible only for drawing the current
///    UI from that in-memory state.
/// 2.1. When in-memory state changes, we call `cx.notify()` or related update paths; GPUI then asks the
///    app to render again using the latest values.
///
/// This separation keeps the app maintainable:
/// - constructor / action methods = data setup and mutations
/// - render = view composition from current state
/// - no heavy state initialization should live inside the render function itself
///
/// The extra stack size is intentional because the UI logic is fairly rich and the
/// editor/canvas rendering can recurse deeply enough to overflow the default thread stack
/// on Windows.
fn main() {
    // The render function is large; increase the main thread stack size to 64 MB
    // to avoid STATUS_STACK_OVERFLOW on Windows (default stack is only 1 MB).
    let stack_size = 64 * 1024 * 1024; // 64 MB we reserve, but here ~2-4 mb used
    let builder = std::thread::Builder::new()
        .name("notes-main".to_string())
        .stack_size(stack_size);
    let handler = builder
        .spawn(|| {
            application().run(|cx: &mut App| {
                let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |window, cx| {
                        let app = cx.new(NotesApp::new); // calling NotesApp constructor present at action.rs
                        app.focus_handle(cx).focus(window, cx);
                        app
                    },
                )
                .expect("failed to open the Notes window");

                cx.activate(true);
            });
        })
        .expect("failed to spawn main thread");
    handler.join().expect("main thread panicked");
}
