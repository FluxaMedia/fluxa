use serde_json::{Value, json};

fn host_for_url(value: &str) -> Option<String> {
    let authority = value.split_once("://")?.1.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit('@').next()?.trim();
    let host = authority
        .strip_prefix('[')
        .and_then(|value| value.split_once(']').map(|(host, _)| host))
        .unwrap_or_else(|| authority.split(':').next().unwrap_or_default());
    let host = host.trim_start_matches("www.").to_ascii_lowercase();
    (!host.is_empty()).then_some(host)
}

fn normalized_service(service: &Value) -> Option<Value> {
    let name = service.get("name")?.as_str()?;
    let url = service.get("url")?.as_str()?;
    let last = service.get("last").unwrap_or(&Value::Null);
    Some(json!({
        "id": service.get("id"),
        "name": name,
        "url": url,
        "state": last.get("state"),
        "up": last.get("up"),
        "latency": last.get("latency"),
        "checkedAt": last.get("checkedAt"),
        "uptimePercent": service.get("uptimePercent"),
        "uptime24h": service.get("uptimeWindows").and_then(|windows| windows.get("h24")),
        "maintenance": service.get("maintenance"),
    }))
}

pub(crate) fn addon_uptime_match_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let services = request.get("services")?.as_array()?;
    let addons = request.get("addons")?.as_array()?;
    let normalized_services: Vec<(&Value, String)> = services
        .iter()
        .filter_map(|service| {
            host_for_url(service.get("url")?.as_str()?).map(|host| (service, host))
        })
        .collect();
    let matches: Vec<Value> = addons
        .iter()
        .filter_map(|addon| {
            let id = addon.get("id")?.as_str()?;
            let host = host_for_url(addon.get("url")?.as_str()?)?;
            let service = normalized_services
                .iter()
                .find(|(_, service_host)| service_host == &host)
                .and_then(|(service, _)| normalized_service(service))?;
            Some(json!({ "id": id, "service": service }))
        })
        .collect();
    serde_json::to_string(&matches).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_manifest_and_configure_urls_by_host() {
        let result = addon_uptime_match_plan_json(
            r#"{
                "addons":[{"id":"aiometadata","url":"https://aiometadata.elfhosted.com/manifest.json"}],
                "services":[{
                    "id":"aiometadata-elfhosted",
                    "name":"AIOMetadata (ElfHosted)",
                    "url":"https://aiometadata.elfhosted.com/configure",
                    "last":{"state":"UP","up":true,"latency":393},
                    "uptimeWindows":{"h24":98.47}
                }]
            }"#,
        )
        .unwrap();
        let matches: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(matches[0]["id"], "aiometadata");
        assert_eq!(matches[0]["service"]["state"], "UP");
        assert_eq!(matches[0]["service"]["uptime24h"], 98.47);
    }

    #[test]
    fn ignores_untracked_addons() {
        let result = addon_uptime_match_plan_json(
            r#"{
                "addons":[{"id":"other","url":"https://other.example/manifest.json"}],
                "services":[{"name":"Tracked","url":"https://tracked.example/","last":{}}]
            }"#,
        )
        .unwrap();
        assert_eq!(result, "[]");
    }
}
