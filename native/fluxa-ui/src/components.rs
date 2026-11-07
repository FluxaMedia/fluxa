//! Reusable Fluxa UI components.
//!
//! Components own visual paint and typography. Screens own composition and
//! navigation only. This keeps desktop, Android, and TV on the same card and
//! control implementation while allowing each host to provide a different
//! viewport and input adapter.

use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, RichText, Sense, Ui, Vec2};
use std::hash::{Hash, Hasher};

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
mod focus;
mod glass;
mod icon_button;
mod input;
mod keycap;
mod media;
mod menu;
mod mini_player;
mod overlay;
mod pan;
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
pub(crate) use brand::*;
pub use buttons::*;
pub(crate) use cards::*;
pub use content::*;
pub(crate) use controls::*;
pub use detail_modal::*;
pub use downloads::*;
pub(crate) use dropdown::*;
pub(crate) use filmstrip::*;
pub use focus::*;
pub use glass::*;
pub use icon_button::*;
pub use input::*;
pub(crate) use keycap::*;
pub use media::*;
pub use menu::*;
pub use mini_player::*;
pub(crate) use overlay::*;
pub use pan::*;
pub use recent_searches::*;
pub(crate) use seats::*;
pub(crate) use slider::*;
pub use sticky_bar::*;
pub(crate) use streams::*;
pub use subtitle_preview::SubtitleLook;
pub(crate) use subtitle_preview::subtitle_sample;
pub use surfaces::*;
pub(crate) use ticket::*;
pub use toggle::*;

use super::{
    ArtworkPriority, DetailEpisode, DetailTrailer, HomeAssets, HomeCard, HomeLayout, Shade,
    UiMetrics, Viewport, accent, artwork_target_size, background, badge_fill, button_height,
    cover_uv, danger_color, draw_text, full_uv, ink, paint_vertical_gradient, radius, shade, size,
    stroke, text_primary, text_width, tone, truncate_to_width,
};
