use crate::addons::sources::repository::{is_unsupported_source, normalize_stream};
use fluxa_aggmeta::types::Meta;
use fluxa_streamkit::format::{FormatInput, UserLists, format_stream};
use fluxa_streamkit::parse::{ParseEnv, ParseOutcome, parser_for};
use fluxa_streamkit::rules::config::{
    Criterion, DedupConfig, Direction, RankedExpression, SortKey,
};
use fluxa_streamkit::rules::{Pipeline, PipelineConfig, Reason};
use fluxa_streamkit::types::{
    AddonRef, MediaType, ParsedStream, RawStream, RequestContext, TitleEntry,
};
use serde_json::{Value, json};

const RESOLUTIONS: [&str; 9] = [
    "2160p", "1440p", "1080p", "720p", "576p", "480p", "360p", "240p", "144p",
];
const QUALITIES: [&str; 13] = [
    "BluRay REMUX",
    "BluRay",
    "WEB-DL",
    "WEBRip",
    "HDRip",
    "HC HD-Rip",
    "DVD REMUX",
    "DVDRip",
    "HDTV",
    "CAM",
    "TS",
    "TC",
    "SCR",
];
const LOW_QUALITIES: [&str; 4] = ["CAM", "TS", "TC", "SCR"];
const NAME_TEMPLATE: &str = "{addon.name}\n{stream.resolution::default('')}";
const DESCRIPTION_TEMPLATE: &str = "{stream.filename::default('')}\n{stream.quality::default('')} {stream.size::bytes} {?👤 {stream.seeders} ?}{stream.visualTags::join(' | ')}";

fn flag(prefs: &Value, key: &str, default: bool) -> bool {
    prefs.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn text<'a>(prefs: &'a Value, key: &str) -> &'a str {
    prefs
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
}

pub fn enabled(prefs: &Value) -> bool {
    flag(prefs, "streamPipelineEnabled", true)
}

pub fn formatting(prefs: &Value) -> bool {
    flag(prefs, "streamPipelineFormat", false)
}

pub fn config_from_prefs(prefs: &Value) -> PipelineConfig {
    let mut config = PipelineConfig::default();
    let filters = &mut config.filters;
    let floor = RESOLUTIONS
        .iter()
        .position(|name| *name == text(prefs, "streamMinResolution"));
    filters.resolutions.excluded = floor
        .map(|floor| {
            RESOLUTIONS
                .iter()
                .skip(floor + 1)
                .map(|name| (*name).to_owned())
                .collect()
        })
        .unwrap_or_default();
    if flag(prefs, "streamHideLowQuality", false) {
        filters.qualities.excluded = LOW_QUALITIES
            .iter()
            .map(|name| (*name).to_owned())
            .collect();
    }
    filters.resolutions.preferred = RESOLUTIONS.iter().map(|item| (*item).to_owned()).collect();
    filters.qualities.preferred = QUALITIES.iter().map(|item| (*item).to_owned()).collect();
    let expressions = &mut filters.expressions;
    expressions.excluded = list_expressions(text(prefs, "streamExcludedExpression"));
    expressions.ranked = list_expressions(text(prefs, "streamPreferredExpression"))
        .into_iter()
        .map(|expression| RankedExpression {
            expression,
            score: 100.0,
            enabled: true,
        })
        .collect();

    config.deduplicator = DedupConfig {
        enabled: flag(prefs, "streamDedupe", false),
        ..DedupConfig::default()
    };

    let keys: &[SortKey] = match text(prefs, "streamSortOrder") {
        "quality" => &[SortKey::Resolution, SortKey::Quality, SortKey::Size],
        "size" => &[SortKey::Size],
        "seeders" => &[SortKey::Seeders, SortKey::Resolution],
        _ => &[],
    };
    let mut criteria = Vec::new();
    if !config.filters.expressions.ranked.is_empty() {
        criteria.push(Criterion {
            key: SortKey::StreamExpressionScore,
            direction: Direction::Desc,
        });
    }
    criteria.extend(keys.iter().map(|key| Criterion {
        key: *key,
        direction: Direction::Desc,
    }));
    config.sort.global.global = criteria;

    config.limits.global = text(prefs, "streamLimit").parse().ok().filter(|n| *n > 0);
    config
}

fn list_expressions(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn invalid_expressions(prefs: &Value) -> Vec<String> {
    Pipeline::new(config_from_prefs(prefs))
        .validate()
        .into_iter()
        .map(|(source, _)| source)
        .collect()
}

fn episode_ids(id: &str) -> (Option<u32>, Option<u32>) {
    let mut parts = id.rsplit(':');
    let episode = parts.next().and_then(|part| part.parse().ok());
    let season = parts.next().and_then(|part| part.parse().ok());
    match (season, episode) {
        (Some(season), Some(episode)) => (Some(season), Some(episode)),
        _ => (None, None),
    }
}

pub fn request_context(kind: &str, id: &str, meta: Option<&Meta>) -> RequestContext {
    let series = matches!(kind, "series" | "tv" | "show" | "anime");
    let mut context = RequestContext {
        media_type: if series {
            MediaType::Series
        } else {
            MediaType::Movie
        },
        ..RequestContext::default()
    };
    if series {
        (context.season, context.episode) = episode_ids(id);
    }
    let Some(meta) = meta else {
        return context;
    };
    context.is_anime = kind == "anime"
        || ["mal:", "kitsu:", "anilist:", "anidb:"]
            .iter()
            .any(|prefix| id.starts_with(prefix))
        || meta.p_mal.is_some()
        || meta.p_kitsu.is_some()
        || meta.p_anilist.is_some();
    if !meta.name.trim().is_empty() {
        context.titles.push(TitleEntry {
            title: meta.name.clone(),
            language: None,
        });
    }
    context.year = meta
        .year
        .get(..4)
        .and_then(|year| year.parse().ok())
        .or_else(|| {
            meta.release_info
                .get(..4)
                .and_then(|year| year.parse().ok())
        });
    context.genres = meta.genres.clone();
    context.country = Some(meta.country.clone()).filter(|country| !country.is_empty());
    if let (Some(videos), Some(season), Some(episode)) =
        (&meta.videos, context.season, context.episode)
    {
        let mut ordered: Vec<_> = videos.iter().filter(|video| video.season > 0).collect();
        ordered.sort_by_key(|video| (video.season, video.episode));
        context.absolute_episode = ordered
            .iter()
            .position(|video| video.season == season && video.episode == episode)
            .and_then(|index| u32::try_from(index + 1).ok());
        context.tvdb_season = ordered
            .iter()
            .find(|video| video.season == season && video.episode == episode)
            .and_then(|video| video.tvdb_season);
    }
    context
}

pub struct Outcome {
    pub streams: Vec<Value>,
    pub removed: Vec<(String, Reason)>,
}

pub fn parse_values(streams: Vec<Value>, addon: &str) -> Vec<ParsedStream> {
    let reference = AddonRef {
        instance_id: addon.to_owned(),
        name: addon.to_owned(),
        ..AddonRef::default()
    };
    let parser = parser_for("");
    let env = ParseEnv {
        now_ms: web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map_or(0, |elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(0)),
    };
    let mut sequence = 0;
    streams
        .into_iter()
        .filter_map(|value| {
            let name = value
                .get("addonName")
                .and_then(Value::as_str)
                .unwrap_or(addon)
                .to_owned();
            let mut owner = reference.clone();
            owner.instance_id = name.clone();
            owner.name = name;
            let raw: RawStream = serde_json::from_value(value).ok()?;
            match parser.parse(raw, &owner, &mut sequence, &env) {
                Ok(ParseOutcome::Stream(stream)) => Some(*stream),
                _ => None,
            }
        })
        .collect()
}

pub fn run(prefs: &Value, context: &RequestContext, parsed: Vec<ParsedStream>) -> Outcome {
    if !enabled(prefs) {
        return Outcome {
            streams: parsed
                .into_iter()
                .filter_map(|stream| {
                    let value = normalize_stream(stream.original?, &stream.addon.name);
                    (!is_unsupported_source(&value)).then_some(value)
                })
                .collect(),
            removed: Vec::new(),
        };
    }
    let pipeline = Pipeline::new(config_from_prefs(prefs));
    let output = pipeline.run(context, parsed);
    let format = formatting(prefs);
    let user = UserLists::default();
    let streams = output
        .streams
        .into_iter()
        .filter_map(|stream| {
            let mut value = normalize_stream(stream.original.clone()?, &stream.addon.name);
            if is_unsupported_source(&value) {
                return None;
            }
            if format {
                let formatted = format_stream(
                    NAME_TEMPLATE,
                    DESCRIPTION_TEMPLATE,
                    &FormatInput {
                        stream: &stream,
                        request: context,
                        addon_name: &stream.addon.name,
                        user: &user,
                    },
                );
                value["name"] = json!(formatted.name.trim());
                value["description"] = json!(formatted.description.trim());
            }
            Some(value)
        })
        .collect();
    let removed = output
        .removed
        .into_iter()
        .map(|removal| (removal.stream_id, removal.reason))
        .collect();
    Outcome { streams, removed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluxa_streamkit::types::{AddonRef, StreamType};

    fn stream(id: &str, resolution: &str, filename: &str) -> ParsedStream {
        let addon = AddonRef {
            instance_id: "addon".into(),
            name: "Example".into(),
            ..AddonRef::default()
        };
        let mut stream = ParsedStream::blank(id, addon, StreamType::Http);
        stream.url = Some(format!("https://cdn.example/{id}.mkv"));
        stream.filename = Some(filename.into());
        stream.parsed.resolution = Some(resolution.into());
        stream.original = Some(json!({"url": stream.url, "name": id}));
        stream
    }

    #[test]
    fn excluded_resolution_is_removed_and_reported() {
        let prefs = json!({"streamMinResolution": "720p"});
        let outcome = run(
            &prefs,
            &RequestContext::default(),
            vec![
                stream("a", "1080p", "A.1080p.mkv"),
                stream("b", "480p", "B.480p.mkv"),
            ],
        );
        assert_eq!(outcome.streams.len(), 1);
        assert_eq!(outcome.removed.len(), 1);
    }

    #[test]
    fn default_prefs_keep_addon_order() {
        let outcome = run(
            &json!({}),
            &RequestContext::default(),
            vec![
                stream("a", "720p", "A.720p.mkv"),
                stream("b", "2160p", "B.2160p.mkv"),
            ],
        );
        assert_eq!(outcome.streams[0]["name"], "a");
    }

    #[test]
    fn quality_order_puts_highest_resolution_first() {
        let outcome = run(
            &json!({"streamSortOrder": "quality"}),
            &RequestContext::default(),
            vec![
                stream("a", "720p", "A.720p.mkv"),
                stream("b", "2160p", "B.2160p.mkv"),
            ],
        );
        assert_eq!(outcome.streams[0]["name"], "b");
    }

    #[test]
    fn limit_truncates_the_list() {
        let outcome = run(
            &json!({"streamLimit": "1"}),
            &RequestContext::default(),
            vec![
                stream("a", "720p", "A.720p.mkv"),
                stream("b", "2160p", "B.2160p.mkv"),
            ],
        );
        assert_eq!(outcome.streams.len(), 1);
    }

    #[test]
    fn episode_comes_from_the_request_id() {
        let context = request_context("series", "tt0944947:2:5", None);
        assert_eq!((context.season, context.episode), (Some(2), Some(5)));
    }

    #[test]
    fn formatted_lines_replace_name_and_description() {
        let outcome = run(
            &json!({"streamPipelineFormat": true}),
            &RequestContext::default(),
            vec![stream("a", "1080p", "A.1080p.mkv")],
        );
        assert!(
            outcome.streams[0]["name"]
                .as_str()
                .unwrap_or_default()
                .contains("1080p")
        );
    }
}
