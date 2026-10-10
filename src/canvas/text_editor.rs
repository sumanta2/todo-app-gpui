//! Reusable styled text field used by titles and other single-block editors.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement};

use crate::constants::colors::{cursor, note_hint, note_ink};
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
                            rgb(cursor())
                        } else {
                            rgba(0x00000000)
                        })
                } else {
                    div()
                })
                .child(
                    div()
                        .text_color(rgb(note_hint()))
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
            row = row.child(if runs.is_empty() {
                div()
                    .text_color(rgb(note_ink()))
                    .text_size(px(font_size))
                    .line_height(px(line_height))
                    .child("\u{00A0}")
                    .into_any_element()
            } else {
                styled_line_element(&runs, note_ink(), self.zoom)
            });

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

/// Paints one line. Same-size runs share one text layout, so a bold run does not
/// insert a gap before the regular characters that follow it.
pub(crate) fn styled_line_element(
    runs: &[crate::text::styles::StyledRun],
    fallback_color: u32,
    zoom: f32,
) -> AnyElement {
    let zoom = if zoom <= 0.05 { 1.0 } else { zoom };
    let groups = crate::text::shaping::font_size_groups(runs, fallback_color);
    let mut row = div().flex().flex_row().items_center();
    for group in groups {
        let font_size = group.font_size * zoom;
        let line_height =
            crate::constants::typography::line_height_for_font_size(group.font_size) * zoom;
        row = row.child(
            div()
                .flex_shrink_0()
                .text_size(px(font_size))
                .line_height(px(line_height))
                .whitespace_nowrap()
                .child(gpui::StyledText::new(group.text).with_runs(group.runs)),
        );
    }
    row.into_any_element()
}
