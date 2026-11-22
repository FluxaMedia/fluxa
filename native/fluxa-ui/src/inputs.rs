use std::hash::{Hash, Hasher};

use crate::gfx::{Context, Id};

fn arc_id<T: Send + Sync + 'static>(context: &Context, id: &'static str) -> usize {
    context
        .data(|data| data.get_temp::<std::sync::Arc<T>>(Id::new(id)))
        .map_or(0, |value| {
            std::sync::Arc::as_ptr(&value) as *const u8 as usize
        })
}

fn debug_of<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    context: &Context,
    id: &'static str,
) -> String {
    context
        .data(|data| data.get_temp::<T>(Id::new(id)))
        .map_or_else(String::new, |value| format!("{value:?}"))
}

pub(crate) fn fingerprint(context: &Context) -> u64 {
    let mut hasher = crate::fast::Fx::default();
    crate::sidebar::state_sig(&mut hasher);
    crate::navigation::state_sig(&mut hasher);
    crate::motion::state_sig(&mut hasher);
    crate::home_category::state_sig(&mut hasher);
    crate::components::glass_sig(&mut hasher);
    crate::metrics::state_sig(&mut hasher);
    crate::poster_overlay::state_sig(&mut hasher);
    crate::update::state_sig(&mut hasher);
    crate::poster_overlay::top_numbers_sig(&mut hasher);
    crate::strings_sig(&mut hasher);
    arc_id::<crate::poster_overlay::PosterOverlays>(context, "fluxa-poster-overlays")
        .hash(&mut hasher);
    arc_id::<crate::poster_overlay::Enrichment>(context, "fluxa-poster-enrichment")
        .hash(&mut hasher);
    arc_id::<crate::poster_overlay::PersonalIndex>(context, "fluxa-poster-personal")
        .hash(&mut hasher);
    arc_id::<Vec<(&'static str, crate::gfx::TextureId, crate::gfx::Vec2)>>(
        context,
        "fluxa-poster-logos",
    )
    .hash(&mut hasher);
    arc_id::<std::collections::HashMap<String, serde_json::Value>>(
        context,
        "fluxa-episode-details",
    )
    .hash(&mut hasher);
    debug_of::<Vec<(&'static str, f32)>>(context, "fluxa-scroll-strip-offsets").hash(&mut hasher);
    debug_of::<bool>(context, "fluxa-scroll-direct").hash(&mut hasher);
    debug_of::<crate::components::PendingField>(context, "fluxa-pending-field-text")
        .hash(&mut hasher);
    debug_of::<Option<crate::gfx::Rect>>(context, "fluxa-input-caret").hash(&mut hasher);
    debug_of::<f32>(context, "fluxa-keyboard-inset").hash(&mut hasher);
    debug_of::<bool>(context, "fluxa-screen-covered").hash(&mut hasher);
    debug_of::<bool>(context, "fluxa-ui-settled").hash(&mut hasher);
    debug_of::<bool>(context, "fluxa-detail-modal-open").hash(&mut hasher);
    debug_of::<crate::skin::PlayingVideo>(context, "skin-video-playing").hash(&mut hasher);
    debug_of::<crate::gfx::TextureId>(context, "skin-player-video").hash(&mut hasher);
    hasher.finish()
}
