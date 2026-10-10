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
    pub activated: Option<u64>,
    pub text_input: Option<String>,
    pub text_input_node: Option<u64>,
    pub setting_change: Option<(String, serde_json::Value)>,
    pub filter_change: Option<(String, String)>,
    /// Core page requests generated when a shelf approaches its loaded end.
    pub load_more: Vec<serde_json::Value>,
    pub seek_to: Option<f64>,
    pub seek_hover: Option<f64>,
    pub player_gesture: Option<PlayerGesture>,
    pub player_speed_hold: bool,
    pub profiles: Option<ProfilesRequest>,
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
        layout
            .focusable
            .extend(navigation_focus_rects(home_nav_viewport(viewport, home), metrics));
    }
    let hero_height = home_hero_height_for(viewport, home);
    let flow = home_flow(viewport, metrics, home);
    if show_hero && home.item_id.is_some() && !home_hero_pinned(viewport, home) {
        let play_y = flow.hero_top + hero_height
            - if compact { 92.0 } else { 194.0 }
            - home.scroll_offset;
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
        let mut card_x = margin - row_scroll;
        for (index, card) in cards.iter().enumerate() {
            let (item_width, item_height) = home_card_dimensions(metrics, card, kind);
            let lead = home_card_lead(title, kind, index, item_height);
            layout.focusable.push((
                NODE_CARD_BASE + flat as u64,
                Rect::from_min_size(
                    Pos2::new(card_x + lead, visible_y + shelf.heading),
                    Vec2::new(item_width, item_height),
                ),
            ));
            card_x += lead + item_width + metrics.horizontal_spacing;
            flat += 1;
        }
    }
    if compact {
        layout
            .focusable
            .extend(navigation_focus_rects(home_nav_viewport(viewport, home), metrics));
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

/// The shelf under the given touch point, if it is over a row's cards rather
/// than the hero or the space between shelves.
pub fn home_row_at_y(viewport: Viewport, home: &HomeModel, y: f32) -> Option<usize> {
    let metrics = home_metrics(viewport);
    let content_y = y + home.scroll_offset;
    home_shelves(metrics, home, home_flow(viewport, metrics, home).rows_top)
        .find(|shelf| {
            let cards_top = shelf.top + shelf.heading;
            content_y >= cards_top && content_y <= cards_top + shelf.body
        })
        .map(|shelf| shelf.index)
}

/// Maximum horizontal drag offset for a Home shelf.
pub fn home_row_scroll_max(viewport: Viewport, home: &HomeModel, row_index: usize) -> f32 {
    let Some((title, cards, kind)) = home.content_rows_with_kind().nth(row_index) else {
        return 0.0;
    };
    let metrics = home_metrics(viewport);
    let (width, height, _) = home_row_dimensions(metrics, kind);
    let expanded_here = home.expanded_posters
        && kind == HomeRowKind::Poster
        && home
            .expanded_node
            .and_then(|node| node.checked_sub(NODE_CARD_BASE))
            .is_some_and(|card| home_row_of_card(home, card as usize) == Some(row_index));
    let extra = if expanded_here {
        (height * 16.0 / 9.0 - width).max(0.0)
    } else {
        0.0
    };
    home_row_scroll_max_for_cards(viewport, metrics, title, cards, kind) + extra
}

pub(crate) fn home_row_scroll_max_for_cards(
    viewport: Viewport,
    metrics: UiMetrics,
    title: &str,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    let margin = metrics.screen_margin;
    let content_width = cards
        .iter()
        .enumerate()
        .map(|(index, card)| {
            let (width, height) = home_card_dimensions(metrics, card, kind);
            home_card_lead(title, kind, index, height) + width + metrics.horizontal_spacing
        })
        .sum::<f32>()
        - metrics.horizontal_spacing;
    (content_width - (viewport.width - margin * 2.0)).max(0.0)
}

/// Maximum vertical offset for Home's touch/scroll adapter.
pub fn home_scroll_max(viewport: Viewport, home: &HomeModel) -> f32 {
    let compact = viewport.is_compact();
    let show_hero = home_shows_hero(home);
    let metrics = home_metrics(viewport);
    let hero_height = home_hero_height_for(viewport, home);
    let mut content_height = home_flow(viewport, metrics, home).rows_top;
    if home_hero_pinned(viewport, home) {
        let last_row_top = home_shelves(metrics, home, content_height)
            .take(64)
            .last()
            .map_or(0.0, |shelf| shelf.top);
        let first_row_top = home_row_start(viewport, metrics, hero_height, show_hero);
        return (last_row_top - first_row_top).max(0.0);
    }
    if let Some(last) = home_shelves(metrics, home, content_height).take(64).last() {
        content_height = last.top + last.height();
    } else {
        content_height -= shelf_gap(metrics);
    }
    let bottom_navigation_reserve = if compact {
        mobile_nav_reserve(viewport)
    } else {
        0.0
    };
    (content_height - viewport.height + bottom_navigation_reserve + metrics.page_padding).max(0.0)
}
