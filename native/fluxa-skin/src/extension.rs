use crate::contract::{contract, valid_id};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    Script,
    Service,
    Subtitles,
    SkinHelper,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", default)]
pub struct Contributions {
    pub home_rows: Vec<Entry>,
    pub detail_sections: Vec<Entry>,
    pub settings_pages: Vec<Entry>,
    pub actions: Vec<Entry>,
    pub events: Vec<String>,
    pub data: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Extension {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    pub kind: ExtensionKind,
    pub entry: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub network_hosts: Vec<String>,
    #[serde(default)]
    pub refresh_seconds: Option<u64>,
    #[serde(default)]
    pub contributes: Contributions,
}

impl Extension {
    pub fn allows(&self, permission: &str) -> bool {
        self.permissions.iter().any(|granted| granted == permission)
    }
}

fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

pub fn parse_extension(source: &str) -> Result<Extension, Vec<String>> {
    let extension: Extension =
        serde_json::from_str(source).map_err(|error| vec![error.to_string()])?;
    let spec = contract();
    let mut problems = Vec::new();
    if extension.schema_version != 1 {
        problems.push(format!(
            "schemaVersion {} is not supported",
            extension.schema_version
        ));
    }
    if !valid_id(&extension.id) {
        problems.push(format!(
            "id {:?} may use letters, digits, - and _",
            extension.id
        ));
    }
    let entry = std::path::Path::new(&extension.entry);
    if extension.entry.is_empty()
        || !entry
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
    {
        problems.push(format!(
            "entry {:?} must be a relative path",
            extension.entry
        ));
    }
    for permission in &extension.permissions {
        if !spec.permissions.contains(permission) {
            problems.push(format!("unknown permission {permission:?}"));
        }
    }
    if extension.allows("network") != !extension.network_hosts.is_empty() {
        problems.push(
            "networkHosts must be listed exactly when the network permission is requested".into(),
        );
    }
    for host in &extension.network_hosts {
        if !valid_host(host) {
            problems.push(format!(
                "networkHosts entry {host:?} must be a bare host name"
            ));
        }
    }

    let needs = [
        (
            "homeRows",
            "ui.home_rows",
            extension.contributes.home_rows.len(),
        ),
        (
            "detailSections",
            "ui.detail_sections",
            extension.contributes.detail_sections.len(),
        ),
        (
            "settingsPages",
            "ui.settings",
            extension.contributes.settings_pages.len(),
        ),
        ("actions", "ui.actions", extension.contributes.actions.len()),
    ];
    for (field, permission, count) in needs {
        if count > 0 && !extension.allows(permission) {
            problems.push(format!(
                "contributes.{field} needs the {permission} permission"
            ));
        }
    }
    let entries = extension
        .contributes
        .home_rows
        .iter()
        .chain(&extension.contributes.detail_sections)
        .chain(&extension.contributes.settings_pages)
        .chain(&extension.contributes.actions);
    for entry in entries {
        if !valid_id(&entry.id) {
            problems.push(format!(
                "contribution id {:?} may use letters, digits, - and _",
                entry.id
            ));
        }
    }
    for event in &extension.contributes.events {
        let Some(family) = event.split('.').next() else {
            continue;
        };
        if !spec.events.contains(event) {
            problems.push(format!("unknown event {event:?}"));
        } else if family != "app" && !extension.allows(&format!("events.{family}")) {
            problems.push(format!(
                "event {event} needs the events.{family} permission"
            ));
        }
    }
    if !extension.contributes.events.is_empty()
        && !extension.allows("background")
        && extension.kind != ExtensionKind::Service
    {
        problems.push(
            "events are only delivered to services or extensions with the background permission"
                .into(),
        );
    }

    if problems.is_empty() {
        Ok(extension)
    } else {
        Err(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> Result<Extension, Vec<String>> {
        parse_extension(&value.to_string())
    }

    fn base() -> serde_json::Value {
        json!({
            "schemaVersion": 1, "id": "example-tracker", "name": "Example Tracker",
            "version": "1.0.0", "kind": "service", "entry": "main.js",
            "permissions": ["network", "storage", "events.playback"],
            "networkHosts": ["api.example.com"],
            "contributes": {"events": ["playback.ended"], "actions": []}
        })
    }

    #[test]
    fn well_formed_service_is_accepted() {
        assert!(parse(base()).is_ok());
    }

    #[test]
    fn network_permission_requires_declared_hosts() {
        let mut manifest = base();
        manifest["networkHosts"] = json!([]);
        assert!(parse(manifest).is_err());
    }

    #[test]
    fn urls_are_not_host_names() {
        let mut manifest = base();
        manifest["networkHosts"] = json!(["https://api.example.com/v1"]);
        assert!(parse(manifest).is_err());
    }

    #[test]
    fn contributions_need_their_permission() {
        let mut manifest = base();
        manifest["contributes"]["homeRows"] = json!([{"id": "recent", "title": "Recent"}]);
        let problems = parse(manifest).unwrap_err();
        assert!(problems.iter().any(|p| p.contains("ui.home_rows")));
    }

    #[test]
    fn events_need_the_matching_family_permission() {
        let mut manifest = base();
        manifest["contributes"]["events"] = json!(["library.changed"]);
        let problems = parse(manifest).unwrap_err();
        assert!(problems.iter().any(|p| p.contains("events.library")));
    }

    #[test]
    fn entry_cannot_escape_the_folder() {
        let mut manifest = base();
        manifest["entry"] = json!("../outside.js");
        assert!(parse(manifest).is_err());
    }

    #[test]
    fn unknown_permission_is_rejected() {
        let mut manifest = base();
        manifest["permissions"] = json!(["network", "filesystem"]);
        assert!(parse(manifest).is_err());
    }
}
