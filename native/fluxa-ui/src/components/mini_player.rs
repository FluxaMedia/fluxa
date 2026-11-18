use super::*;

const MINI_RECT: &str = "fluxa-mini-player-rect";

pub(crate) fn mark_mini_player(context: &crate::gfx::Context, rect: Rect) {
    let pass = context.cumulative_pass_nr();
    context.data_mut(|data| data.insert_temp(Id::new(MINI_RECT), (pass, rect)));
}

pub(crate) fn over_mini_player(context: &crate::gfx::Context, pos: Pos2) -> bool {
    let now = context.cumulative_pass_nr();
    context
        .data(|data| data.get_temp::<(u64, Rect)>(Id::new(MINI_RECT)))
        .is_some_and(|(pass, rect)| now.saturating_sub(pass) <= 2 && rect.contains(pos))
}
