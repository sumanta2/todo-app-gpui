//! Top-level layout.
//!
//! - `title_bar` — window caption, search, and window controls
//! - `ribbon` — Home and View menus
//! - `sidebar` — notebook list and the unpinned note switcher
//! - `detail_pane` — page list plus editor or viewer

pub(crate) mod detail_pane;
pub(crate) mod ribbon;
pub(crate) mod sidebar;
pub(crate) mod title_bar;
