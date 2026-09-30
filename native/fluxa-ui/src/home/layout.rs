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
}

/// consume the same responsive rectangles as the painter.
pub fn home_layout(viewport: Viewport, home: &HomeModel) -> HomeLayout {
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let metrics = home_metrics(viewport);
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let mut layout = HomeLayout::default();
    if !compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    let hero_height = home_hero_height(viewport);
    if show_hero {
        let play_y = hero_height - if compact { 92.0 } else { 194.0 } - home.scroll_offset;
        let size = Vec2::new(
            if compact {
                108.0
            } else {
                (metrics.horizontal_card_width * 0.46).max(160.0)
            },
            if compact { 42.0 } else { 50.0 },
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
    let row_start = home_row_start(viewport, metrics, hero_height, show_hero);
    let mut flat = 0;
    let mut row_y = row_start;
    for (row_index, (_, cards, kind)) in home
        .content_rows_with_kind()
        .into_iter()
        .enumerate()
        .take(64)
    {
        let (_, _, body_height) = home_row_dimensions(metrics, kind);
        let row_height = home_row_heading_height(metrics, viewport.is_tv()) + body_height;
        let visible_y = row_y - home.scroll_offset;
        if visible_y + row_height < 0.0 || visible_y > viewport.height {
            row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
            flat += cards.len();
            continue;
        }
        let row_scroll = home
            .row_scroll_offsets
            .get(row_index)
            .copied()
            .unwrap_or(0.0);
        let mut card_x = margin - row_scroll;
        for card in cards {
            let (item_width, item_height) = home_card_dimensions(metrics, card, kind);
            layout.focusable.push((
                NODE_CARD_BASE + flat as u64,
                Rect::from_min_size(
                    Pos2::new(
                        card_x,
                        visible_y + home_row_heading_height(metrics, viewport.is_tv()),
                    ),
                    Vec2::new(item_width, item_height),
                ),
            ));
            card_x += item_width + metrics.horizontal_spacing;
            flat += 1;
        }
        row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    if compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    layout
}

/// The shelf under the given touch point, if it is over a row's cards rather
/// than the hero or the space between shelves.
pub fn home_row_at_y(viewport: Viewport, home: &HomeModel, y: f32) -> Option<usize> {
    let metrics = home_metrics(viewport);
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let hero_height = home_hero_height(viewport);
    let mut row_y = home_row_start(viewport, metrics, hero_height, show_hero);
    let content_y = y + home.scroll_offset;
    for (index, (_, cards, kind)) in home.content_rows_with_kind().into_iter().enumerate() {
        let body_height = home_row_body_height(metrics, cards, kind);
        let heading_height = home_row_heading_height(metrics, viewport.is_tv());
        if content_y >= row_y + heading_height && content_y <= row_y + heading_height + body_height
        {
            return Some(index);
        }
        row_y += heading_height + body_height + metrics.section_gap + metrics.vertical_spacing;
        if cards.is_empty() {
            continue;
        }
    }
    None
}

/// Maximum horizontal drag offset for a Home shelf.
pub fn home_row_scroll_max(viewport: Viewport, home: &HomeModel, row_index: usize) -> f32 {
    let Some((_, cards, kind)) = home.content_rows_with_kind().nth(row_index) else {
        return 0.0;
    };
    let metrics = home_metrics(viewport);
    home_row_scroll_max_for_cards(viewport, metrics, cards, kind)
}

pub(crate) fn home_row_scroll_max_for_cards(
    viewport: Viewport,
    metrics: UiMetrics,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    let margin = if viewport.is_compact() {
        metrics.page_padding
    } else if viewport.is_tv() {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let content_width = cards
        .iter()
        .map(|card| home_card_dimensions(metrics, card, kind).0 + metrics.horizontal_spacing)
        .sum::<f32>()
        - metrics.horizontal_spacing;
    (content_width - (viewport.width - margin * 2.0)).max(0.0)
}

/// Maximum vertical offset for Home's touch/scroll adapter.
pub fn home_scroll_max(viewport: Viewport, home: &HomeModel) -> f32 {
    let compact = viewport.is_compact();
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let metrics = home_metrics(viewport);
    let hero_height = home_hero_height(viewport);
    let mut content_height = home_row_start(viewport, metrics, hero_height, show_hero);
    for (_, cards, kind) in home.content_rows_with_kind().into_iter().take(64) {
        content_height += home_row_heading_height(metrics, viewport.is_tv())
            + home_row_body_height(metrics, cards, kind)
            + metrics.section_gap
            + metrics.vertical_spacing;
    }
    content_height -= metrics.section_gap + metrics.vertical_spacing;
    let bottom_navigation_reserve = if compact {
        mobile_nav_reserve(viewport)
    } else {
        0.0
    };
    (content_height - viewport.height + bottom_navigation_reserve + metrics.page_padding).max(0.0)
}
