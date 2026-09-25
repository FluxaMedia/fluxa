//! Typed visual primitives shared by every Fluxa native surface.
//!
//! This is intentionally a constrained, Rust-native subset of CSS rather than
//! a CSS parser. Screens describe layout, paint, text, assets, and transitions
//! with the same values on desktop, Android, and TV; each host only supplies
//! the surface and input adapter.

use std::time::Duration;

use crate::{Color, Easing};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    #[default]
    Block,
    FlexRow,
    FlexColumn,
    Overlay,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
    Scroll,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutStyle {
    pub display: Display,
    pub gap: f32,
    pub padding: Edges,
    pub margin: Edges,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub align_x: Align,
    pub align_y: Align,
    pub overflow: Overflow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderStyle {
    pub width: f32,
    pub color: Color,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShadowStyle {
    pub color: Color,
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintStyle {
    pub background: Option<Color>,
    pub border: BorderStyle,
    pub shadow: Option<ShadowStyle>,
    pub opacity: f32,
    pub clip_radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontStyle {
    pub family: String,
    pub size: f32,
    pub weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            family: "Fluxa Sans".to_owned(),
            size: 16.0,
            weight: 400,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetKind {
    HeroBackground,
    Poster,
    EpisodeThumbnail,
    TitleLogo,
    Avatar,
    SvgIcon,
    Emoji,
    #[default]
    Generic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageStyle {
    pub kind: AssetKind,
    pub fit: ImageFit,
    pub focal_point: [f32; 2],
    pub opacity: f32,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            kind: AssetKind::Generic,
            fit: ImageFit::Cover,
            focal_point: [0.5, 0.5],
            opacity: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionStyle {
    pub duration: Duration,
    pub easing: Easing,
}

impl Default for TransitionStyle {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(220),
            easing: Easing::EaseOutCubic,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub font: FontStyle,
    pub image: Option<ImageStyle>,
    pub transition: Option<TransitionStyle>,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            paint: PaintStyle {
                opacity: 1.0,
                ..PaintStyle::default()
            },
            font: FontStyle::default(),
            image: None,
            transition: None,
        }
    }
}

impl UiStyle {
    pub fn card(background: Color, radius: f32) -> Self {
        Self {
            paint: PaintStyle {
                background: Some(background),
                border: BorderStyle {
                    radius,
                    ..BorderStyle::default()
                },
                clip_radius: radius,
                opacity: 1.0,
                ..PaintStyle::default()
            },
            image: Some(ImageStyle {
                kind: AssetKind::Poster,
                ..ImageStyle::default()
            }),
            ..Self::default()
        }
    }

    pub fn button(background: Color, radius: f32) -> Self {
        Self {
            layout: LayoutStyle {
                display: Display::FlexRow,
                align_x: Align::Center,
                align_y: Align::Center,
                ..LayoutStyle::default()
            },
            paint: PaintStyle {
                background: Some(background),
                border: BorderStyle {
                    radius,
                    ..BorderStyle::default()
                },
                clip_radius: radius,
                opacity: 1.0,
                ..PaintStyle::default()
            },
            font: FontStyle {
                weight: 600,
                ..FontStyle::default()
            },
            transition: Some(TransitionStyle::default()),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_asset_kinds_keep_poster_and_hero_paint_contracts_distinct() {
        let poster = UiStyle::card(Color::rgb(0.1, 0.1, 0.1), 12.0);
        let hero = ImageStyle {
            kind: AssetKind::HeroBackground,
            fit: ImageFit::Cover,
            ..ImageStyle::default()
        };

        assert_eq!(poster.image.expect("card image").kind, AssetKind::Poster);
        assert_eq!(hero.kind, AssetKind::HeroBackground);
    }

    #[test]
    fn button_style_has_a_real_text_and_motion_contract() {
        let style = UiStyle::button(Color::rgb(1.0, 1.0, 1.0), 9.0);

        assert_eq!(style.font.weight, 600);
        assert!(style.transition.is_some());
        assert_eq!(style.layout.align_x, Align::Center);
    }
}
