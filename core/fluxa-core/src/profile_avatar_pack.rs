use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryPlanRequest {
    repository_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryPlanRequest {
    repository_url: String,
    repository: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogRequest {
    repository_url: String,
    #[serde(default)]
    reference: Option<String>,
    tree: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackRequest {
    manifest_url: String,
    pack: Value,
}

struct GitHubRepository {
    owner: String,
    name: String,
    path: Option<String>,
}

pub(crate) fn profile_avatar_pack_manifest_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<RepositoryPlanRequest>(request_json).ok()?;
    let manifest_url = direct_manifest_url(&request.repository_url)?;
    serde_json::to_string(&json!({ "manifestUrl": manifest_url })).ok()
}

/// Normalizes a GitHub repository pasted by a user and returns the first
/// platform-owned HTTP request needed to discover its default branch.
pub(crate) fn profile_avatar_pack_repository_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<RepositoryPlanRequest>(request_json).ok()?;
    let repository = parse_repository_url(&request.repository_url)?;
    serde_json::to_string(&json!({
        "owner": repository.owner,
        "repository": repository.name,
        "repositoryApiUrl": format!("https://api.github.com/repos/{}/{}", repository.owner, repository.name),
    }))
    .ok()
}

/// Creates the recursive GitHub tree request after the platform has fetched
/// the repository metadata returned by `profileAvatarPackRepositoryPlan`.
pub(crate) fn profile_avatar_pack_discovery_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DiscoveryPlanRequest>(request_json).ok()?;
    let repository = parse_repository_url(&request.repository_url)?;
    let reference = request
        .repository
        .get("default_branch")
        .or_else(|| request.repository.get("defaultBranch"))
        .and_then(Value::as_str)
        .filter(|value| valid_ref(value))?
        .to_string();
    serde_json::to_string(&json!({
        "reference": reference,
        "treeApiUrl": format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            repository.owner,
            repository.name,
            percent_encode(&reference),
        ),
    }))
    .ok()
}

/// Extracts every supported avatar-pack manifest from a GitHub recursive tree.
/// A pack is a `pack.json` or `json.pack` blob in any directory, allowing
/// repositories to group packs under arbitrary category paths.
pub(crate) fn profile_avatar_pack_catalog_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<CatalogRequest>(request_json).ok()?;
    let repository = parse_repository_url(&request.repository_url)?;
    let reference = request.reference.filter(|value| valid_ref(value))?;
    if request.tree.get("truncated").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let entries = request.tree.get("tree")?.as_array()?;
    let mut categories = Vec::new();
    let mut seen_paths = HashSet::new();

    for entry in entries {
        if entry.get("type").and_then(Value::as_str) != Some("blob") {
            continue;
        }
        let Some(path) = entry.get("path").and_then(Value::as_str) else {
            continue;
        };
        let (directory, filename) = path.rsplit_once('/').unwrap_or(("", path));
        if !matches!(
            filename.to_ascii_lowercase().as_str(),
            "pack.json" | "json.pack"
        ) || !valid_path(path)
            || !seen_paths.insert(path.to_string())
        {
            continue;
        }
        if let Some(target) = &repository.path
            && !path_under_target(directory, target)
        {
            continue;
        }
        let name = if directory.is_empty() {
            repository.name.clone()
        } else {
            directory
                .rsplit('/')
                .next()
                .unwrap_or(directory)
                .to_string()
        };
        categories.push(json!({
            "name": name,
            "path": directory,
            "manifestUrl": raw_file_url(&repository, &reference, path),
        }));
    }

    categories.sort_by(|left, right| {
        left["path"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&right["path"].as_str().unwrap_or("").to_lowercase())
    });
    serde_json::to_string(&json!({
        "owner": repository.owner,
        "repository": repository.name,
        "reference": reference,
        "categories": categories,
    }))
    .ok()
}

/// Validates a fetched pack document before image URLs reach a platform image
/// loader. The source repository is intentionally not restricted here: packs
/// may legitimately host images on a CDN, but only HTTPS image URLs are safe.
pub(crate) fn profile_avatar_pack_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<PackRequest>(request_json).ok()?;
    if !is_https_url(&request.manifest_url) {
        return None;
    }
    let images = request.pack.as_array().cloned().or_else(|| {
        request
            .pack
            .get("images")
            .and_then(Value::as_array)
            .cloned()
    })?;
    let title = request
        .pack
        .get("title")
        .or_else(|| request.pack.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| default_pack_title(&request.manifest_url));
    let mut avatars = Vec::new();
    let mut seen_urls = HashSet::new();
    for image in images {
        let Some(raw_url) = image.get("url").and_then(Value::as_str).map(str::trim) else {
            continue;
        };
        if !is_https_url(raw_url) {
            continue;
        }
        let url = normalize_avatar_url(raw_url);
        if !seen_urls.insert(url.clone()) {
            continue;
        }
        let name = image
            .get("name")
            .or_else(|| image.get("title"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Avatar");
        avatars.push(json!({"name": name, "url": url}));
    }
    serde_json::to_string(&json!({
        "title": title,
        "manifestUrl": request.manifest_url,
        "avatars": avatars,
    }))
    .ok()
}

fn direct_manifest_url(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if !is_https_url(trimmed) {
        return None;
    }
    let normalized = normalize_avatar_url(trimmed);
    is_manifest_filename(&normalized).then_some(normalized)
}

fn is_manifest_filename(url: &str) -> bool {
    let path = url.split('?').next().unwrap_or(url);
    let filename = path.rsplit('/').next().unwrap_or("");
    matches!(
        filename.to_ascii_lowercase().as_str(),
        "pack.json" | "json.pack"
    )
}

fn parse_repository_url(input: &str) -> Option<GitHubRepository> {
    let input = input.trim().trim_end_matches('/');
    let path = input
        .strip_prefix("https://github.com/")
        .or_else(|| input.strip_prefix("http://github.com/"))
        .or_else(|| input.strip_prefix("github.com/"))
        .unwrap_or(input);
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.splitn(3, '/');
    let owner = parts.next()?;
    let name = parts.next().filter(|name| !name.is_empty())?;
    if !valid_repository_part(owner) || !valid_repository_part(name) {
        return None;
    }
    Some(GitHubRepository {
        owner: owner.to_string(),
        name: name.to_string(),
        path: parts.next().and_then(extract_target_path),
    })
}

fn extract_target_path(rest: &str) -> Option<String> {
    let mut segments = rest.splitn(2, '/');
    let kind = segments.next()?;
    if kind != "blob" && kind != "tree" {
        return None;
    }
    let (_reference, path) = segments.next()?.split_once('/')?;
    let path = path.split('?').next().unwrap_or(path);
    if !valid_path(path) {
        return None;
    }
    let path = percent_decode(path);
    let (directory, filename) = path.rsplit_once('/').unwrap_or(("", &path));
    let directory = if matches!(
        filename.to_ascii_lowercase().as_str(),
        "pack.json" | "json.pack"
    ) {
        directory
    } else {
        path.as_str()
    };
    (!directory.is_empty()).then(|| directory.to_string())
}

fn path_under_target(directory: &str, target: &str) -> bool {
    directory == target || directory.starts_with(&format!("{target}/"))
}

fn valid_repository_part(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_ref(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 255
        && !value.contains([' ', '\\', '?', '#'])
        && !value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn default_pack_title(manifest_url: &str) -> String {
    manifest_url
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').nth(2))
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Avatar Pack".to_string())
}

fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn raw_file_url(repository: &GitHubRepository, reference: &str, path: &str) -> String {
    let path = path
        .split('/')
        .map(percent_encode)
        .collect::<Vec<_>>()
        .join("/");
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        repository.owner,
        repository.name,
        percent_encode(reference),
        path,
    )
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                vec![byte as char]
            } else {
                format!("%{byte:02X}").chars().collect()
            }
        })
        .collect()
}

#[expect(
    clippy::indexing_slicing,
    reason = "loop condition bounds every byte access"
)]
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 3 <= bytes.len()
            && let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16)
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn normalize_avatar_url(url: &str) -> String {
    let Some(rest) = url.strip_prefix("https://github.com/") else {
        return url.to_string();
    };
    let mut blob_segments = rest.splitn(2, "/blob/");
    let Some(owner_repo) = blob_segments.next() else {
        return url.to_string();
    };
    let Some(ref_and_path) = blob_segments.next() else {
        return url.to_string();
    };
    let mut owner_repo_parts = owner_repo.splitn(2, '/');
    let (Some(owner), Some(repo)) = (owner_repo_parts.next(), owner_repo_parts.next()) else {
        return url.to_string();
    };
    if owner.is_empty() || repo.is_empty() {
        return url.to_string();
    }
    let ref_and_path = ref_and_path.split('?').next().unwrap_or(ref_and_path);
    let Some((reference, path)) = ref_and_path.split_once('/') else {
        return url.to_string();
    };
    if reference.is_empty() || path.is_empty() {
        return url.to_string();
    }
    format!("https://raw.githubusercontent.com/{owner}/{repo}/{reference}/{path}")
}

fn is_https_url(value: &str) -> bool {
    let value = value.trim();
    let Some(rest) = value.strip_prefix("https://") else {
        return false;
    };
    !rest.is_empty() && !rest.starts_with('/') && !rest.contains([' ', '\\', '\n', '\r'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_plan_accepts_github_urls_only() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_repository_plan_json(
                r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars.git"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output["owner"], "eueueue292");
        assert_eq!(output["repository"], "Fusion-Profile-Avatars");
        assert!(
            profile_avatar_pack_repository_plan_json(
                r#"{"repositoryUrl":"https://github.com.evil/a/b"}"#
            )
            .is_none()
        );
    }

    #[test]
    fn catalog_discovers_nested_packs_and_builds_raw_urls() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_catalog_json(
                r#"{
                    "repositoryUrl":"eueueue292/Fusion-Profile-Avatars",
                    "reference":"main",
                    "tree":{"tree":[
                        {"path":"Attack On Titan/pack.json","type":"blob"},
                        {"path":"Disney+/Marvel/json.pack","type":"blob"},
                        {"path":"README.md","type":"blob"}
                    ]}
                }"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output["categories"].as_array().unwrap().len(), 2);
        assert_eq!(output["categories"][0]["path"], "Attack On Titan");
        assert_eq!(output["categories"][1]["name"], "Marvel");
        assert_eq!(
            output["categories"][1]["manifestUrl"],
            "https://raw.githubusercontent.com/eueueue292/Fusion-Profile-Avatars/main/Disney%2B/Marvel/json.pack"
        );
    }

    #[test]
    fn repository_plan_accepts_a_url_pointing_at_one_pack() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_repository_plan_json(
                r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Solo%20Leveling%20S2/pack.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output["owner"], "eueueue292");
        assert_eq!(output["repository"], "Fusion-Profile-Avatars");
    }

    #[test]
    fn manifest_plan_rewrites_a_github_blob_url_to_raw() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_manifest_plan_json(
                r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Solo%20Leveling%20S2/pack.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            output["manifestUrl"],
            "https://raw.githubusercontent.com/eueueue292/Fusion-Profile-Avatars/main/Solo%20Leveling%20S2/pack.json"
        );
    }

    #[test]
    fn manifest_plan_accepts_any_https_host_serving_a_manifest() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_manifest_plan_json(
                r#"{"repositoryUrl":"https://example.com/packs/solo-leveling/pack.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            output["manifestUrl"],
            "https://example.com/packs/solo-leveling/pack.json"
        );
    }

    #[test]
    fn manifest_plan_rejects_urls_not_pointing_at_a_manifest_file() {
        assert!(
            profile_avatar_pack_manifest_plan_json(
                r#"{"repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars"}"#
            )
            .is_none()
        );
        assert!(
            profile_avatar_pack_manifest_plan_json(
                r#"{"repositoryUrl":"https://example.com/packs/solo-leveling/avatar.png"}"#
            )
            .is_none()
        );
    }

    #[test]
    fn catalog_scopes_to_the_pack_a_blob_url_points_at() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_catalog_json(
                r#"{
                    "repositoryUrl":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Solo%20Leveling%20S2/pack.json",
                    "reference":"main",
                    "tree":{"tree":[
                        {"path":"Solo Leveling S2/pack.json","type":"blob"},
                        {"path":"Attack On Titan/pack.json","type":"blob"}
                    ]}
                }"#,
            )
            .unwrap(),
        )
        .unwrap();
        let categories = output["categories"].as_array().unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0]["path"], "Solo Leveling S2");
    }

    #[test]
    fn catalog_discovers_root_level_pack_and_names_it_after_the_repository() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_catalog_json(
                r#"{
                    "repositoryUrl":"you/your-repo",
                    "reference":"main",
                    "tree":{"tree":[
                        {"path":"pack.json","type":"blob"},
                        {"path":"images/luffy.png","type":"blob"}
                    ]}
                }"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output["categories"].as_array().unwrap().len(), 1);
        assert_eq!(output["categories"][0]["name"], "your-repo");
        assert_eq!(output["categories"][0]["path"], "");
        assert_eq!(
            output["categories"][0]["manifestUrl"],
            "https://raw.githubusercontent.com/you/your-repo/main/pack.json"
        );
    }

    #[test]
    fn pack_parser_accepts_bare_array_without_a_title() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_json(
                r#"{
                    "manifestUrl":"https://raw.githubusercontent.com/you/your-repo/main/pack.json",
                    "pack":[
                        {"name":"Luffy","url":"https://images.example/luffy.png"},
                        {"name":"Zoro","url":"https://images.example/zoro.png"}
                    ]
                }"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output["title"], "your-repo");
        assert_eq!(output["avatars"].as_array().unwrap().len(), 2);
        assert_eq!(output["avatars"][0]["name"], "Luffy");
    }

    #[test]
    fn pack_parser_keeps_only_unique_https_avatars() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_json(
                r#"{
                    "manifestUrl":"https://example.com/pack.json",
                    "pack":{"title":"Test pack","images":[
                        {"name":"A","url":"https://images.example/a.png"},
                        {"name":"Duplicate","url":"https://images.example/a.png"},
                        {"name":"Unsafe","url":"file:///tmp/avatar.png"}
                    ]}
                }"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(output["avatars"].as_array().unwrap().len(), 1);
        assert_eq!(output["avatars"][0]["name"], "A");
    }

    #[test]
    fn pack_parser_rewrites_github_blob_urls_to_raw() {
        let output: Value = serde_json::from_str(
            &profile_avatar_pack_json(
                r#"{
                    "manifestUrl":"https://example.com/pack.json",
                    "pack":{"title":"Hell's Paradise","images":[
                        {"name":"Choubei","url":"https://github.com/eueueue292/Fusion-Profile-Avatars/blob/main/Hells%20Paradise/Choubei.PNG?raw=true&v=3"}
                    ]}
                }"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            output["avatars"][0]["url"],
            "https://raw.githubusercontent.com/eueueue292/Fusion-Profile-Avatars/main/Hells%20Paradise/Choubei.PNG"
        );
    }
}
