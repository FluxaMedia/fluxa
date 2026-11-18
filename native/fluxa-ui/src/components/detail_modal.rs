use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailModalOutcome {
    Close,
    Play,
    Watchlist,
    OpenPage,
    Episode(usize),
    Similar(usize),
    SimilarPlay(usize),
    SimilarWatchlist(usize),
}

fn open_id() -> Id {
    Id::new("fluxa-detail-modal-open")
}

fn outcome_id() -> Id {
    Id::new("fluxa-detail-modal-outcome")
}

pub fn detail_modal_is_open(context: &crate::gfx::Context) -> bool {
    context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false)
}

fn covered_id() -> Id {
    Id::new("fluxa-screen-covered")
}

pub fn set_screen_covered(context: &crate::gfx::Context, covered: bool) {
    context.data_mut(|data| data.insert_temp(covered_id(), covered));
}

const SETTLED_ID: &str = "fluxa-ui-settled";

pub fn set_ui_settled(context: &crate::gfx::Context, settled: bool) {
    context.data_mut(|data| data.insert_temp(crate::gfx::Id::new(SETTLED_ID), settled));
}

pub(crate) fn ui_settled(context: &crate::gfx::Context) -> bool {
    context
        .data(|data| data.get_temp::<bool>(crate::gfx::Id::new(SETTLED_ID)))
        .unwrap_or(true)
}

pub(crate) fn screen_covered(context: &crate::gfx::Context) -> bool {
    detail_modal_is_open(context)
        || crate::gfx::Popup::is_any_open(context)
        || context
            .data(|data| data.get_temp::<bool>(covered_id()))
            .unwrap_or(false)
}

pub fn set_detail_modal_open(context: &crate::gfx::Context, open: bool) {
    let was = context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false);
    if was != open {
        crate::motion::reset(context, Id::new("fluxa-detail-modal-appear"));
    }
    context.data_mut(|data| data.insert_temp(open_id(), open));
}

pub fn push_detail_modal_outcome(context: &crate::gfx::Context, outcome: DetailModalOutcome) {
    context.data_mut(|data| data.insert_temp(outcome_id(), outcome));
}

pub fn take_detail_modal_outcome(context: &crate::gfx::Context) -> Option<DetailModalOutcome> {
    context.data_mut(|data| {
        let outcome = data.get_temp::<DetailModalOutcome>(outcome_id());
        data.remove::<DetailModalOutcome>(outcome_id());
        outcome
    })
}
