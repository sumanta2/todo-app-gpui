//! Caret advances taken from the same shaper GPUI uses to paint text.
//!
//! Runs that share a font size are shaped together. Shaping each run alone makes the
//! last glyph of a bold run wider than it is beside the following regular text, which
//! pushes every later caret to the right.
//!
//! Positions are stored in unzoomed pixels: the shaper runs at `size * zoom`, then x is
//! divided by zoom so the selection overlay can scale them again.

use gpui::{font, px, FontWeight, Hsla, SharedString, StrikethroughStyle, TextRun, UnderlineStyle, Window};

use crate::text::styles::{split_text_into_full_runs, StyledRun};

/// One face for a field that does not mix fonts inside the line.
pub(crate) struct UniformFace<'a> {
    pub(crate) family: &'a str,
    pub(crate) size: f32,
    pub(crate) weight: FontWeight,
    pub(crate) italic: bool,
}

/// One stretch of a line that shares a font size, shaped as a single GPUI line.
pub(crate) struct FontSizeGroup {
    pub(crate) font_size: f32,
    pub(crate) text: SharedString,
    pub(crate) runs: Vec<TextRun>,
}

/// Groups styled runs that share a font size so bold and regular text stay on one line.
pub(crate) fn font_size_groups(runs: &[StyledRun], fallback_color: u32) -> Vec<FontSizeGroup> {
    let mut groups = Vec::new();
    let mut current: Option<FontSizeGroup> = None;
    for run in runs {
        let display = run.text.replace(' ', "\u{00A0}");
        if display.is_empty() {
            continue;
        }
        let text_run = text_run_for(&display, run, fallback_color);
        let same_group = current
            .as_ref()
            .is_some_and(|group| (group.font_size - run.font_size).abs() < 0.05);
        if same_group {
            let group = current.as_mut().unwrap();
            group.text = SharedString::from(format!("{}{}", group.text, display));
            group.runs.push(text_run);
        } else {
            if let Some(group) = current.take() {
                groups.push(group);
            }
            current = Some(FontSizeGroup {
                font_size: run.font_size,
                text: SharedString::from(display),
                runs: vec![text_run],
            });
        }
    }
    if let Some(group) = current {
        groups.push(group);
    }
    groups
}

fn text_run_for(display: &str, run: &StyledRun, fallback_color: u32) -> TextRun {
    let family = crate::constants::typography::font_style_name(run.font_family);
    let mut gpui_font = font(family);
    gpui_font.weight = if run.is_bold {
        FontWeight::BOLD
    } else {
        FontWeight::NORMAL
    };
    if run.is_italic {
        gpui_font = gpui_font.italic();
    }
    let color = hsla_color(if run.font_color != 0 {
        run.font_color
    } else {
        fallback_color
    });
    TextRun {
        len: display.len(),
        font: gpui_font,
        color,
        background_color: if run.bg_color != 0 {
            Some(hsla_color(run.bg_color))
        } else {
            None
        },
        underline: if run.is_underline {
            Some(UnderlineStyle {
                thickness: px(1.0),
                color: Some(color),
                wavy: false,
            })
        } else {
            None
        },
        strikethrough: if run.is_strike {
            Some(StrikethroughStyle {
                thickness: px(1.0),
                color: Some(color),
            })
        } else {
            None
        },
    }
}

fn hsla_color(color: u32) -> Hsla {
    Hsla::from(gpui::rgb(color))
}

/// Logical x before each character, plus the x after the last character.
pub(crate) fn shaped_line_prefix(
    window: &Window,
    text: &str,
    bold: &[bool],
    italic: &[bool],
    families: &[u8],
    sizes: &[f32],
    zoom: f32,
) -> Vec<f32> {
    if text.is_empty() {
        return vec![0.0];
    }
    let runs = split_text_into_full_runs(text, bold, italic, &[], &[], families, sizes, &[], &[]);
    let mut prefix = Vec::with_capacity(text.chars().count() + 1);
    prefix.push(0.0);
    let mut origin = 0.0f32;
    for group in font_size_groups(&runs, 0) {
        let group_prefix = shaped_group_prefix(window, &group, zoom);
        for x in group_prefix.into_iter().skip(1) {
            prefix.push(origin + x);
        }
        origin = prefix.last().copied().unwrap_or(origin);
    }
    prefix
}

fn shaped_group_prefix(window: &Window, group: &FontSizeGroup, zoom: f32) -> Vec<f32> {
    let zoom = zoom.max(0.05);
    let shaped = window.text_system().shape_line(
        group.text.clone(),
        px(group.font_size.max(1.0) * zoom),
        &group.runs,
        None,
    );
    let mut prefix = Vec::with_capacity(group.text.chars().count() + 1);
    prefix.push(0.0);
    let mut byte = 0usize;
    for ch in group.text.chars() {
        byte += ch.len_utf8();
        prefix.push(shaped.x_for_index(byte).as_f32() / zoom);
    }
    prefix
}

/// Logical x positions for a line painted with one face.
pub(crate) fn shaped_uniform_prefix(
    window: &Window,
    text: &str,
    face: UniformFace<'_>,
    zoom: f32,
) -> Vec<f32> {
    let zoom = zoom.max(0.05);
    let mut prefix = Vec::with_capacity(text.chars().count() + 1);
    prefix.push(0.0);
    if text.is_empty() {
        return prefix;
    }

    let display = text.replace(' ', "\u{00A0}");
    let mut gpui_font = font(face.family);
    gpui_font.weight = face.weight;
    if face.italic {
        gpui_font = gpui_font.italic();
    }
    let run = TextRun {
        len: display.len(),
        font: gpui_font,
        ..TextRun::default()
    };
    let painted = px((face.size.max(1.0)) * zoom);
    let shaped = window
        .text_system()
        .shape_line(SharedString::from(display.as_str()), painted, &[run], None);

    let mut byte = 0usize;
    for ch in display.chars() {
        byte += ch.len_utf8();
        prefix.push(shaped.x_for_index(byte).as_f32() / zoom);
    }
    prefix
}

/// Character index closest to `rel_x` for a single-face line. `rel_x` is in unzoomed pixels.
pub(crate) fn uniform_index_at_x(
    window: &Window,
    text: &str,
    face: UniformFace<'_>,
    zoom: f32,
    rel_x: f32,
) -> usize {
    let prefix = shaped_uniform_prefix(window, text, face, zoom);
    crate::text::selection::index_in_prefix(&prefix, rel_x)
}

/// UTF-8 byte offset before each character, and one past the last character.
#[cfg(test)]
fn utf8_boundaries(text: &str) -> Vec<usize> {
    let mut bounds = Vec::with_capacity(text.chars().count() + 1);
    bounds.push(0);
    let mut byte = 0usize;
    for ch in text.chars() {
        byte += ch.len_utf8();
        bounds.push(byte);
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::utf8_boundaries;

    #[test]
    fn character_boundaries_follow_utf8_bytes() {
        assert_eq!(utf8_boundaries("Hello"), vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(utf8_boundaries("é"), vec![0, 2]);
        assert_eq!(utf8_boundaries("a😀b"), vec![0, 1, 5, 6]);
        assert_eq!(utf8_boundaries(""), vec![0]);
    }
}
