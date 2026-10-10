#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod constants;
mod helpers;
mod models;
mod text;
mod views;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, Focusable, IntoElement, Render,
    TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

use crate::app::NotesApp;
impl Render for NotesApp {
    /// Builds the main application layout for the notes editor.
    ///
    /// The caption bar and ribbon span the window. Under them, a pinned notebook list sits
    /// beside the canvas and page list. An unpinned list stays hidden and the note switcher
    /// remains on the section tabs.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = if self.full_page_view || !self.sidebar_expanded() {
            None
        } else {
            Some(self.render_sidebar(window, cx))
        };
        let show_ribbon = !self.full_page_view && self.selected_note().is_some();
        let ribbon = if show_ribbon {
            Some(self.render_home_ribbon(cx))
        } else {
            None
        };
        let docked_dropdowns = if show_ribbon {
            self.render_docked_dropdowns(cx)
        } else {
            None
        };
        let home_menus = if show_ribbon {
            self.render_home_menu_layers(cx)
        } else {
            Vec::new()
        };
        let view_menus = if show_ribbon {
            self.render_view_menu_layers(cx)
        } else {
            Vec::new()
        };
        let note_menus = if show_ribbon {
            self.render_note_menu_layers(cx)
        } else {
            Vec::new()
        };
        let detail_pane = self.render_detail_pane(window, cx);
        let title_bar = self.render_title_bar(window, cx);

        div()
            .id("notes-app")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _, cx| {
                this.handle_key(event, cx);
            }))
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.drag_page_sidebar(event.position.x.as_f32()) {
                    cx.notify();
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.page_sidebar_resizing {
                        this.page_sidebar_resizing = false;
                        cx.notify();
                    }
                }),
            )
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(crate::constants::colors::onenote_bar()))
            .text_color(rgb(crate::constants::colors::onenote_ink()))
            .font_family(self.font_family.as_str())
            .child(title_bar)
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .w_full()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.title_search_focused {
                                this.title_search_focused = false;
                                cx.notify();
                            }
                        }),
                    )
                    .children(ribbon)
                    .children(docked_dropdowns)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_1()
                            .w_full()
                            .min_h(px(0.0))
                            .children(sidebar)
                            .child(detail_pane),
                    )
                    .children(home_menus)
                    .children(view_menus)
                    .children(note_menus),
            )
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
    // The first paint uses a few megabytes of stack. 8 MB is enough and is faster to
    // reserve than the previous 64 MB thread stack.
    let stack_size = 8 * 1024 * 1024;
    let builder = std::thread::Builder::new()
        .name("notes-main".to_string())
        .stack_size(stack_size);
    let handler = builder
        .spawn(|| {
            application().run(|cx: &mut App| {
                // Grayscale text is cheaper to rasterize than subpixel on the first frame.
                cx.set_text_rendering_mode(gpui::TextRenderingMode::Grayscale);
                let bounds = Bounds::centered(None, size(px(1000.0), px(700.0)), cx);

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("Modern Notes".into()),
                            appears_transparent: true,
                            traffic_light_position: None,
                        }),
                        ..Default::default()
                    },
                    |window, cx| {
                        let app = cx.new(NotesApp::new); // calling NotesApp::new in app/editing.rs
                                                         // Paint text first. Decode images on the next frame so the window can appear.
                        app.update(cx, |_, cx| {
                            cx.on_next_frame(window, |this, _, cx| {
                                if this.defer_images {
                                    this.defer_images = false;
                                    cx.notify();
                                }
                            });
                        });
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
