use super::*;
use crate::draw_text;

pub const ADDON_NAME_LIMIT: usize = 60;

pub fn addon_action_node(index: usize, action: u64) -> u64 {
    NODE_SETTINGS_ADDON_ACTION_BASE + index as u64 * 8 + action
}

pub fn addon_action(node: u64) -> Option<(usize, u64)> {
    let offset = node.checked_sub(NODE_SETTINGS_ADDON_ACTION_BASE)?;
    (offset < 8 * 200).then(|| ((offset / 8) as usize, offset % 8))
}

pub fn kit_section(addon: &serde_json::Value) -> Option<usize> {
    let title = match manifest(addon)
        .get("id")
        .and_then(serde_json::Value::as_str)?
    {
        "fluxa.streamkit" => "Streamkit",
        "fluxa.metadatakit" => "Metadatakit",
        _ => return None,
    };
    SETTINGS_SECTIONS
        .iter()
        .position(|section| section.title == title)
}

pub fn addon_tracking_consent(addon: &serde_json::Value) -> Option<bool> {
    addon
        .get("trackingConsent")
        .and_then(serde_json::Value::as_bool)
}

pub fn addon_configure_url(addon: &serde_json::Value) -> Option<String> {
    let configurable = manifest(addon)
        .pointer("/behaviorHints/configurable")
        .and_then(serde_json::Value::as_bool)
        == Some(true);
    let base = addon_transport_url(addon)?
        .trim_end_matches("manifest.json")
        .trim_end_matches('/');
    configurable.then(|| format!("{base}/configure"))
}

pub fn addon_enabled(addon: &serde_json::Value) -> bool {
    addon.get("enabled").and_then(serde_json::Value::as_bool) != Some(false)
}

pub fn addon_title(addon: &serde_json::Value) -> Option<&str> {
    addon
        .get("displayName")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .or_else(|| text(addon, "name"))
}

pub fn addon_transport_url(addon: &serde_json::Value) -> Option<&str> {
    addon
        .get("transportUrl")
        .or_else(|| addon.pointer("/manifest/transportUrl"))
        .and_then(serde_json::Value::as_str)
}

fn manifest(addon: &serde_json::Value) -> &serde_json::Value {
    addon
        .get("manifest")
        .filter(|m| m.is_object())
        .unwrap_or(addon)
}

fn text<'a>(addon: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    manifest(addon)
        .get(key)
        .or_else(|| addon.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn host(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or_default()
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn addon_chips(addon: &serde_json::Value, language: &str) -> Vec<String> {
    let manifest = manifest(addon);
    let list = |key: &str| {
        manifest
            .get(key)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|value| {
                value
                    .as_str()
                    .or_else(|| value.get("name").and_then(serde_json::Value::as_str))
            })
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    };
    let catalogs = manifest
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let mut out: Vec<String> = Vec::new();
    for (prefix, values) in [("resource", list("resources")), ("type", list("types"))] {
        for value in values {
            if prefix == "resource" && value == "catalog" {
                if catalogs == 1 {
                    out.push(localized("settings.addon_catalog_one", language));
                } else if catalogs > 1 {
                    out.push(
                        localized("settings.addon_catalog_count", language)
                            .replace("%s", &catalogs.to_string()),
                    );
                }
                continue;
            }
            let label = localized_or(
                &format!("settings.addon_{prefix}.{value}"),
                &capitalize(&value),
                language,
            );
            if !out.contains(&label) {
                out.push(label);
            }
        }
    }
    out
}

struct Card<'a> {
    title: &'a str,
    warning: Option<String>,
    version: Option<&'a str>,
    subtitle: &'a str,
    description: Option<&'a str>,
    logo: Option<&'a str>,
    icon: Option<&'static str>,
    chips: Vec<String>,
    toggle: Option<(u64, bool)>,
    group: Option<(u64, bool, String)>,
    leading: Vec<(u64, &'static str, bool)>,
    trailing: Vec<(u64, &'static str, bool)>,
    builtin: Option<(u64, String)>,
}

fn addon_card<'a>(
    addon: &'a serde_json::Value,
    index: usize,
    count: usize,
    language: &str,
    unreachable: &[(String, String)],
) -> Card<'a> {
    let url = addon_transport_url(addon);
    let failure = url.and_then(|url| {
        unreachable
            .iter()
            .find(|(failing, _)| failing == host(url))
            .map(|(_, detail)| detail.clone())
    });
    let resolved = failure.is_none();
    let kit = kit_section(addon);
    let kit_title = kit.map(|section| SETTINGS_SECTIONS[section].title);
    Card {
        title: if resolved {
            addon_title(addon).unwrap_or("Add-on")
        } else {
            url.and_then(|url| url.rsplit('/').find(|part| !part.is_empty()))
                .unwrap_or("Add-on")
        },
        warning: failure,
        version: text(addon, "version").filter(|_| resolved),
        subtitle: url.map(host).filter(|_| kit.is_none()).unwrap_or_default(),
        description: text(addon, "description").filter(|_| resolved),
        logo: text(addon, "logo").filter(|_| resolved),
        icon: kit_title,
        chips: if resolved {
            addon_chips(addon, language)
        } else {
            Vec::new()
        },
        toggle: Some((addon_action_node(index, 4), addon_enabled(addon)))
            .filter(|_| url.is_some() && kit.is_none()),
        group: addon_tracking_consent(addon).map(|granted| {
            (
                addon_action_node(index, 6),
                granted,
                localized("settings.addon_tracking_consent", language),
            )
        }),
        leading: if kit.is_some() {
            Vec::new()
        } else {
            vec![
                (addon_action_node(index, 0), "ChevronUp", index > 0),
                (
                    addon_action_node(index, 1),
                    "ChevronDown",
                    index + 1 < count,
                ),
            ]
        },
        trailing: if kit.is_some() {
            Vec::new()
        } else {
            let configure =
                addon_configure_url(addon).map(|_| (addon_action_node(index, 7), "Sliders", true));
            configure
                .into_iter()
                .chain([
                    (addon_action_node(index, 2), "Refresh", url.is_some()),
                    (addon_action_node(index, 5), "Edit", url.is_some()),
                    (addon_action_node(index, 3), "Delete", url.is_some()),
                ])
                .collect()
        },
        builtin: kit_title.map(|title| {
            (
                addon_action_node(index, 3),
                localized("settings.addon_builtin", language)
                    .replace("%s", &section_label(title, language)),
            )
        }),
    }
}

fn repository_card<'a>(repo: &'a serde_json::Value, index: usize, language: &str) -> Card<'a> {
    let url = text(repo, "manifestUrl").unwrap_or_default();
    let count = repo
        .get("scraperCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    Card {
        title: text(repo, "name").unwrap_or("Plugin repository"),
        warning: None,
        version: text(repo, "version"),
        subtitle: url,
        description: text(repo, "description"),
        logo: text(repo, "logo").or_else(|| text(repo, "iconUrl")),
        icon: Some("Plugins"),
        chips: vec![
            localized("settings.plugin_scraper_count", language).replace("%s", &count.to_string()),
        ],
        toggle: None,
        group: Some((
            NODE_SETTINGS_PLUGIN_GROUP_BASE + index as u64,
            repo.get("groupResults")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            localized("settings.plugin_group_results", language),
        )),
        leading: Vec::new(),
        builtin: None,
        trailing: vec![
            (
                NODE_SETTINGS_PLUGIN_REFRESH_BASE + index as u64,
                "Refresh",
                true,
            ),
            (
                NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + index as u64,
                "Delete",
                true,
            ),
        ],
    }
}

fn streamkit_card<'a>(addon: &'a serde_json::Value, index: usize, language: &str) -> Card<'a> {
    let url = addon_transport_url(addon);
    Card {
        title: text(addon, "name").unwrap_or("Add-on"),
        warning: None,
        version: text(addon, "version"),
        subtitle: url.map(host).unwrap_or_default(),
        description: text(addon, "description"),
        logo: text(addon, "logo"),
        icon: Some("Plugins"),
        chips: addon_chips(addon, language),
        toggle: None,
        group: None,
        leading: Vec::new(),
        builtin: None,
        trailing: vec![(
            NODE_SETTINGS_STREAMKIT_ADDON_BASE + index as u64,
            "Delete",
            url.is_some(),
        )],
    }
}

pub fn plugin_config_input(node: u64) -> Option<usize> {
    let index = node.checked_sub(NODE_SETTINGS_PLUGIN_CONFIG_INPUT_BASE)? as usize;
    (index < PLUGIN_CONFIG_FIELDS).then_some(index)
}

pub fn plugin_config_fields(scraper: &serde_json::Value) -> &[serde_json::Value] {
    scraper
        .get("configFields")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], |fields| {
            &fields[..fields.len().min(PLUGIN_CONFIG_FIELDS)]
        })
}

fn open_config(settings: &SettingsModel) -> Option<&serde_json::Value> {
    let scraper = scrapers(settings).get(settings.plugin_config?)?;
    (!plugin_config_fields(scraper).is_empty()).then_some(scraper)
}

fn repositories(settings: &SettingsModel) -> &[serde_json::Value] {
    settings
        .plugins
        .get("repositories")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], |items| &items[..items.len().min(20)])
}

fn scrapers(settings: &SettingsModel) -> &[serde_json::Value] {
    settings
        .plugins
        .get("scrapers")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], |items| &items[..items.len().min(30)])
}

fn plugin_error(settings: &SettingsModel) -> Option<&str> {
    settings
        .plugins
        .get("error")
        .and_then(|v| v.get("message"))
        .and_then(serde_json::Value::as_str)
}

fn streamkit_note(settings: &SettingsModel, language: &str) -> Option<String> {
    if settings.streamkit_addon_rejected() {
        return Some(localized("settings.streamkit_addon_not_stream", language));
    }
    settings.addon_error.clone()
}

type FormInput<'a> = (u64, &'a str, String);
type FormButton = (u64, String, components::ButtonKind);
type FormIcon = (u64, Option<crate::gfx::TextureId>);

pub const SERVER_INPUTS: [u64; 3] = [
    NODE_SETTINGS_SERVER_ADDRESS,
    NODE_SETTINGS_SERVER_USERNAME,
    NODE_SETTINGS_SERVER_PASSWORD,
];

pub fn server_input(node: u64) -> Option<usize> {
    SERVER_INPUTS.iter().position(|input| *input == node)
}

pub fn server_index(node: u64) -> Option<usize> {
    let offset = node.checked_sub(NODE_SETTINGS_SERVER_BASE)?;
    (offset < 50).then_some(offset as usize)
}

fn server_card<'a>(server: &'a serde_json::Value, index: usize, language: &str) -> Card<'a> {
    let kind = text(server, "kind").unwrap_or_default();
    let libraries = server
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    Card {
        title: text(server, "name").unwrap_or("Server"),
        warning: None,
        version: None,
        subtitle: text(server, "baseUrl").map(host).unwrap_or_default(),
        description: None,
        logo: None,
        icon: Some("Server"),
        chips: vec![
            capitalize(kind),
            localized("settings.server_libraries", language).replace("%s", &libraries.to_string()),
        ],
        toggle: None,
        group: None,
        leading: Vec::new(),
        builtin: None,
        trailing: vec![(NODE_SETTINGS_SERVER_BASE + index as u64, "Delete", true)],
    }
}

fn server_notice(settings: &SettingsModel, language: &str) -> Option<(String, Color32)> {
    let prompt = settings
        .account_auth
        .as_ref()
        .filter(|prompt| matches!(prompt.provider.as_str(), "plex" | "jellyfin" | "emby"))?;
    if prompt.failed {
        return Some((
            localized("settings.server_failed", language),
            danger_color(),
        ));
    }
    Some((
        format!(
            "{} {}  ·  {}",
            localized("trakt.device.enter_code", language),
            prompt.code,
            prompt.url
        ),
        crate::stylesheet::color("text.body"),
    ))
}

const SERVER_KINDS: [(u64, &str); 3] = [
    (NODE_SETTINGS_SERVER_JELLYFIN, "Jellyfin"),
    (NODE_SETTINGS_SERVER_EMBY, "Emby"),
    (NODE_SETTINGS_SERVER_PLEX, "Plex"),
];
const PLEX_KIND: usize = 2;

#[cfg(test)]
mod configure_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn configure_url_sits_beside_the_manifest() {
        let addon = json!({
            "transportUrl": "https://addon.example.com/token/manifest.json",
            "manifest": {"behaviorHints": {"configurable": true}}
        });
        assert_eq!(
            addon_configure_url(&addon).as_deref(),
            Some("https://addon.example.com/token/configure")
        );
    }

    #[test]
    fn plain_addon_has_no_configure_page() {
        let addon =
            json!({"transportUrl": "https://addon.example.com/manifest.json", "manifest": {}});
        assert_eq!(addon_configure_url(&addon), None);
    }
}

impl Card<'_> {
    fn json(&self) -> serde_json::Value {
        let button = |(node, icon, enabled): &(u64, &str, bool)| serde_json::json!({"node": node.to_string(), "icon": icon, "enabled": enabled});
        serde_json::json!({
            "kind": "card",
            "title": self.title,
            "warning": self.warning,
            "version": self.version,
            "hint": self.subtitle,
            "description": self.description,
            "logo": self.logo,
            "icon": self.icon.map(|icon| if icon == "Server" { "Servers" } else { icon }),
            "chips": self.chips,
            "toggle": self.toggle.map(|(node, on)| serde_json::json!({"node": node.to_string(), "on": on})),
            "group": self.group.as_ref().map(|(node, on, label)| serde_json::json!({"node": node.to_string(), "on": on, "label": label})),
            "buttons": self.leading.iter().chain(self.trailing.iter()).map(button).collect::<Vec<_>>(),
            "builtin": self.builtin.as_ref().map(|(node, label)| serde_json::json!({"node": node.to_string(), "label": label})),
        })
    }
}

fn form_json(
    label: String,
    key: &str,
    value: &str,
    input: u64,
    button: (u64, String),
    refresh: Option<u64>,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "form",
        "title": label,
        "key": key,
        "value": value,
        "placeholder": "https://",
        "button": button.1,
        "node": button.0.to_string(),
        "fill": format!("{input}={key}"),
        "refresh": refresh.map(|node| node.to_string()),
    })
}

fn note_json(text: String, danger: bool) -> serde_json::Value {
    serde_json::json!({"kind": "note", "title": text, "danger": danger})
}

pub(super) fn addons_blocks(settings: &SettingsModel, language: &str) -> Vec<serde_json::Value> {
    let count = settings.addons.len();
    let mut rows = Vec::new();
    if let Some(error) = settings.addon_error.as_deref() {
        rows.push(note_json(error.to_owned(), true));
    }
    if count == 0 {
        rows.push(note_json(
            localized("settings.addon_empty", language),
            false,
        ));
    }
    rows.extend(settings.addons.iter().enumerate().map(|(index, addon)| {
        addon_card(
            addon,
            index,
            count,
            language,
            &settings.unreachable_addon_hosts,
        )
        .json()
    }));
    vec![
        serde_json::json!({"heading": "", "rows": [form_json(
            localized("settings.addon_url", language),
            "addonUrl",
            &settings.addon_url,
            NODE_SETTINGS_ADDON_URL,
            (NODE_SETTINGS_ADDON_INSTALL, localized("settings.addon_install", language)),
            Some(NODE_SETTINGS_ADDON_REFRESH),
        )]}),
        serde_json::json!({
            "heading": format!("{}  ·  {count}", localized("settings.addon_installed", language)),
            "rows": rows,
        }),
    ]
}

pub(super) fn streamkit_blocks(settings: &SettingsModel, language: &str) -> Vec<serde_json::Value> {
    let addons = settings.streamkit_addons();
    let mut rows = Vec::new();
    if let Some(error) = streamkit_note(settings, language) {
        rows.push(note_json(error, true));
    }
    if addons.is_empty() {
        rows.push(note_json(
            localized("settings.streamkit_addons_empty", language),
            false,
        ));
    }
    rows.extend(
        addons
            .iter()
            .enumerate()
            .map(|(index, addon)| streamkit_card(addon, index, language).json()),
    );
    vec![
        serde_json::json!({"heading": "", "rows": [form_json(
            localized("settings.streamkit_addon_url", language),
            "streamkitAddonUrl",
            &settings.streamkit_addon_url,
            NODE_SETTINGS_STREAMKIT_ADDON_URL,
            (NODE_SETTINGS_STREAMKIT_ADDON_ADD, localized("settings.streamkit_addon_add", language)),
            None,
        )]}),
        serde_json::json!({
            "heading": format!("{}  ·  {}", localized("settings.streamkit_addons", language), addons.len()),
            "rows": rows,
        }),
    ]
}

pub(super) fn plugins_blocks(settings: &SettingsModel, language: &str) -> Vec<serde_json::Value> {
    let repos = repositories(settings);
    let scrapers = scrapers(settings);
    let mut rows = Vec::new();
    if let Some(error) = plugin_error(settings) {
        rows.push(note_json(error.to_owned(), true));
    }
    if repos.is_empty() {
        rows.push(note_json(
            localized("settings.plugin_empty", language),
            false,
        ));
    }
    rows.extend(
        repos
            .iter()
            .enumerate()
            .map(|(index, repo)| repository_card(repo, index, language).json()),
    );
    let mut groups = vec![
        serde_json::json!({"heading": "", "rows": [
            form_json(
                localized("settings.plugin_url", language),
                "pluginUrl",
                &settings.plugin_url,
                NODE_SETTINGS_PLUGIN_URL,
                (NODE_SETTINGS_PLUGIN_INSTALL, localized("settings.plugin_add", language)),
                None,
            ),
            note_json(localized("settings.plugin_risk", language), false),
        ]}),
        serde_json::json!({
            "heading": format!("{}  ·  {}", localized("settings.plugin_repositories", language), repos.len()),
            "rows": rows,
        }),
    ];
    if scrapers.is_empty() {
        return groups;
    }
    let items = scrapers.iter().enumerate().map(|(index, scraper)| {
        let name = text(scraper, "name")
            .or_else(|| text(scraper, "id"))
            .unwrap_or("Scraper");
        let detail =
            text(scraper, "description").or_else(|| text(scraper, "repositoryUrl").map(host));
        serde_json::json!({
            "kind": "item",
            "title": name,
            "hint": detail.unwrap_or_default(),
            "toggle": {
                "node": (NODE_SETTINGS_PLUGIN_SCRAPER_BASE + index as u64).to_string(),
                "on": scraper.get("enabled").and_then(serde_json::Value::as_bool).unwrap_or(false),
            },
            "gear": (!plugin_config_fields(scraper).is_empty())
                .then(|| (NODE_SETTINGS_PLUGIN_CONFIG_BASE + index as u64).to_string()),
        })
    });
    groups.push(serde_json::json!({
        "heading": format!("{}  ·  {}", localized("settings.plugin_scrapers", language), scrapers.len()),
        "rows": items.collect::<Vec<_>>(),
    }));
    if let Some(scraper) = open_config(settings) {
        let fields = plugin_config_fields(scraper);
        let mut config: Vec<serde_json::Value> = fields
            .iter()
            .enumerate()
            .map(|(slot, field)| {
                let label = field
                    .get("label")
                    .or_else(|| field.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                serde_json::json!({
                    "kind": "field",
                    "title": label,
                    "key": format!("pluginConfig{slot}"),
                    "value": settings.plugin_config_fields[slot],
                    "placeholder": label,
                    "inputKind": "text",
                })
            })
            .collect();
        let fill = (0..fields.len())
            .map(|slot| {
                format!(
                    "{}=pluginConfig{slot}",
                    NODE_SETTINGS_PLUGIN_CONFIG_INPUT_BASE + slot as u64
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        config.push(serde_json::json!({
            "kind": "button",
            "button": localized("settings.plugin_preferences_save", language),
            "node": NODE_SETTINGS_PLUGIN_CONFIG_SAVE.to_string(),
            "fill": fill,
        }));
        groups.push(serde_json::json!({
            "heading": format!("{}  ·  {}", localized("settings.plugin_preferences", language), text(scraper, "name").unwrap_or("Scraper")),
            "rows": config,
        }));
    }
    groups
}

pub(super) fn servers_blocks(settings: &SettingsModel, language: &str) -> Vec<serde_json::Value> {
    let tabs = serde_json::json!({
        "kind": "tabs",
        "items": SERVER_KINDS
            .iter()
            .enumerate()
            .map(|(index, (node, label))| serde_json::json!({
                "label": label,
                "node": node.to_string(),
                "selected": index == settings.server_kind,
            }))
            .collect::<Vec<_>>(),
    });
    let mut form = Vec::new();
    if settings.server_kind == PLEX_KIND {
        form.push(note_json(
            localized("settings.server_plex_hint", language),
            false,
        ));
        form.push(serde_json::json!({
            "kind": "button",
            "button": localized("auth.sign_in", language),
            "node": NODE_SETTINGS_SERVER_CONNECT.to_string(),
            "fill": "",
        }));
    } else {
        let hints = [
            "settings.server_address",
            "settings.server_username",
            "settings.server_password",
        ];
        let keys = ["serverAddress", "serverUsername", "serverPassword"];
        for (index, label) in hints.iter().enumerate() {
            form.push(serde_json::json!({
                "kind": "field",
                "title": localized(label, language),
                "key": keys[index],
                "value": settings.server_fields[index],
                "placeholder": localized(label, language),
                "inputKind": if index == 2 { "password" } else { "text" },
            }));
        }
        form.push(serde_json::json!({
            "kind": "button",
            "button": localized("settings.account_connect", language),
            "node": NODE_SETTINGS_SERVER_CONNECT.to_string(),
            "fill": SERVER_INPUTS
                .iter()
                .zip(keys)
                .map(|(node, key)| format!("{node}={key}"))
                .collect::<Vec<_>>()
                .join(";"),
        }));
    }
    if let Some((message, _)) = server_notice(settings, language) {
        form.push(note_json(message, false));
    }
    let mut cards = Vec::new();
    if settings.media_servers.is_empty() {
        cards.push(note_json(
            localized("settings.server_empty", language),
            false,
        ));
    }
    cards.extend(
        settings
            .media_servers
            .iter()
            .enumerate()
            .map(|(index, server)| server_card(server, index, language).json()),
    );
    vec![
        serde_json::json!({"heading": "", "rows": [tabs]}),
        serde_json::json!({"heading": "", "rows": form}),
        serde_json::json!({
            "heading": format!("{}  ·  {}", localized("settings.server_connected", language), settings.media_servers.len()),
            "rows": cards,
        }),
    ]
}
