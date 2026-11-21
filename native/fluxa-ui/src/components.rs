//! Reusable Fluxa UI components.
//!
//! Components own visual paint and typography. Screens own composition and
//! navigation only. This keeps desktop, Android, and TV on the same card and
//! control implementation while allowing each host to provide a different
//! viewport and input adapter.

use crate::gfx::{
    Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, RichText, Sense, Ui, Vec2,
};

mod avatar;
mod brand;
mod buttons;
mod cards;
mod content;
mod controls;
mod detail_modal;
mod downloads;
mod dropdown;
mod filmstrip;
mod fit;
mod focus;
mod glass;
mod icon_button;
mod input;
mod input_touch;
mod keycap;
mod media;
mod menu;
mod mini_player;
mod overlay;
mod pan;
mod profile_tile;
mod recent_searches;
mod seats;
mod slider;
mod sticky_bar;
mod streams;
mod subtitle_preview;
mod surfaces;
mod ticket;
mod toggle;

pub use avatar::*;
pub use buttons::*;
pub(crate) use cards::*;
pub use content::*;
pub(crate) use controls::*;
pub use detail_modal::*;
pub use downloads::*;
pub(crate) use dropdown::*;
pub use focus::*;
pub use glass::*;
pub use input::*;
pub use input_touch::{dismiss_text_toolbar, text_grip_hit};
pub use media::*;
pub use menu::*;
pub use mini_player::*;
pub(crate) use overlay::*;
pub use pan::*;
pub use recent_searches::*;
pub(crate) use seats::*;
pub use subtitle_preview::SubtitleLook;
pub use surfaces::*;
pub(crate) use ticket::*;
pub use toggle::*;

use super::{
    ArtworkPriority, HomeAssets, HomeCard, Shade, Viewport, accent, artwork_target_size,
    badge_fill, button_height, cover_uv, danger_color, draw_text, full_uv, ink, radius, shade,
    size, stroke, text_primary, tone, truncate_to_width,
};
