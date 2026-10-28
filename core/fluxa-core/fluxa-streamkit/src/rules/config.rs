use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ListRule {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
    pub required: Vec<String>,
    pub preferred: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Band {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Band {
    pub fn is_unset(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }

    pub fn contains(&self, value: f64) -> bool {
        self.min.is_none_or(|m| value >= m) && self.max.is_none_or(|m| value <= m)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RangeRule {
    pub required: Band,
    pub excluded: Band,
    pub included: Band,
    pub types: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternRule {
    pub pattern: String,
    pub name: Option<String>,
    pub negate: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternFilter {
    pub included: Vec<PatternRule>,
    pub excluded: Vec<PatternRule>,
    pub required: Vec<PatternRule>,
    pub preferred: Vec<PatternRule>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct KeywordFilter {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
    pub required: Vec<String>,
    pub preferred: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankedPattern {
    pub pattern: String,
    pub name: Option<String>,
    pub score: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Limits {
    pub global: Band,
    pub movies: Band,
    pub series: Band,
    pub anime: Band,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScaledLimits {
    pub global: Limits,
    pub resolutions: HashMap<String, Limits>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Scope {
    pub request_types: Vec<String>,
    pub addons: Vec<String>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            request_types: Vec::new(),
            addons: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct YearMatching {
    pub enabled: bool,
    pub tolerance: i32,
    pub strict: bool,
    pub strict_types: Vec<String>,
    pub use_initial_air_date: bool,
    #[serde(flatten)]
    pub scope: Scope,
}

impl Default for YearMatching {
    fn default() -> Self {
        Self {
            enabled: false,
            tolerance: 1,
            strict: true,
            strict_types: vec!["movie".into()],
            use_initial_air_date: false,
            scope: Scope::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SeasonEpisodeMatching {
    pub enabled: bool,
    pub strict: bool,
    #[serde(flatten)]
    pub scope: Scope,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EpisodeTitleMatching {
    pub enabled: bool,
    pub similarity_threshold: f64,
    #[serde(flatten)]
    pub scope: Scope,
}

impl Default for EpisodeTitleMatching {
    fn default() -> Self {
        Self {
            enabled: false,
            similarity_threshold: 0.8,
            scope: Scope::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TitleMode {
    #[default]
    Exact,
    Contains,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ambiguous {
    #[default]
    Discard,
    Keep,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TitleMatching {
    pub enabled: bool,
    pub mode: TitleMode,
    pub similarity_threshold: f64,
    pub ambiguous_results: Ambiguous,
    #[serde(flatten)]
    pub scope: Scope,
}

impl Default for TitleMatching {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: TitleMode::Exact,
            similarity_threshold: 0.85,
            ambiguous_results: Ambiguous::Discard,
            scope: Scope::default(),
        }
    }
}

fn enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankedExpression {
    pub expression: String,
    pub score: f64,
    #[serde(default = "enabled")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ExpressionConfig {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
    pub required: Vec<String>,
    pub preferred: Vec<String>,
    pub ranked: Vec<RankedExpression>,
    pub pinned: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FilterConfig {
    pub expressions: ExpressionConfig,
    pub stream_types: ListRule,
    pub resolutions: ListRule,
    pub qualities: ListRule,
    pub encodes: ListRule,
    pub visual_tags: ListRule,
    pub audio_tags: ListRule,
    pub audio_channels: ListRule,
    pub languages: ListRule,
    pub subtitles: ListRule,
    pub release_groups: ListRule,
    pub regex: PatternFilter,
    pub keywords: KeywordFilter,
    pub ranked_regex: Vec<RankedPattern>,
    pub seeders: RangeRule,
    pub age: RangeRule,
    pub size: ScaledLimits,
    pub bitrate: ScaledLimits,
    pub exclude_uncached: bool,
    pub exclude_cached: bool,
    pub exclude_season_packs: bool,
    pub skip_language_filter: bool,
    pub max_age_hours: Option<f64>,
    pub year_matching: YearMatching,
    pub season_episode_matching: SeasonEpisodeMatching,
    pub episode_title_matching: EpisodeTitleMatching,
    pub title_matching: TitleMatching,
    pub language_inference: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortKey {
    Quality,
    Resolution,
    Language,
    Subtitle,
    VisualTag,
    AudioTag,
    AudioChannel,
    StreamType,
    Encode,
    Size,
    Service,
    Seeders,
    Private,
    Age,
    Addon,
    RegexPatterns,
    Cached,
    Library,
    Keyword,
    StreamExpressionMatched,
    StreamExpressionScore,
    RegexScore,
    Seadex,
    Bitrate,
    ReleaseGroup,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Desc,
    Asc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Criterion {
    pub key: SortKey,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CriteriaSet {
    pub global: Vec<Criterion>,
    pub cached: Vec<Criterion>,
    pub uncached: Vec<Criterion>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SortConfig {
    pub global: CriteriaSet,
    pub movies: CriteriaSet,
    pub series: CriteriaSet,
    pub anime: CriteriaSet,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiGroup {
    #[default]
    Aggressive,
    Conservative,
    KeepAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BucketMode {
    SingleResult,
    PerService,
    PerAddon,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryBehaviour {
    #[default]
    Ignore,
    Prefer,
    Exclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tiebreaker {
    TorrentSeeders,
    UsenetAge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DedupKey {
    Filename,
    InfoHash,
    SmartDetect,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DedupConfig {
    pub enabled: bool,
    pub keys: Vec<DedupKey>,
    pub multi_group_behaviour: MultiGroup,
    pub cached: BucketMode,
    pub uncached: BucketMode,
    pub p2p: BucketMode,
    pub http: BucketMode,
    pub live: BucketMode,
    pub youtube: BucketMode,
    pub external: BucketMode,
    pub exclude_addons: Vec<String>,
    pub library_behaviour: LibraryBehaviour,
    pub tiebreakers_before_addon: Vec<Tiebreaker>,
    pub tiebreakers_after_addon: Vec<Tiebreaker>,
    pub smart_detect_attributes: Vec<String>,
    pub rounding_percent: f64,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            keys: vec![DedupKey::Filename, DedupKey::InfoHash],
            multi_group_behaviour: MultiGroup::Aggressive,
            cached: BucketMode::PerAddon,
            uncached: BucketMode::PerAddon,
            p2p: BucketMode::PerAddon,
            http: BucketMode::Disabled,
            live: BucketMode::Disabled,
            youtube: BucketMode::Disabled,
            external: BucketMode::Disabled,
            exclude_addons: Vec::new(),
            library_behaviour: LibraryBehaviour::Ignore,
            tiebreakers_before_addon: Vec::new(),
            tiebreakers_after_addon: Vec::new(),
            smart_detect_attributes: [
                "size",
                "resolution",
                "quality",
                "visualTags",
                "audioTags",
                "audioChannels",
                "languages",
                "encode",
                "editions",
                "network",
                "releaseGroup",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            rounding_percent: 10.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LimitMode {
    #[default]
    Independent,
    Conjunctive,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LimitConfig {
    pub global: Option<usize>,
    pub indexer: Option<usize>,
    pub release_group: Option<usize>,
    pub resolution: Option<usize>,
    pub quality: Option<usize>,
    pub addon: Option<usize>,
    pub stream_type: Option<usize>,
    pub service: Option<usize>,
    pub mode: LimitMode,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PipelineConfig {
    pub filters: FilterConfig,
    pub deduplicator: DedupConfig,
    pub sort: SortConfig,
    pub limits: LimitConfig,
    pub services: Vec<String>,
    pub addons: Vec<String>,
    pub preferred_stream_types: Vec<String>,
    pub best_release_hashes: Vec<String>,
    pub best_release_groups: Vec<String>,
}
