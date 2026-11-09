use super::*;

#[derive(Clone, Debug)]
pub struct ChoiceRequest {
    pub key: &'static str,
    pub title: String,
    pub options: Vec<(String, String)>,
    pub selected: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerGesture {
    Volume(f32),
    VolumeSet(f32),
    Brightness(f32),
}

#[derive(Clone, Debug, Default)]
pub struct HomeLayout {
    pub choices: Vec<(u64, ChoiceRequest)>,
    pub focusable: Vec<(u64, Rect)>,
    pub neighbors: Vec<(u64, crate::Dir, u64)>,
    pub activated: Option<u64>,
    pub hovered: Option<(u64, Rect)>,
    pub text_input: Option<String>,
    pub text_input_node: Option<u64>,
    pub setting_change: Option<(String, serde_json::Value)>,
    pub filter_change: Option<(String, String)>,
    /// Core page requests generated when a shelf approaches its loaded end.
    pub load_more: Vec<serde_json::Value>,
    pub seek_to: Option<f64>,
    pub seek_hover: Option<f64>,
    pub filmstrip: Option<Vec<f64>>,
    pub player_gesture: Option<PlayerGesture>,
    pub player_speed_hold: bool,
    pub profiles: Option<ProfilesRequest>,
    pub welcome: Option<WelcomeRequest>,
    pub watch_together: Option<WatchTogetherRequest>,
    pub scroll_max: Option<f32>,
    pub short_step: Option<i32>,
}

/// consume the same responsive rectangles as the painter.
pub fn home_layout(viewport: Viewport, home: &HomeModel) -> HomeLayout {
    let compact = viewport.is_compact();
    let show_hero = home_shows_hero(home);
    let metrics = home_metrics(viewport);
    let margin = metrics.screen_margin;
    let mut layout = HomeLayout::default();
    if !compact {
        layout.focusable.extend(navigation_focus_rects(
            home_nav_viewport(viewport, home),
            metrics,
        ));
    }
    let hero_height = home_hero_height_for(viewport, home);
    let flow = home_flow(viewport, metrics, home);
    if show_hero && home.item_id.is_some() && !home_hero_pinned(viewport, home) {
        let play_y =
            flow.hero_top + hero_height - metrics.home_hero_bottom_inset - home.scroll_offset;
        let size = Vec2::new(
            if compact {
                108.0
            } else {
                (metrics.horizontal_card_width * 0.46).max(160.0)
            },
            metrics.button_height,
        );
        let play = Rect::from_min_size(
            Pos2::new(
                if compact {
                    (viewport.width - size.x) * 0.5
                } else {
                    margin
                },
                play_y,
            ),
            size,
        );
        layout.focusable.push((NODE_PLAY, play));
    }
    for shelf in home_shelves(metrics, home, flow.rows_top).take(64) {
        let (title, cards, kind) = (shelf.title, shelf.cards, shelf.kind);
        let row_height = shelf.height();
        let visible_y = shelf.top - home.scroll_offset;
        if visible_y + row_height < 0.0 || visible_y > viewport.height + row_height {
            continue;
        }
        let mut flat = shelf.first_card;
        let row_scroll = home
            .row_scroll_offsets
            .get(shelf.index)
            .copied()
            .unwrap_or(0.0);
        let origin = Pos2::new(margin - row_scroll, visible_y + shelf.heading);
        for rect in shelf_card_rects(metrics, title, kind, cards, origin, |_, width, _| width) {
            layout.focusable.push((NODE_CARD_BASE + flat as u64, rect));
            flat += 1;
        }
    }
    if compact {
        layout.focusable.extend(navigation_focus_rects(
            home_nav_viewport(viewport, home),
            metrics,
        ));
    }
    layout
}

pub fn home_row_of_card(home: &HomeModel, card: usize) -> Option<usize> {
    let mut start = 0;
    for (index, (_, cards, _)) in home.content_rows_with_kind().enumerate() {
        if card < start + cards.len() {
            return Some(index);
        }
        start += cards.len();
    }
    None
}

pub(crate) fn shelf_card_rects(
    metrics: UiMetrics,
    title: &str,
    kind: HomeRowKind,
    cards: &[HomeCard],
    origin: Pos2,
    widen: impl Fn(usize, f32, f32) -> f32,
) -> Vec<Rect> {
    let slots: Vec<crate::kit::Slot> = cards
        .iter()
        .enumerate()
        .map(|(index, card)| {
            let (width, height) = home_card_dimensions(metrics, card, kind);
            crate::kit::Slot::sized(widen(index, width, height), height)
                .lead(home_card_lead(title, kind, index, height))
        })
        .collect();
    crate::kit::solve(
        crate::kit::Flow::Row,
        metrics.horizontal_spacing,
        origin,
        Vec2::new(1.0e7, 1.0e4),
        &slots,
    )
}

pub(crate) fn home_row_scroll_max_for_cards(
    viewport: Viewport,
    metrics: UiMetrics,
    title: &str,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    let margin = metrics.screen_margin;
    let rects = shelf_card_rects(metrics, title, kind, cards, Pos2::ZERO, |_, width, _| width);
    let content_width = rects.last().map_or(0.0, |rect| rect.right());
    (content_width - (viewport.width - margin * 2.0)).max(0.0)
}
