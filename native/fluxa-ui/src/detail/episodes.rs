use super::*;
use crate::kit;








pub(super) struct EpisodeContext<'a> {
    pub viewport: Viewport,
    pub detail: &'a DetailModel,
    pub episodes: &'a [(usize, &'a DetailEpisode)],
    pub season: i64,
    pub metrics: UiMetrics,
    pub margin: f32,
}



