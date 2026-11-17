use super::*;
use crate::dp::Dp;
use crate::kit;
use std::cell::{Cell, RefCell};


mod episodes;







#[derive(Clone, Copy)]
pub(crate) enum Artwork {
    Top(Vec2),
    Circle(f32),
}

impl Artwork {
    pub(crate) fn focus(self, tile: Rect) -> Rect {
        match self {
            Artwork::Top(size) => crate::flex::anchored(tile, Align2::LEFT_TOP, size, 0.0),
            Artwork::Circle(diameter) => {
                crate::flex::anchored(tile, Align2::CENTER_TOP, Vec2::splat(diameter), 0.0)
            }
        }
    }
}

pub(crate) struct TileHit {
    pub rect: Rect,
    clicked: bool,
}

impl TileHit {
    pub fn clicked(&self) -> bool {
        self.clicked
    }
}

pub(crate) fn tile(
    ui: &mut crate::gfx::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    size: Vec2,
    sense: Sense,
    add: impl FnOnce(&mut crate::gfx::Ui, Rect),
) -> TileHit {
    let rect = ui.next_slot(size);
    if !ui.is_rect_visible(rect) {
        ui.advance_cursor_after_rect(rect);
        return TileHit {
            rect,
            clicked: false,
        };
    }
    kit::at(ui, rect, |ui| add(ui, rect));
    ui.allocate_rect(rect, Sense::hover());
    let clicked = ui.interact(rect, Id::new(id), sense).clicked();
    TileHit { rect, clicked }
}



fn trailer_kind_label(kind: &str, language: &str) -> String {
    let key = match kind {
        "Trailer" => "detail.trailer_kind.trailer",
        "Teaser" => "detail.trailer_kind.teaser",
        "Clip" => "detail.trailer_kind.clip",
        "Featurette" => "detail.trailer_kind.featurette",
        "Behind the Scenes" => "detail.trailer_kind.behind_the_scenes",
        "Bloopers" => "detail.trailer_kind.bloopers",
        _ => return kind.to_owned(),
    };
    localized(key, language)
}







pub fn detail_data(detail: &DetailModel) -> serde_json::Value {
    let language = if detail.language.is_empty() {
        "en"
    } else {
        detail.language.as_str()
    };
    let t = |key: &str| crate::localized(key, language);
    let selected = detail.current_season();
    let series = detail.is_series();
    let season_episodes: Vec<_> = detail
        .episodes
        .iter()
        .enumerate()
        .filter(|(_, episode)| episode.season == selected)
        .collect();
    let first = season_episodes
        .first()
        .map(|(_, episode)| (episode.season, episode.number, Some(episode.title.as_str())));
    let play_label = crate::play_label(language, detail.resume.as_ref(), first.filter(|_| series));
    let seasons: Vec<_> = detail
        .seasons()
        .into_iter()
        .map(|season| {
            serde_json::json!({
                "id": season.to_string(),
                "key": "detail:season",
                "label": if season == 0 {
                    t("auto.specials")
                } else {
                    t("format.season_number").replacen("%s", &season.to_string(), 1)
                },
                "selected": season == selected,
            })
        })
        .collect();
    let episodes: Vec<_> = season_episodes
        .iter()
        .map(|(index, episode)| {
            let view = detail.episode_view(episode);
            serde_json::json!({
                "index": index.to_string(),
                "label": format!("S{}E{}", episode.season, episode.number),
                "title": view.title,
                "overview": view.overview.unwrap_or_default(),
                "thumb": if view.hide_still { None } else { episode.thumbnail.clone() },
                "watched": episode.watched,
                "progress": episode.progress,
            })
        })
        .collect();
    let people = |list: &[DetailCastMember]| -> Vec<serde_json::Value> {
        list.iter()
            .enumerate()
            .map(|(index, person)| {
                serde_json::json!({
                    "index": index.to_string(),
                    "name": person.name,
                    "role": person.role,
                    "photo": person.photo,
                })
            })
            .collect()
    };
    let kinds = detail.trailer_kinds();
    let current = detail.current_trailer_kind();
    let trailer_kinds: Vec<_> = kinds
        .iter()
        .map(|kind| {
            serde_json::json!({
                "id": kind,
                "key": "detail:trailerKind",
                "label": trailer_kind_label(kind, language),
                "selected": Some(kind) == current.as_ref(),
            })
        })
        .collect();
    let trailers: Vec<_> = detail
        .trailer_cards
        .iter()
        .enumerate()
        .filter(|(_, trailer)| Some(&trailer.kind) == current.as_ref())
        .map(|(index, trailer)| {
            serde_json::json!({
                "index": index.to_string(),
                "title": trailer.title,
                "thumb": trailer.thumbnail,
            })
        })
        .collect();
    let ratings: Vec<_> = detail
        .ratings
        .iter()
        .enumerate()
        .map(|(index, (label, value))| {
            serde_json::json!({"index": index.to_string(), "label": label, "value": value, "logo": crate::rating_logo(label, value).unwrap_or("")})
        })
        .collect();
    let info: Vec<_> = detail
        .show_info
        .iter()
        .map(|(label, value)| serde_json::json!({"label": label, "value": value}))
        .collect();
    let shown = |section: DetailSection| {
        detail.section_order().contains(&section) && detail.has_content(section)
    };
    serde_json::json!({
        "loading": detail.is_loading,
        "error": detail.streams_error.as_ref().or(detail.error.as_ref()),
        "id": detail.id,
        "title": detail.title,
        "logo": detail.logo_url,
        "backdrop": detail.background_url,
        "trailerUrls": detail.trailers,
        "poster": detail.poster_url,
        "meta": detail.meta_line,
        "facts": detail.facts.join("  ·  "),
        "genres": detail.genres.join("  ·  "),
        "infoLines": detail.info_lines,
        "description": detail.description,
        "playLabel": play_label,
        "watchlistIcon": if detail.in_watchlist { "Check" } else { "Plus" },
        "completed": detail.completed,
        "dropped": detail.dropped,
        "favorite": detail.favorite,
        "favoriteIcon": if detail.favorite { "HeartFilled" } else { "Heart" },
        "favoritesEnabled": detail.favorites_enabled,
        "series": series,
        "shuffle": detail.episodes.len() > 1,
        "seasons": seasons,
        "multiSeason": detail.seasons().len() > 1,
        "episodes": episodes,
        "cast": people(
            &detail
                .cast
                .iter()
                .chain(&detail.crew)
                .cloned()
                .collect::<Vec<_>>(),
        ),
        "titles": {
            "episodes": t("auto.episodes"),
            "cast": t(DetailSection::Cast.title_key()),
            "trailers": t(DetailSection::Trailers.title_key()),
            "info": t(DetailSection::ShowInfo.title_key()),
            "similar": t(DetailSection::Recommendations.title_key()),
            "franchise": t(DetailSection::Franchise.title_key()),
        },
        "trailerKinds": trailer_kinds,
        "trailers": trailers,
        "ratings": ratings,
        "info": info,
        "similar": detail.similar,
        "franchise": detail.franchise,
        "show": {
            "episodes": series && shown(DetailSection::Episodes),
            "cast": shown(DetailSection::Cast),
            "trailers": shown(DetailSection::Trailers),
            "info": shown(DetailSection::ShowInfo),
            "similar": !detail.similar.is_empty() && shown(DetailSection::Recommendations),
            "franchise": !detail.franchise.is_empty() && shown(DetailSection::Franchise),
        },
    })
}
