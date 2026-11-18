use super::*;
use crate::dp::Dp;
use crate::flex;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterSpan {
    pub start_fraction: f32,
    pub end_fraction: f32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SegmentSpan {
    pub start_fraction: f32,
    pub end_fraction: f32,
    pub kind: SkipKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipKind {
    Intro,
    Recap,
    Outro,
}

impl SkipKind {
    pub(crate) fn band(self) -> &'static str {
        match self {
            SkipKind::Intro => "#e6bf40",
            SkipKind::Recap => "#6fa8dc",
            SkipKind::Outro => "#c27ba0",
        }
    }

    pub(crate) fn label_key(self) -> &'static str {
        match self {
            SkipKind::Intro => "player.skip_intro",
            SkipKind::Recap => "player.skip_recap",
            SkipKind::Outro => "player.skip_outro",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkipCard {
    pub kind: SkipKind,
    pub seek_to: f64,
    pub progress: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NextEpisodeCard {
    pub label: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub countdown: Option<u32>,
}
