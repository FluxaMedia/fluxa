use super::*;

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
