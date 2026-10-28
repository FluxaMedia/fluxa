use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Trailer {
    pub source: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(rename = "ytId")]
    pub yt_id: String,
    pub lang: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BehaviorHints {
    pub default_video_id: Option<String>,
    pub has_scheduled_videos: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CastMember {
    pub name: String,
    pub character: String,
    pub photo: Option<String>,
    pub tmdb_id: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppExtras {
    pub cast: Vec<CastMember>,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
    pub season_posters: BTreeMap<u32, String>,
    pub certification: Option<String>,
    pub certification_local: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub season: u32,
    pub episode: u32,
    pub released: Option<String>,
    pub available: bool,
    pub overview: String,
    pub thumbnail: Option<String>,
    pub runtime: Option<String>,
    #[serde(rename = "isFiller", skip_serializing_if = "Option::is_none")]
    pub is_filler: Option<bool>,
    #[serde(rename = "isRecap", skip_serializing_if = "Option::is_none")]
    pub is_recap: Option<bool>,
    #[serde(rename = "_tvdbSeason", skip_serializing_if = "Option::is_none")]
    pub tvdb_season: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub description: String,
    pub imdb_id: Option<String>,
    pub slug: String,
    pub genres: Vec<String>,
    pub director: Vec<String>,
    pub writer: Vec<String>,
    pub year: String,
    pub released: Option<String>,
    #[serde(rename = "releaseInfo")]
    pub release_info: String,
    pub runtime: String,
    pub country: String,
    pub status: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    #[serde(rename = "landscapePoster")]
    pub landscape_poster: Option<String>,
    pub logo: Option<String>,
    pub trailers: Vec<Trailer>,
    pub videos: Option<Vec<Video>>,
    #[serde(rename = "behaviorHints")]
    pub behavior_hints: BehaviorHints,
    pub app_extras: AppExtras,
    pub keywords: Vec<String>,
    #[serde(rename = "_metaProvider", skip_serializing_if = "Option::is_none")]
    pub meta_provider: Option<String>,
    #[serde(rename = "_imdbId", skip_serializing_if = "Option::is_none")]
    pub p_imdb: Option<String>,
    #[serde(rename = "_tmdbId", skip_serializing_if = "Option::is_none")]
    pub p_tmdb: Option<u64>,
    #[serde(rename = "_tvdbId", skip_serializing_if = "Option::is_none")]
    pub p_tvdb: Option<u64>,
    #[serde(rename = "_malId", skip_serializing_if = "Option::is_none")]
    pub p_mal: Option<u64>,
    #[serde(rename = "_kitsuId", skip_serializing_if = "Option::is_none")]
    pub p_kitsu: Option<u64>,
    #[serde(rename = "_anilistId", skip_serializing_if = "Option::is_none")]
    pub p_anilist: Option<u64>,
    #[serde(rename = "_anidbId", skip_serializing_if = "Option::is_none")]
    pub p_anidb: Option<u64>,
    #[serde(rename = "_seasonNumbering", skip_serializing_if = "Option::is_none")]
    pub season_numbering: Option<String>,
}

impl Meta {
    pub fn strip_private(&mut self) {
        self.meta_provider = None;
        self.p_imdb = None;
        self.p_tmdb = None;
        self.p_tvdb = None;
        self.p_mal = None;
        self.p_kitsu = None;
        self.p_anilist = None;
        self.p_anidb = None;
        self.season_numbering = None;
        if let Some(videos) = self.videos.as_mut() {
            for v in videos {
                v.tvdb_season = None;
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CatalogPage {
    pub metas: Vec<Meta>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Movie,
    Series,
    Anime,
}

impl ContentType {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentType::Movie => "movie",
            ContentType::Series => "series",
            ContentType::Anime => "anime",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Tmdb,
    Tvdb,
    Tvmaze,
    Imdb,
    Mal,
    Kitsu,
    Anilist,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Tmdb => "tmdb",
            Provider::Tvdb => "tvdb",
            Provider::Tvmaze => "tvmaze",
            Provider::Imdb => "imdb",
            Provider::Mal => "mal",
            Provider::Kitsu => "kitsu",
            Provider::Anilist => "anilist",
        }
    }

    pub fn allowed_for(self, kind: ContentType) -> bool {
        use Provider::*;
        match kind {
            ContentType::Movie => matches!(self, Tmdb | Imdb),
            ContentType::Series => matches!(self, Tvdb | Tmdb | Tvmaze | Imdb),
            ContentType::Anime => matches!(self, Mal | Kitsu | Anilist | Tvdb | Tmdb | Imdb),
        }
    }
}

pub fn default_provider(kind: ContentType) -> Provider {
    match kind {
        ContentType::Movie => Provider::Tmdb,
        ContentType::Series => Provider::Tvdb,
        ContentType::Anime => Provider::Mal,
    }
}

pub fn fallback_chain(primary: Provider) -> Vec<Provider> {
    let mut chain = vec![primary];
    for p in [Provider::Tmdb, Provider::Imdb] {
        if !chain.contains(&p) {
            chain.push(p);
        }
    }
    chain
}
