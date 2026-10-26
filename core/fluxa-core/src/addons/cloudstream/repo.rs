use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CloudstreamRepo {
    pub name: String,
    pub description: Option<String>,
    pub plugin_lists: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CloudstreamPlugin {
    pub internal_name: String,
    pub name: String,
    pub url: String,
    pub version: i64,
    pub status: i64,
    pub language: Option<String>,
    #[serde(deserialize_with = "null_as_default")]
    pub tv_types: Vec<String>,
    pub file_size: Option<u64>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
}

fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

pub fn parse_repo(raw: &str) -> Option<CloudstreamRepo> {
    let repo: CloudstreamRepo = serde_json::from_str(raw).ok()?;
    if repo.plugin_lists.is_empty() {
        return None;
    }
    Some(repo)
}

pub fn parse_plugins(raw: &str) -> Vec<CloudstreamPlugin> {
    let items: Vec<serde_json::Value> = serde_json::from_str(raw).unwrap_or_default();
    items
        .into_iter()
        .filter_map(|item| serde_json::from_value::<CloudstreamPlugin>(item).ok())
        .filter(|plugin| !plugin.internal_name.is_empty() && plugin.url.starts_with("http"))
        .collect()
}

pub fn plugin_installable(plugin: &CloudstreamPlugin) -> bool {
    plugin.status != 0
}

pub fn supported_types(tv_types: &[String]) -> Vec<&'static str> {
    let mut movie = false;
    let mut tv = false;
    for kind in tv_types.iter().flat_map(|kind| kind.split(',')) {
        match kind.trim() {
            "Movie" | "AnimeMovie" | "Torrent" => movie = true,
            "TvSeries" | "Anime" | "OVA" | "AsianDrama" | "Cartoon" | "Documentary" => tv = true,
            "All" | "Others" => {
                movie = true;
                tv = true;
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    if movie {
        out.push("movie");
    }
    if tv {
        out.push("tv");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_without_plugin_lists_is_rejected() {
        assert!(parse_repo(r#"{"name":"x","manifestVersion":1}"#).is_none());
    }

    #[test]
    fn repo_reads_plugin_list_urls() {
        let repo = parse_repo(
            r#"{"name":"Phisher Repo","manifestVersion":1,"pluginLists":["https://example.com/plugins.json"]}"#,
        )
        .unwrap();
        assert_eq!(repo.name, "Phisher Repo");
        assert_eq!(repo.plugin_lists, vec!["https://example.com/plugins.json"]);
    }

    #[test]
    fn plugins_skip_entries_without_a_download_url() {
        let plugins = parse_plugins(
            r#"[
                {"internalName":"HiAnime","name":"HiAnime","url":"https://example.com/HiAnime.cs3","version":12,"status":1,"tvTypes":["Anime","OVA"],"language":"en"},
                {"internalName":"Broken","name":"Broken","url":"","version":1,"status":1},
                {"name":"NoInternalName","url":"https://example.com/x.cs3"}
            ]"#,
        );
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].internal_name, "HiAnime");
        assert_eq!(plugins[0].version, 12);
    }

    #[test]
    fn plugins_with_null_tv_types_still_parse() {
        let plugins = parse_plugins(
            r#"[{"internalName":"A","name":"A","url":"https://example.com/A.cs3","tvTypes":null}]"#,
        );
        assert_eq!(plugins.len(), 1);
        assert!(plugins[0].tv_types.is_empty());
    }

    #[test]
    fn comma_joined_tv_types_are_split() {
        let types = supported_types(&["Movie,Anime,Cartoon".to_string()]);
        assert_eq!(types, vec!["movie", "tv"]);
    }

    #[test]
    fn anime_only_plugin_is_tv_only() {
        assert_eq!(
            supported_types(&["Anime".to_string(), "OVA".to_string()]),
            vec!["tv"]
        );
    }

    #[test]
    fn disabled_plugin_is_not_installable() {
        let plugin = CloudstreamPlugin {
            status: 0,
            ..Default::default()
        };
        assert!(!plugin_installable(&plugin));
    }
}
