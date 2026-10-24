use fluxa_core::player::stream_policy;

use crate::{RendererState, Route};

pub(crate) fn tick(state: &mut RendererState) {
    let wanted = state.route == Route::Detail || state.detail_modal;
    if !wanted || state.player.is_some() || !state.settings.bool_value("prefetchDetailStreams") {
        return;
    }
    let detail = &state.detail;
    if detail.id.is_empty() || detail.is_loading {
        return;
    }
    let series = matches!(detail.content_type.as_str(), "series" | "tv" | "show");
    if series && detail.episodes.is_empty() {
        return;
    }
    let first = detail
        .episodes
        .iter()
        .find(|episode| episode.season > 0)
        .or(detail.episodes.first())
        .map(|episode| episode.id.clone());
    let item = crate::actions::playback_item(&detail.item, detail.resume.as_ref(), first);
    let profile = state
        .core_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.pointer("/profile/active"));
    if stream_policy::resumes_saved_stream(&item, profile) {
        return;
    }
    let video = item
        .get("lastVideoId")
        .and_then(|id| id.as_str())
        .unwrap_or(&detail.id);
    let key = format!("{}|{video}", detail.id);
    if state.prefetched.as_deref() == Some(key.as_str()) {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    session.executor().prefetch_direct_streams(item);
    state.prefetched = Some(key);
}
