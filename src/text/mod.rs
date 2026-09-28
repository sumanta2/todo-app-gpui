//! Text layout shared by the editors.
//!
//! - `metrics` — character width for each font
//! - `selection` — hit testing, line width, and cursor movement
//! - `styles` — bold, italic, underline, and strikethrough

pub(crate) mod metrics;
pub(crate) mod selection;
pub(crate) mod styles;
