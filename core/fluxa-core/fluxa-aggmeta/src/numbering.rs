use std::collections::{BTreeMap, HashMap};

use crate::types::Meta;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeasonNumbering {
    #[default]
    Sequential,
    Years,
}

impl SeasonNumbering {
    pub fn as_str(self) -> &'static str {
        match self {
            SeasonNumbering::Sequential => "sequential",
            SeasonNumbering::Years => "years",
        }
    }

    pub fn parse(text: &str) -> Self {
        if text.eq_ignore_ascii_case("years") {
            SeasonNumbering::Years
        } else {
            SeasonNumbering::Sequential
        }
    }
}

pub fn resolve_numbering(
    show_key: &str,
    per_show: &HashMap<String, SeasonNumbering>,
    global: SeasonNumbering,
) -> SeasonNumbering {
    per_show.get(show_key).copied().unwrap_or(global)
}

pub fn needs_refetch(meta: &Meta, wanted: SeasonNumbering) -> bool {
    wanted == SeasonNumbering::Years && meta.season_numbering.is_none()
}

pub fn project(meta: &mut Meta, numbering: SeasonNumbering) -> HashMap<String, String> {
    let mut renamed = HashMap::new();
    let Some(videos) = meta.videos.as_mut() else {
        return renamed;
    };
    match numbering {
        SeasonNumbering::Sequential => {
            for v in videos.iter_mut() {
                v.tvdb_season = None;
            }
        }
        SeasonNumbering::Years => {
            let mut season_map: BTreeMap<u32, u32> = BTreeMap::new();
            for v in videos.iter_mut() {
                let Some(tvdb) = v.tvdb_season else { continue };
                let sequential = v.season;
                season_map.insert(sequential, tvdb);
                let parts: Vec<&str> = v.id.split(':').collect();
                let id_matches = match parts.as_slice() {
                    [show, s, e] => {
                        (show.starts_with("tt") || show.starts_with("tvdb"))
                            && s.parse::<u32>().ok() == Some(sequential)
                            && e.parse::<u32>().ok() == Some(v.episode)
                    }
                    [kind, n, s, e] => {
                        *kind == "tvdb"
                            && n.parse::<u64>().is_ok()
                            && s.parse::<u32>().ok() == Some(sequential)
                            && e.parse::<u32>().ok() == Some(v.episode)
                    }
                    _ => false,
                };
                if id_matches {
                    let old = v.id.clone();
                    let (head, _) = old.rsplit_once(':').unwrap_or((&old, ""));
                    let (show, _) = head.rsplit_once(':').unwrap_or((head, ""));
                    v.id = format!("{show}:{tvdb}:{}", v.episode);
                    renamed.insert(old, v.id.clone());
                }
                v.season = tvdb;
                v.tvdb_season = None;
            }
            if let Some(default) = meta.behavior_hints.default_video_id.clone() {
                if let Some(new) = renamed.get(&default) {
                    meta.behavior_hints.default_video_id = Some(new.clone());
                }
            }
            let posters = std::mem::take(&mut meta.app_extras.season_posters);
            meta.app_extras.season_posters = posters
                .into_iter()
                .map(|(season, url)| (season_map.get(&season).copied().unwrap_or(season), url))
                .collect();
        }
    }
    meta.season_numbering = Some(numbering.as_str().to_string());
    renamed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Video;

    fn show() -> Meta {
        let video = |id: &str, season: u32, episode: u32, tvdb: u32| Video {
            id: id.into(),
            season,
            episode,
            tvdb_season: Some(tvdb),
            ..Default::default()
        };
        let mut meta = Meta::default();
        meta.videos = Some(vec![
            video("tt100:1:1", 1, 1, 2019),
            video("tt100:1:2", 1, 2, 2019),
            video("tmdb:5:2:1", 2, 1, 2020),
        ]);
        meta.behavior_hints.default_video_id = Some("tt100:1:1".into());
        meta.app_extras.season_posters.insert(1, "/p1".into());
        meta.app_extras.season_posters.insert(2, "/p2".into());
        meta
    }

    #[test]
    fn sequential_drops_the_private_season() {
        let mut meta = show();
        assert!(project(&mut meta, SeasonNumbering::Sequential).is_empty());
        let videos = meta.videos.unwrap();
        assert!(videos.iter().all(|v| v.tvdb_season.is_none()));
        assert_eq!(videos[0].season, 1);
    }

    #[test]
    fn years_rewrites_matching_ids_and_default_video() {
        let mut meta = show();
        let renamed = project(&mut meta, SeasonNumbering::Years);
        assert_eq!(
            renamed.get("tt100:1:1").map(String::as_str),
            Some("tt100:2019:1")
        );
        assert_eq!(
            meta.behavior_hints.default_video_id.as_deref(),
            Some("tt100:2019:1")
        );
        let videos = meta.videos.unwrap();
        assert_eq!(videos[0].season, 2019);
    }

    #[test]
    fn foreign_id_shapes_keep_their_season_in_the_id() {
        let mut meta = show();
        project(&mut meta, SeasonNumbering::Years);
        let video = &meta.videos.unwrap()[2];
        assert_eq!(video.id, "tmdb:5:2:1");
        assert_eq!(video.season, 2020);
    }

    #[test]
    fn season_posters_are_rekeyed() {
        let mut meta = show();
        project(&mut meta, SeasonNumbering::Years);
        let keys: Vec<u32> = meta.app_extras.season_posters.keys().copied().collect();
        assert_eq!(keys, vec![2019, 2020]);
    }

    #[test]
    fn list_without_marker_is_refetched_under_years() {
        let meta = show();
        assert!(needs_refetch(&meta, SeasonNumbering::Years));
        assert!(!needs_refetch(&meta, SeasonNumbering::Sequential));
    }
}
