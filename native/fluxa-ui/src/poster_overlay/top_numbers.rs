use super::*;
use std::sync::RwLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberStyle {
    Outline,
    Solid,
    Ghost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberColor {
    White,
    Gray,
    Accent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopNumbers {
    pub all_rows: bool,
    pub count: usize,
    pub style: NumberStyle,
    pub color: NumberColor,
    pub scale: f32,
    pub beside: bool,
}

static TOP_NUMBERS: RwLock<Option<TopNumbers>> = RwLock::new(None);

pub fn set_top_numbers(numbers: Option<TopNumbers>) {
    *TOP_NUMBERS.write().unwrap() = numbers;
}

fn current_numbers() -> Option<TopNumbers> {
    *TOP_NUMBERS.read().unwrap()
}

const TRENDING_WORDS: [&str; 6] = ["trending", "top", "popular", "trend", "popüler", "en çok"];

pub(crate) fn rank_for(title: &str, kind: HomeRowKind, index: usize) -> Option<usize> {
    let numbers = current_numbers()?;
    if kind != HomeRowKind::Poster || index >= numbers.count {
        return None;
    }
    let title = title.to_lowercase();
    (numbers.all_rows || TRENDING_WORDS.iter().any(|word| title.contains(word)))
        .then_some(index + 1)
}

fn beside_size(card_height: f32, scale: f32) -> f32 {
    card_height * 0.8 * scale
}

pub(crate) fn lead_width(card_height: f32, rank: Option<usize>) -> f32 {
    let (Some(rank), Some(numbers)) = (rank, current_numbers()) else {
        return 0.0;
    };
    if !numbers.beside {
        return 0.0;
    }
    let digits = rank.to_string().len() as f32;
    digits * beside_size(card_height, numbers.scale) * 0.5
}

fn paint_number(
    painter: &Painter,
    rank: usize,
    size: f32,
    anchor: Pos2,
    numbers: TopNumbers,
    accent: Color32,
    surface: Color32,
) {
    let tone = match numbers.color {
        NumberColor::White => Color32::WHITE,
        NumberColor::Gray => Color32::from_gray(150),
        NumberColor::Accent => accent,
    };
    let galley = painter.layout_no_wrap(rank.to_string(), FontId::proportional(size), tone);
    let pos = anchor - Vec2::new(galley.size().x, galley.size().y * 0.92);
    match numbers.style {
        NumberStyle::Outline => {
            let reach = (size * 0.03).max(1.5);
            for step in 0..12 {
                let angle = step as f32 * std::f32::consts::TAU / 12.0;
                let offset = Vec2::angled(angle) * reach;
                painter.galley_with_override_text_color(pos + offset, galley.clone(), tone);
            }
            painter.galley_with_override_text_color(pos, galley, surface);
        }
        NumberStyle::Solid => {
            painter.galley(pos, galley, tone);
        }
        NumberStyle::Ghost => {
            painter.galley_with_override_text_color(pos, galley, tone.gamma_multiply(0.35));
        }
    }
}

pub(crate) fn paint_beside(
    painter: &Painter,
    poster: Rect,
    rank: usize,
    accent: Color32,
    surface: Color32,
) {
    let Some(numbers) = current_numbers().filter(|numbers| numbers.beside) else {
        return;
    };
    let size = beside_size(poster.height(), numbers.scale);
    let anchor = Pos2::new(poster.left() + size * 0.06, poster.bottom());
    paint_number(painter, rank, size, anchor, numbers, accent, surface);
}

pub(crate) fn paint_overlay(
    painter: &Painter,
    poster: Rect,
    rank: usize,
    accent: Color32,
    surface: Color32,
) {
    let Some(numbers) = current_numbers().filter(|numbers| !numbers.beside) else {
        return;
    };
    let size = poster.height() * 0.34 * numbers.scale;
    let painter = painter.with_clip_rect(poster.intersect(painter.clip_rect()));
    let anchor = Pos2::new(
        poster.left() + size * 0.2 + size * 0.5 * rank.to_string().len() as f32,
        poster.bottom() - size * 0.05,
    );
    paint_number(&painter, rank, size, anchor, numbers, accent, surface);
}

impl super::super::SettingsModel {
    pub fn top_numbers(&self) -> Option<TopNumbers> {
        if !self.bool_value("posterTopNumbers") {
            return None;
        }
        Some(TopNumbers {
            all_rows: self.str_value("posterTopNumbersRows") == Some("all"),
            count: self
                .str_value("posterTopNumbersCount")
                .and_then(|count| count.parse().ok())
                .unwrap_or(10),
            style: match self.str_value("posterTopNumbersStyle") {
                Some("solid") => NumberStyle::Solid,
                Some("ghost") => NumberStyle::Ghost,
                _ => NumberStyle::Outline,
            },
            color: match self.str_value("posterTopNumbersColor") {
                Some("gray") => NumberColor::Gray,
                Some("accent") => NumberColor::Accent,
                _ => NumberColor::White,
            },
            scale: match self.str_value("posterTopNumbersSize") {
                Some("small") => 0.8,
                Some("large") => 1.2,
                _ => 1.0,
            },
            beside: self.str_value("posterTopNumbersPosition") != Some("overlay"),
        })
    }
}
