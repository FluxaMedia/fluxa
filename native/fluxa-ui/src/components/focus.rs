use super::*;

pub(crate) fn focus_ring(
    painter: &Painter,
    focusable: &[(u64, Rect)],
    focused: Option<u64>,
    viewport: Viewport,
    in_player: bool,
    metrics: UiMetrics,
) {
    use crate::*;
    let Some(node) = focused else {
        return;
    };
    if viewport.is_compact() || is_text_node(node) || (in_player && node == NODE_PLAYER_SEEK) {
        return;
    }
    if let Some((_, rect)) = focusable.iter().find(|(id, _)| *id == node) {
        let rect = rect.expand(metrics.focus_ring_expand);
        let radius = if in_player {
            if (NODE_PLAYER_SOURCE_BASE..NODE_PLAYER_PANEL_ROW_BASE).contains(&node) {
                metrics.card_radius + metrics.focus_ring_expand
            } else {
                rect.height().min(rect.width()) * 0.5
            }
        } else if is_pill_node(node) {
            rect.height() * 0.5
        } else {
            metrics.focus_ring_radius
        };
        painter.rect_stroke(
            rect,
            radius,
            egui::Stroke::new(metrics.focus_ring_width, accent()),
            egui::StrokeKind::Outside,
        );
    }
}

pub(super) fn is_pill_node(node: u64) -> bool {
    use crate::*;
    crate::HomeCategory::from_node(node).is_some()
        || matches!(
            node,
            NODE_HOME
                | NODE_LIBRARY
                | NODE_DISCOVER
                | NODE_CALENDAR
                | NODE_PROFILE
                | NODE_SHORTS
                | NODE_PLAY
                | NODE_HERO_WATCHLIST
                | NODE_HOME_ADD_ADDONS
                | NODE_DETAIL_BACK
                | NODE_DETAIL_PLAY
                | NODE_DETAIL_WATCHLIST
                | NODE_DETAIL_COMPLETED
                | NODE_DETAIL_DROPPED
                | NODE_DETAIL_FAVORITE
                | NODE_DETAIL_SHUFFLE
                | NODE_SHORTS_PLAY
                | NODE_SHORTS_WATCHLIST
                | NODE_SHORTS_INFO
                | NODE_SETTINGS_BACK
                | NODE_SETTINGS_SWITCH_PROFILE
                | NODE_SETTINGS_LOG_OUT
                | NODE_FOLDER_BACK
        )
        || (NODE_PROFILES_GEAR..NODE_PROFILES_CELL_BASE + 200).contains(&node)
}

pub fn is_text_node(node: u64) -> bool {
    matches!(
        node,
        crate::NODE_LIBRARY_SEARCH
            | crate::NODE_DISCOVER_SEARCH
            | crate::NODE_SIDEBAR_SEARCH
            | crate::NODE_SETTINGS_SEARCH
            | crate::NODE_SETTINGS_ADDON_URL
            | crate::NODE_SETTINGS_PLUGIN_URL
            | crate::NODE_SETTINGS_STREAMKIT_ADDON_URL
            | crate::NODE_SETTINGS_WATCH_CODE
            | crate::NODE_REPORT_TITLE
            | crate::NODE_ADDON_EDIT_NAME
            | crate::NODE_REPORT_BODY
    ) || crate::poster_field(node).is_some()
        || crate::server_input(node).is_some()
        || crate::badge_input(node).is_some()
        || crate::plugin_config_input(node).is_some()
}
