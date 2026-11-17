use super::*;
use crate::flex::Insets;
use crate::gfx::Ui;

#[derive(Clone, Debug)]
pub struct ExpandedPoster {
    pub node: u64,
    pub id: String,
    pub raw: serde_json::Value,
    pub trailer: bool,
}

pub fn expanded_poster(context: &crate::gfx::Context) -> Option<ExpandedPoster> {
    context.data(|data| data.get_temp(Id::new("fluxa-expanded-poster")))
}

pub fn set_episode_details(
    context: &crate::gfx::Context,
    details: std::sync::Arc<std::collections::HashMap<String, serde_json::Value>>,
) {
    context.data_mut(|data| data.insert_temp(Id::new("fluxa-episode-details"), details));
}


pub fn previewed_poster(context: &crate::gfx::Context) -> Option<ExpandedPoster> {
    context.data(|data| data.get_temp(Id::new("fluxa-previewed-poster")))
}

#[derive(Clone, Default)]
struct ExpandState {
    focus: Option<(u64, f64)>,
    progress: Vec<(u64, f32)>,
}



pub(crate) struct PosterHover<'a> {
    pub screen: Rect,
    pub language: &'a str,
    pub delay: f64,
    pub resume: Option<&'a HomeCard>,
    pub trailer: Option<&'a HeroTrailer>,
    pub metrics: UiMetrics,
}

