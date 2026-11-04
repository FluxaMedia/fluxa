use crate::ids::IdSet;
use crate::idtables::Resolver;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnilistTarget {
    pub media_id: u64,
    pub episode: Option<u32>,
}

fn number(part: Option<&str>) -> Option<u32> {
    part?.parse().ok()
}

pub fn anilist_target(resolver: &Resolver, item_id: &str, is_movie: bool) -> Option<AnilistTarget> {
    let ids = IdSet::from_prefixed(item_id);
    let tail: Vec<&str> = item_id.split(':').skip(1).collect();
    let tail = if ids.imdb.is_some() {
        &tail[..]
    } else {
        tail.get(1..).unwrap_or(&[])
    };
    let (season, episode) = match tail {
        [season, episode] => (number(Some(season)), number(Some(episode))),
        [episode] => (None, number(Some(episode))),
        _ => (None, None),
    };
    let episode = if is_movie { None } else { episode };

    let anime_native =
        ids.anilist.is_some() || ids.kitsu.is_some() || ids.mal.is_some() || ids.anidb.is_some();
    if anime_native {
        let media_id = resolver.resolve_all(ids, false).ids.anilist?;
        return Some(AnilistTarget { media_id, episode });
    }

    let resolved = resolver.resolve_all(ids.clone(), false).ids;
    let lists = resolver.anime_lists.as_ref();
    let tvdb = ids.tvdb.or(resolved.tvdb).or_else(|| {
        let entry = lists?.by_anidb(resolved.anidb?)?;
        entry.tvdb
    });
    let entries = lists.map(|lists| match (tvdb, ids.tmdb, &ids.imdb) {
        (Some(tvdb), _, _) => lists.by_tvdb(tvdb),
        (_, Some(tmdb), _) => lists.by_tmdb_tv(tmdb),
        (_, _, Some(imdb)) => lists.by_imdb(imdb),
        _ => Vec::new(),
    });
    let from_lists = |anidb: u64, episode: Option<u32>| {
        let given = IdSet {
            anidb: Some(anidb),
            ..IdSet::default()
        };
        let media_id = resolver.resolve_all(given, false).ids.anilist?;
        Some(AnilistTarget { media_id, episode })
    };
    if is_movie {
        let anidb = entries
            .as_ref()
            .and_then(|entries| entries.first())
            .map(|entry| entry.anidb)
            .or(resolved.anidb)?;
        return from_lists(anidb, None);
    }
    let episode = episode?;
    let hit = tvdb
        .zip(lists)
        .and_then(|(tvdb, lists)| lists.tvdb_to_anidb(tvdb, season.unwrap_or(1), episode));
    match hit {
        Some(hit) => from_lists(hit.anidb, Some(hit.episode)),
        None if season.unwrap_or(1) <= 1 => Some(AnilistTarget {
            media_id: resolved.anilist?,
            episode: Some(episode),
        }),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anime_lists::{AnimeEntry, AnimeLists, DefaultSeason};
    use crate::idtables::AnimeIdTable;

    fn entry(anidb: u64, season: u32) -> AnimeEntry {
        AnimeEntry {
            anidb,
            tvdb: Some(900),
            default_season: DefaultSeason::Number(season),
            episode_offset: 0,
            tmdb_tv: None,
            tmdb_movie: None,
            imdb: Vec::new(),
            name: String::new(),
            mappings: Vec::new(),
        }
    }

    fn resolver() -> Resolver {
        let fribb = r#"[
            {"anidb_id": 11, "anilist_id": 101, "mal_id": 1001, "kitsu_id": 51,
             "imdb_id": "tt0000900"},
            {"anidb_id": 12, "anilist_id": 102, "mal_id": 1002}
        ]"#;
        Resolver {
            anime: Some(AnimeIdTable::from_fribb(fribb).unwrap()),
            anime_lists: Some(AnimeLists::from_entries(vec![entry(11, 1), entry(12, 2)])),
            ..Resolver::default()
        }
    }

    #[test]
    fn anilist_ids_pass_through_with_their_episode() {
        let target = anilist_target(&resolver(), "anilist:7:1:9", false).unwrap();
        assert_eq!(
            target,
            AnilistTarget {
                media_id: 7,
                episode: Some(9)
            }
        );
    }

    #[test]
    fn kitsu_and_mal_ids_map_through_the_table() {
        let kitsu = anilist_target(&resolver(), "kitsu:51:3", false).unwrap();
        assert_eq!(
            kitsu,
            AnilistTarget {
                media_id: 101,
                episode: Some(3)
            }
        );
        let mal = anilist_target(&resolver(), "mal:1002:4", false).unwrap();
        assert_eq!(
            mal,
            AnilistTarget {
                media_id: 102,
                episode: Some(4)
            }
        );
    }

    #[test]
    fn imdb_seasons_land_on_their_own_anilist_entry() {
        let first = anilist_target(&resolver(), "tt0000900:1:5", false).unwrap();
        assert_eq!(first.media_id, 101);
        let second = anilist_target(&resolver(), "tt0000900:2:5", false).unwrap();
        assert_eq!(
            second,
            AnilistTarget {
                media_id: 102,
                episode: Some(5)
            }
        );
    }

    #[test]
    fn unknown_titles_resolve_to_nothing() {
        assert!(anilist_target(&resolver(), "tt9999999:1:1", false).is_none());
        assert!(anilist_target(&resolver(), "kitsu:99999:1", false).is_none());
    }
}
