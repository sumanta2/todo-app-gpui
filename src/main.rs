#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod helpers;
mod models;
mod views;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, Focusable, IntoElement, Render, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

use crate::app::NotesApp;

impl Render for NotesApp {
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
            .font_family("Segoe UI")
            .child(sidebar)
            .child(detail_pane)
    }
}

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
                        let app = cx.new(NotesApp::new);
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
