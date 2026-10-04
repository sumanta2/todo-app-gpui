//! Reusable styled text field used by titles and other single-block editors.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement};

use crate::constants::colors::{NOTE_HINT, NOTE_INK};
use crate::constants::typography::CURSOR_WIDTH;

#[derive(Clone)]
pub struct TextEditor {
    pub text: String,
    pub bold_flags: Vec<bool>,
    pub italic_flags: Vec<bool>,
    pub underline_flags: Vec<bool>,
    pub strike_flags: Vec<bool>,
    pub font_families: Vec<u8>,
    pub font_sizes: Vec<f32>,
    pub font_colors: Vec<u32>,
    pub bg_colors: Vec<u32>,
    pub line_layouts: Vec<crate::models::LineLayout>,
    pub zoom: f32,
    pub cursor_visible: bool,
    pub font_size: f32,
}

impl TextEditor {
    /// Line box height. The extra leading scales with zoom so the caret stays on the glyph.
    #[inline]
    pub fn line_height(&self) -> f32 {
        let zoom = self.zoom.max(0.05);
        crate::constants::typography::line_height_for_font_size(self.font_size / zoom) * zoom
    }

    /// Caret height, scaled the same way as the line box.
    #[inline]
    pub fn cursor_height(&self) -> f32 {
        let zoom = self.zoom.max(0.05);
        crate::constants::typography::cursor_height_for_font_size(self.font_size / zoom) * zoom
    }

    /// Renders the full text editor while allowing a caller to wrap each visual line.
    pub fn render_editor_with_line_wrapper<F>(
        &self,
        is_focused: bool,
        mut line_wrapper: F,
    ) -> AnyElement
    where
        F: FnMut(usize, usize, &str, gpui::Div) -> AnyElement,
    {
        let text = self.text.as_str();
        let font_size = self.font_size;
        let line_height = self.line_height();
        let cursor_height = self.cursor_height();

        if text.is_empty() {
            let row = div()
                .relative()
                .flex()
                .items_center()
                .h(px(line_height))
                .line_height(px(line_height))
                .text_size(px(font_size))
                .child(if is_focused {
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(((line_height - cursor_height) * 0.5).max(0.0)))
                        .w(px(CURSOR_WIDTH))
                        .h(px(cursor_height))
                        .bg(if self.cursor_visible {
                            rgb(NOTE_INK)
                        } else {
                            rgba(0x00000000)
                        })
                } else {
                    div()
                })
                .child(
                    div()
                        .text_color(rgb(NOTE_HINT))
                        .text_size(px(font_size))
                        .child("Type note..."),
                );
            return line_wrapper(0, 0, "", row).into_any_element();
        }

        let logical_lines: Vec<&str> = text.split('\n').collect();
        let mut line_rows: Vec<AnyElement> = Vec::new();
        let mut global_offset = 0;

        for (line_idx, line) in logical_lines.iter().enumerate() {
            let line_len = line.chars().count();
            let line_start = global_offset;
            let line_end = global_offset + line_len;

            let line_bold_flags = if line_start < self.bold_flags.len() {
                let end = line_end.min(self.bold_flags.len());
                let mut f = self.bold_flags[line_start..end].to_vec();
                if f.len() < line_len {
                    f.resize(line_len, false);
                }
                f
            } else {
                vec![false; line_len]
            };

            let mut row = div()
                .relative()
                .flex()
                .flex_row()
                .flex_shrink_0()
                .items_center()
                .text_size(px(font_size))
                .line_height(px(line_height))
                .whitespace_nowrap()
                .h(px(line_height));
            row = style_line_row(
                row,
                self.line_layouts.get(line_idx).copied().unwrap_or_default(),
                self.zoom,
            );

            let line_italic = crate::text::styles::slice_flags(
                &self.italic_flags,
                line_start,
                line_end,
                line_len,
            );
            let line_underline = crate::text::styles::slice_flags(
                &self.underline_flags,
                line_start,
                line_end,
                line_len,
            );
            let line_strike = crate::text::styles::slice_flags(
                &self.strike_flags,
                line_start,
                line_end,
                line_len,
            );
            let line_families = slice_u8(&self.font_families, line_start, line_end, line_len);
            let line_sizes = slice_f32(&self.font_sizes, line_start, line_end, line_len);
            let line_colors = slice_u32(&self.font_colors, line_start, line_end, line_len);
            let line_bgs = slice_u32(&self.bg_colors, line_start, line_end, line_len);

            // Render line text as styled runs (bold, italic, underline, strikethrough, font, size)
            let runs = crate::text::styles::split_text_into_full_runs(
                line,
                &line_bold_flags,
                &line_italic,
                &line_underline,
                &line_strike,
                &line_families,
                &line_sizes,
                &line_colors,
                &line_bgs,
            );
            let mut line_elements = Vec::new();
            if runs.is_empty() {
                line_elements.push(
                    div()
                        .text_color(rgb(NOTE_INK))
                        .text_size(px(font_size))
                        .line_height(px(line_height))
                        .child("\u{00A0}")
                        .into_any_element(),
                );
            } else {
                for run in &runs {
                    let color = NOTE_INK;
                    line_elements.push(styled_run_element(run, color, true, self.zoom));
                }
            }

            row = row.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .children(line_elements),
            );

            let row = line_wrapper(line_idx, line_start, line, row);
            line_rows.push(row);
            global_offset += line_len + 1; // +1 for \n
        }

        div()
            .w_full()
            .flex()
            .flex_col()
            .children(line_rows)
            .into_any_element()
    }
}

pub(crate) fn style_line_row(
    row: gpui::Div,
    layout: crate::models::LineLayout,
    zoom: f32,
) -> gpui::Div {
    let indent = layout.indent as f32 * crate::constants::typography::INDENT_STEP * zoom;
    let row = row.w_full().pl(px(indent));
    match layout.align {
        1 => row.justify_center(),
        2 => row.justify_end(),
        _ => row.justify_start(),
    }
}

fn slice_u8(values: &[u8], start: usize, end: usize, len: usize) -> Vec<u8> {
    let mut sliced = if start < values.len() {
        values[start..end.min(values.len())].to_vec()
    } else {
        Vec::new()
    };
    sliced.resize(len, 0);
    sliced
}

fn slice_u32(values: &[u32], start: usize, end: usize, len: usize) -> Vec<u32> {
    let mut sliced = if start < values.len() {
        values[start..end.min(values.len())].to_vec()
    } else {
        Vec::new()
    };
    sliced.resize(len, 0);
    sliced
}

fn slice_f32(values: &[f32], start: usize, end: usize, len: usize) -> Vec<f32> {
    let mut sliced = if start < values.len() {
        values[start..end.min(values.len())].to_vec()
    } else {
        Vec::new()
    };
    sliced.resize(len, crate::constants::typography::CANVAS_BODY_FONT_SIZE);
    sliced
}

/// Paints one styled text run. Decorations are skipped for invisible width-matching ghosts.
pub(crate) fn styled_run_element(
    run: &crate::text::styles::StyledRun,
    color: u32,
    decorations: bool,
    zoom: f32,
) -> AnyElement {
    let zoom = if zoom <= 0.05 { 1.0 } else { zoom };
    let base_size = if run.font_size > 0.0 {
        run.font_size
    } else {
        crate::constants::typography::CANVAS_BODY_FONT_SIZE
    };
    let font_size = base_size * zoom;
    let line_height = crate::constants::typography::line_height_for_font_size(base_size) * zoom;
    let mut text_color = if run.font_color != 0 {
        run.font_color
    } else {
        color
    };
    if run.bg_color != 0 && run.font_color == 0 && text_color != 0 {
        text_color = 0x1e1e1e;
    }
    let mut el = div()
        .relative()
        .text_color(if text_color == 0 {
            rgba(0x00000000)
        } else {
            rgb(text_color)
        })
        .text_size(px(font_size))
        .line_height(px(line_height))
        .font_family(crate::constants::typography::font_style_name(
            run.font_family,
        ))
        .font_weight(if run.is_bold {
            gpui::FontWeight::BOLD
        } else {
            gpui::FontWeight::NORMAL
        });
    if run.bg_color != 0 {
        el = el.bg(rgb(run.bg_color));
    }
    if run.is_italic {
        el = el.italic();
    }
    el = el.child(run.text.replace(' ', "\u{00A0}"));
    if decorations && run.is_underline {
        el = el.border_b_1().border_color(rgb(text_color));
    }
    if decorations && run.is_strike {
        el = el.child(
            div()
                .absolute()
                .top(px((font_size * 0.55).max(1.0)))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(1.0))
                .bg(rgb(text_color)),
        );
    }
    el.into_any_element()
}
