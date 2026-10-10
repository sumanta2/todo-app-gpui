//! Text layout shared by the editors.
//!
//! - `shaping` — caret advances from GPUI's glyph shaper
//! - `selection` — hit testing, line width, and cursor movement
//! - `styles` — bold, italic, underline, and strikethrough

pub(crate) mod selection;
pub(crate) mod shaping;
pub(crate) mod styles;
