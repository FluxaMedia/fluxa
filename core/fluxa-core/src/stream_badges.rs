use crate::types::resource::Stream;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STREAM_BADGE_IMPORT_LIMIT: usize = 3;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBadge {
    pub name: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub tag_color: String,
    #[serde(default)]
    pub tag_style: String,
    #[serde(default)]
    pub text_color: String,
    #[serde(default)]
    pub border_color: String,
}

impl StreamBadge {
    fn dedupe_key(&self) -> &str {
        if self.image_url.trim().is_empty() {
            &self.name
        } else {
            &self.image_url
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBadgeFilter {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub group_id: String,
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    #[serde(default)]
    pub tag_color: String,
    #[serde(default)]
    pub tag_style: String,
    #[serde(default)]
    pub text_color: String,
    #[serde(default)]
    pub border_color: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBadgeGroup {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default = "default_true")]
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBadgeImport {
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub filters: Vec<StreamBadgeFilter>,
    #[serde(default)]
    pub groups: Vec<StreamBadgeGroup>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBadgeRules {
    #[serde(default)]
    pub imports: Vec<StreamBadgeImport>,
}

impl StreamBadgeRules {
    fn active_import(&self) -> Option<&StreamBadgeImport> {
        self.imports
            .iter()
            .find(|import| import.is_active)
            .or_else(|| self.imports.first())
    }

    /// Dedupe imports by source URL (case-insensitive), drop imports with no
    /// usable filters, cap at [`STREAM_BADGE_IMPORT_LIMIT`], and make sure
    /// exactly one import is active. Mirrors NuvioTV's fusion badge rule
    /// normalization so imported badge sets stay well-formed.
    pub fn normalized(mut self) -> Self {
        let mut normalized: Vec<StreamBadgeImport> = Vec::new();
        for mut import in self.imports.drain(..) {
            import.source_url = import.source_url.trim().to_string();
            if import.source_url.is_empty() || import.filters.is_empty() {
                continue;
            }
            if let Some(existing) = normalized
                .iter_mut()
                .find(|existing| existing.source_url.eq_ignore_ascii_case(&import.source_url))
            {
                *existing = import;
            } else if normalized.len() < STREAM_BADGE_IMPORT_LIMIT {
                normalized.push(import);
            }
        }
        if normalized.is_empty() {
            return Self { imports: Vec::new() };
        }
        let active_index = normalized
            .iter()
            .position(|import| import.is_active)
            .unwrap_or(0);
        for (index, import) in normalized.iter_mut().enumerate() {
            import.is_active = index == active_index;
        }
        Self { imports: normalized }
    }

    pub fn upsert(mut self, mut import: StreamBadgeImport, activate: bool) -> Self {
        import.source_url = import.source_url.trim().to_string();
        if import.source_url.is_empty() {
            return self.normalized();
        }
        import.is_active = activate;
        let source_url = import.source_url.clone();
        if let Some(existing) = self
            .imports
            .iter_mut()
            .find(|existing| existing.source_url.eq_ignore_ascii_case(&source_url))
        {
            *existing = import;
        } else {
            self.imports.push(import);
        }
        if activate {
            for existing in self.imports.iter_mut() {
                existing.is_active = existing.source_url.eq_ignore_ascii_case(&source_url);
            }
        }
        self.normalized()
    }

    pub fn set_active_source(mut self, source_url: &str) -> Self {
        let source_url = source_url.trim();
        if source_url.is_empty()
            || !self
                .imports
                .iter()
                .any(|import| import.source_url.eq_ignore_ascii_case(source_url))
        {
            return self.normalized();
        }
        for import in self.imports.iter_mut() {
            import.is_active = import.source_url.eq_ignore_ascii_case(source_url);
        }
        self.normalized()
    }

    pub fn remove_source(mut self, source_url: &str) -> Self {
        let source_url = source_url.trim();
        self.imports
            .retain(|import| !import.source_url.eq_ignore_ascii_case(source_url));
        self.normalized()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamBadgeFilterPayload {
    id: Option<String>,
    group_id: Option<String>,
    name: Option<String>,
    pattern: Option<String>,
    #[serde(rename = "imageURL")]
    image_url: Option<String>,
    is_enabled: Option<bool>,
    tag_color: Option<String>,
    tag_style: Option<String>,
    text_color: Option<String>,
    border_color: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamBadgeGroupPayload {
    id: Option<String>,
    name: Option<String>,
    color: Option<String>,
    is_expanded: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamBadgePayload {
    #[serde(default)]
    filters: Vec<StreamBadgeFilterPayload>,
    #[serde(default)]
    groups: Vec<StreamBadgeGroupPayload>,
}

/// Parses a fetched fusion-badge JSON payload (NuvioTV's importable badge
/// format: `{ filters: [...], groups: [...] }`) into a [`StreamBadgeImport`].
pub fn parse_stream_badge_import_json(source_url: &str, payload: &str) -> Result<String, String> {
    let decoded: StreamBadgePayload =
        serde_json::from_str(payload).map_err(|e| format!("invalid badge JSON: {e}"))?;

    let filters: Vec<StreamBadgeFilter> = decoded
        .filters
        .into_iter()
        .filter_map(|filter| {
            let name = filter.name.unwrap_or_default().trim().to_string();
            let pattern = filter.pattern.unwrap_or_default().trim().to_string();
            if name.is_empty() || pattern.is_empty() {
                return None;
            }
            Some(StreamBadgeFilter {
                id: filter.id.unwrap_or_default(),
                group_id: filter.group_id.unwrap_or_default(),
                name,
                pattern,
                image_url: filter.image_url.unwrap_or_default(),
                is_enabled: filter.is_enabled.unwrap_or(true),
                tag_color: filter.tag_color.unwrap_or_default(),
                tag_style: filter.tag_style.unwrap_or_default(),
                text_color: filter.text_color.unwrap_or_default(),
                border_color: filter.border_color.unwrap_or_default(),
            })
        })
        .collect();

    if filters.is_empty() {
        return Err("badge import did not contain any usable filters".to_string());
    }

    let groups = decoded
        .groups
        .into_iter()
        .map(|group| StreamBadgeGroup {
            id: group.id.unwrap_or_default(),
            name: group.name.unwrap_or_default(),
            color: group.color.unwrap_or_default(),
            is_expanded: group.is_expanded.unwrap_or(true),
        })
        .collect();

    let import = StreamBadgeImport {
        source_url: source_url.trim().to_string(),
        filters,
        groups,
        is_active: true,
    };
    serde_json::to_string(&import).map_err(|e| format!("failed to encode badge import: {e}"))
}

fn rules_from_json(rules_json: &str) -> StreamBadgeRules {
    serde_json::from_str(rules_json).unwrap_or_default()
}

fn rules_to_json(rules: StreamBadgeRules) -> String {
    serde_json::to_string(&rules).unwrap_or_else(|_| r#"{"imports":[]}"#.to_string())
}

pub fn normalize_stream_badge_rules_json(rules_json: &str) -> String {
    rules_to_json(rules_from_json(rules_json).normalized())
}

pub fn upsert_stream_badge_import_json(rules_json: &str, import_json: &str, activate: bool) -> Option<String> {
    let import: StreamBadgeImport = serde_json::from_str(import_json).ok()?;
    Some(rules_to_json(
        rules_from_json(rules_json).upsert(import, activate),
    ))
}

pub fn set_active_stream_badge_source_json(rules_json: &str, source_url: &str) -> String {
    rules_to_json(rules_from_json(rules_json).set_active_source(source_url))
}

pub fn remove_stream_badge_source_json(rules_json: &str, source_url: &str) -> String {
    rules_to_json(rules_from_json(rules_json).remove_source(source_url))
}

struct CompiledFilter {
    badge: StreamBadge,
    regex: Regex,
    literal_hint: Option<String>,
}

fn extract_literal_hint(pattern: &str) -> Option<String> {
    const META_CHARS: &[char] = &['\\', '[', ']', '(', ')', '{', '}', '*', '+', '?', '|', '^', '$', '.'];
    if pattern.chars().count() >= 2 && !pattern.chars().any(|c| META_CHARS.contains(&c)) {
        return Some(pattern.to_ascii_lowercase());
    }
    if pattern.contains('|') {
        return None;
    }
    let stripped = pattern
        .replace("\\b", "")
        .replace("(?i)", "")
        .replace("(?:", "")
        .replace(['(', ')'], "");
    if stripped.chars().count() >= 2 && !stripped.chars().any(|c| META_CHARS.contains(&c)) {
        return Some(stripped.to_ascii_lowercase());
    }
    None
}

fn compile_active_filters(rules: &StreamBadgeRules) -> Vec<CompiledFilter> {
    let Some(import) = rules.active_import() else {
        return Vec::new();
    };
    import
        .filters
        .iter()
        .filter(|filter| filter.is_enabled && !filter.name.trim().is_empty() && !filter.pattern.trim().is_empty())
        .filter_map(|filter| {
            let regex = Regex::new(&format!("(?i){}", filter.pattern)).ok()?;
            Some(CompiledFilter {
                badge: StreamBadge {
                    name: filter.name.clone(),
                    image_url: filter.image_url.clone(),
                    tag_color: filter.tag_color.clone(),
                    tag_style: filter.tag_style.clone(),
                    text_color: filter.text_color.clone(),
                    border_color: filter.border_color.clone(),
                },
                literal_hint: extract_literal_hint(&filter.pattern),
                regex,
            })
        })
        .collect()
}

fn badge_match_candidates(stream: &Stream) -> Vec<String> {
    let mut candidates: Vec<String> = [
        stream.title.as_deref(),
        stream.name.as_deref(),
        stream.description.as_deref(),
        stream
            .behavior_hints
            .as_ref()
            .and_then(|hints| hints.filename.as_deref()),
        stream.extra.get("quality").and_then(Value::as_str),
        stream.extra.get("provider").and_then(Value::as_str),
        stream.extra.get("addonName").and_then(Value::as_str),
        stream.extra.get("group").and_then(Value::as_str),
    ]
    .into_iter()
    .flatten()
    .map(|value: &str| value.trim().to_string())
    .filter(|value: &String| !value.is_empty())
    .collect();

    candidates.dedup();
    if candidates.len() > 1 {
        let joined = candidates.join(" ");
        candidates.push(joined);
    }
    candidates
}

fn matches_any_candidate(filter: &CompiledFilter, candidates: &[String]) -> bool {
    candidates.iter().any(|candidate| {
        if let Some(hint) = &filter.literal_hint
            && !candidate.to_ascii_lowercase().contains(hint.as_str())
        {
            return false;
        }
        filter.regex.is_match(candidate)
    })
}

/// Matches a stream against the active import's enabled filters, returning
/// the matched [`StreamBadge`] list deduped by image URL (falling back to
/// name) in first-match order.
pub fn match_stream_badges_json(stream_json: &str, rules_json: &str) -> String {
    let Ok(stream) = serde_json::from_str::<Stream>(stream_json) else {
        return "[]".to_string();
    };
    let rules = rules_from_json(rules_json);
    let filters = compile_active_filters(&rules);
    if filters.is_empty() {
        return "[]".to_string();
    }
    let candidates = badge_match_candidates(&stream);
    if candidates.is_empty() {
        return "[]".to_string();
    }

    let mut matched: Vec<StreamBadge> = Vec::new();
    for filter in &filters {
        if matches_any_candidate(filter, &candidates)
            && !matched
                .iter()
                .any(|existing| existing.dedupe_key() == filter.badge.dedupe_key())
        {
            matched.push(filter.badge.clone());
        }
    }
    serde_json::to_string(&matched).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> &'static str {
        r##"{
            "filters": [
                {"name":"4K","pattern":"\\b4k\\b","imageURL":"https://example.com/4k.png"},
                {"name":"HDR","pattern":"\\bhdr\\b"},
                {"name":"Disabled","pattern":"nope","isEnabled":false}
            ],
            "groups": [{"id":"g1","name":"Quality","color":"#fff"}]
        }"##
    }

    #[test]
    fn parses_import_and_drops_blank_filters() {
        let import_json =
            parse_stream_badge_import_json("https://example.com/badges.json", sample_payload())
                .unwrap();
        let import: StreamBadgeImport = serde_json::from_str(&import_json).unwrap();
        assert_eq!(import.filters.len(), 3);
        assert_eq!(import.groups.len(), 1);
        assert_eq!(import.source_url, "https://example.com/badges.json");
    }

    #[test]
    fn rejects_payload_with_no_usable_filters() {
        let result = parse_stream_badge_import_json(
            "https://example.com/badges.json",
            r#"{"filters":[{"name":"","pattern":""}]}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn normalized_caps_imports_and_keeps_one_active() {
        let rules = StreamBadgeRules {
            imports: vec![
                StreamBadgeImport {
                    source_url: "a".to_string(),
                    filters: vec![StreamBadgeFilter {
                        name: "x".to_string(),
                        pattern: "x".to_string(),
                        ..Default::default()
                    }],
                    is_active: true,
                    ..Default::default()
                },
                StreamBadgeImport {
                    source_url: "b".to_string(),
                    filters: vec![StreamBadgeFilter {
                        name: "y".to_string(),
                        pattern: "y".to_string(),
                        ..Default::default()
                    }],
                    is_active: true,
                    ..Default::default()
                },
                StreamBadgeImport {
                    source_url: "c".to_string(),
                    filters: vec![StreamBadgeFilter {
                        name: "z".to_string(),
                        pattern: "z".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                StreamBadgeImport {
                    source_url: "d".to_string(),
                    filters: vec![StreamBadgeFilter {
                        name: "w".to_string(),
                        pattern: "w".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
        }
        .normalized();
        assert_eq!(rules.imports.len(), STREAM_BADGE_IMPORT_LIMIT);
        assert_eq!(rules.imports.iter().filter(|i| i.is_active).count(), 1);
    }

    #[test]
    fn upsert_replaces_same_source_url_case_insensitively() {
        let rules = StreamBadgeRules::default();
        let first = parse_stream_badge_import_json("https://EXAMPLE.com/a.json", sample_payload()).unwrap();
        let rules = serde_json::from_str::<StreamBadgeImport>(&first)
            .map(|import| rules.upsert(import, true))
            .unwrap();
        assert_eq!(rules.imports.len(), 1);

        let second = parse_stream_badge_import_json(
            "https://example.com/a.json",
            r#"{"filters":[{"name":"Only","pattern":"only"}]}"#,
        )
        .unwrap();
        let rules = serde_json::from_str::<StreamBadgeImport>(&second)
            .map(|import| rules.upsert(import, true))
            .unwrap();
        assert_eq!(rules.imports.len(), 1);
        assert_eq!(rules.imports[0].filters.len(), 1);
    }

    #[test]
    fn matches_badges_against_stream_title_and_dedupes_by_image_url() {
        let import_json =
            parse_stream_badge_import_json("https://example.com/badges.json", sample_payload())
                .unwrap();
        let rules_json = format!(r#"{{"imports":[{import_json}]}}"#);

        let stream_json = r#"{"title":"Movie.2024.4K.HDR.mkv"}"#;
        let matched = match_stream_badges_json(stream_json, &rules_json);
        let badges: Vec<StreamBadge> = serde_json::from_str(&matched).unwrap();
        assert_eq!(badges.len(), 2);
        assert_eq!(badges[0].name, "4K");
        assert_eq!(badges[1].name, "HDR");
    }

    #[test]
    fn disabled_filters_and_inactive_imports_do_not_match() {
        let import_json =
            parse_stream_badge_import_json("https://example.com/badges.json", sample_payload())
                .unwrap();
        let rules_json = format!(r#"{{"imports":[{import_json}]}}"#);
        let stream_json = r#"{"title":"nope 4k"}"#;
        let matched = match_stream_badges_json(stream_json, &rules_json);
        let badges: Vec<StreamBadge> = serde_json::from_str(&matched).unwrap();
        assert!(badges.iter().all(|b| b.name != "Disabled"));

        let empty_rules = normalize_stream_badge_rules_json(r#"{"imports":[]}"#);
        assert_eq!(match_stream_badges_json(stream_json, &empty_rules), "[]");
    }

    #[test]
    fn set_active_and_remove_source_manage_the_rule_set() {
        let a = parse_stream_badge_import_json("https://a.example/badges.json", sample_payload()).unwrap();
        let b = parse_stream_badge_import_json(
            "https://b.example/badges.json",
            r#"{"filters":[{"name":"Only","pattern":"only"}]}"#,
        )
        .unwrap();
        let rules = StreamBadgeRules::default()
            .upsert(serde_json::from_str(&a).unwrap(), true)
            .upsert(serde_json::from_str(&b).unwrap(), false);
        assert!(rules.imports.iter().find(|i| i.source_url.contains("a.example")).unwrap().is_active);

        let rules = rules.set_active_source("https://b.example/badges.json");
        assert!(rules.imports.iter().find(|i| i.source_url.contains("b.example")).unwrap().is_active);

        let rules = rules.remove_source("https://a.example/badges.json");
        assert_eq!(rules.imports.len(), 1);
        assert_eq!(rules.imports[0].source_url, "https://b.example/badges.json");
    }

    #[test]
    fn malformed_input_falls_back_to_empty_results() {
        assert_eq!(match_stream_badges_json("not json", r#"{"imports":[]}"#), "[]");
        assert_eq!(match_stream_badges_json("{}", "not json"), "[]");
    }
}
