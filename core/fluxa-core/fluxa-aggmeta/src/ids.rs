use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::Meta;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IdSet {
    pub imdb: Option<String>,
    pub tmdb: Option<u64>,
    pub tvdb: Option<u64>,
    pub tvmaze: Option<u64>,
    pub mal: Option<u64>,
    pub kitsu: Option<u64>,
    pub anilist: Option<u64>,
    pub anidb: Option<u64>,
}

#[allow(clippy::expect_used)]
static UPNEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:(?:mdblist|simkl|pmdb_resume)_)?upnext_(?P<c>tt\d+|tmdb:\d+|tvdb:\d+)_(?:S\d+E\d+|trakt\d+|unknown)$",
    )
    .expect("static pattern")
});

pub fn reduce_upnext(id: &str) -> Option<String> {
    UPNEXT
        .captures(id)
        .and_then(|c| c.name("c"))
        .map(|m| m.as_str().to_string())
}

pub fn normalise_request_id(id: &str) -> String {
    let id = id.split(['?', '#']).next().unwrap_or(id);
    reduce_upnext(id).unwrap_or_else(|| id.to_string())
}

fn imdb_prefix(text: &str) -> Option<String> {
    let head = text.split(':').next()?;
    let digits = head.strip_prefix("tt")?;
    (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())).then(|| head.to_string())
}

impl IdSet {
    pub fn from_prefixed(id: &str) -> Self {
        let mut out = IdSet::default();
        if let Some(imdb) = imdb_prefix(id) {
            out.imdb = Some(imdb);
            return out;
        }
        let mut parts = id.split(':');
        let (Some(prefix), Some(num)) = (parts.next(), parts.next()) else {
            return out;
        };
        let Ok(n) = num.parse::<u64>() else {
            return out;
        };
        match prefix {
            "tmdb" => out.tmdb = Some(n),
            "tvdb" => out.tvdb = Some(n),
            "tvmaze" => out.tvmaze = Some(n),
            "kitsu" => out.kitsu = Some(n),
            "mal" => out.mal = Some(n),
            "anilist" => out.anilist = Some(n),
            "anidb" => out.anidb = Some(n),
            _ => {}
        }
        out
    }

    pub fn from_meta(meta: &Meta) -> Self {
        let mut out = IdSet::from_prefixed(&meta.id);
        macro_rules! fill {
            ($slot:ident, $src:expr) => {
                if out.$slot.is_none() {
                    out.$slot = $src.clone();
                }
            };
        }
        fill!(imdb, meta.p_imdb);
        fill!(tmdb, meta.p_tmdb);
        fill!(tvdb, meta.p_tvdb);
        fill!(mal, meta.p_mal);
        fill!(kitsu, meta.p_kitsu);
        fill!(anilist, meta.p_anilist);
        fill!(anidb, meta.p_anidb);
        if let Some(imdb) = meta.imdb_id.clone().filter(|i| !i.is_empty()) {
            out.imdb = Some(imdb);
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        *self == IdSet::default()
    }

    pub fn is_anime_id(&self) -> bool {
        self.mal.is_some() || self.kitsu.is_some() || self.anilist.is_some() || self.anidb.is_some()
    }

    pub fn canonical(&self) -> Option<String> {
        self.imdb
            .clone()
            .or_else(|| self.tmdb.map(|n| format!("tmdb:{n}")))
            .or_else(|| self.tvdb.map(|n| format!("tvdb:{n}")))
            .or_else(|| self.kitsu.map(|n| format!("kitsu:{n}")))
            .or_else(|| self.mal.map(|n| format!("mal:{n}")))
            .or_else(|| self.anilist.map(|n| format!("anilist:{n}")))
            .or_else(|| self.anidb.map(|n| format!("anidb:{n}")))
    }

    pub fn shares_any(&self, other: &IdSet) -> bool {
        fn same<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> bool {
            matches!((a, b), (Some(x), Some(y)) if x == y)
        }
        same(&self.imdb, &other.imdb)
            || same(&self.tmdb, &other.tmdb)
            || same(&self.tvdb, &other.tvdb)
            || same(&self.tvmaze, &other.tvmaze)
            || same(&self.mal, &other.mal)
            || same(&self.kitsu, &other.kitsu)
            || same(&self.anilist, &other.anilist)
            || same(&self.anidb, &other.anidb)
    }
}

pub fn episode_id(show: &str, season: u32, episode: u32) -> String {
    format!("{show}:{season}:{episode}")
}

pub fn anime_episode_id(anime: &str, episode: u32) -> String {
    format!("{anime}:{episode}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upnext_reduces_to_canonical() {
        assert_eq!(
            reduce_upnext("mdblist_upnext_tt0944947_S01E02").as_deref(),
            Some("tt0944947")
        );
        assert_eq!(
            reduce_upnext("upnext_tmdb:1399_unknown").as_deref(),
            Some("tmdb:1399")
        );
        assert_eq!(
            reduce_upnext("simkl_upnext_tvdb:121361_trakt55").as_deref(),
            Some("tvdb:121361")
        );
    }

    #[test]
    fn upnext_rejects_other_shapes() {
        assert_eq!(reduce_upnext("upnext_kitsu:1_S01E02"), None);
        assert_eq!(reduce_upnext("mdblist_upnext_tt1_S1"), None);
        assert_eq!(reduce_upnext("tt0944947"), None);
    }

    #[test]
    fn episode_ids_keep_the_show_prefix() {
        let ids = IdSet::from_prefixed("tt0944947:1:2");
        assert_eq!(ids.imdb.as_deref(), Some("tt0944947"));
        assert_eq!(IdSet::from_prefixed("kitsu:11").kitsu, Some(11));
    }

    #[test]
    fn literal_imdb_field_overrides_prefix() {
        let mut meta = Meta::default();
        meta.id = "tmdb:603".into();
        meta.imdb_id = Some("tt0133093".into());
        meta.p_tvdb = Some(9);
        let ids = IdSet::from_meta(&meta);
        assert_eq!(ids.imdb.as_deref(), Some("tt0133093"));
        assert_eq!(ids.tmdb, Some(603));
        assert_eq!(ids.tvdb, Some(9));
    }
}
