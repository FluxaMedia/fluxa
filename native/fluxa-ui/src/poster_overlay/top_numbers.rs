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
