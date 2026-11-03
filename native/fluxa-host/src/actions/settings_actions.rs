use super::*;

pub(crate) fn settings_action_json(node: u64, settings: &SettingsModel) -> Option<Value> {
    if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
        let url = settings.addon_url.trim();
        return (!url.is_empty()).then(
            || json!({"type":"addonInstallRequested", "transportUrl":url, "forceRefresh":false}),
        );
    }
    if node == fluxa_ui::NODE_SETTINGS_ADDON_REFRESH {
        return Some(
            json!({"type":"addonsRefreshRequested", "profile":settings.profile, "forceRefresh":true}),
        );
    }
    if let Some((index, action)) = fluxa_ui::addon_action(node) {
        let url = fluxa_ui::addon_transport_url(settings.addons.get(index)?)?;
        return Some(match action {
            0 => json!({"type":"addonMoveRequested", "transportUrl":url, "offset":-1}),
            1 => json!({"type":"addonMoveRequested", "transportUrl":url, "offset":1}),
            2 => json!({"type":"addonInstallRequested", "transportUrl":url, "forceRefresh":true}),
            5 => json!({
                "type":"addonRenameRequested",
                "transportUrl":url,
                "name":settings.addon_edit_name,
            }),
            4 => json!({
                "type":"addonToggleRequested",
                "transportUrl":url,
                "enabled":!fluxa_ui::addon_enabled(settings.addons.get(index)?),
            }),
            _ => json!({"type":"addonRemoveRequested", "transportUrl":url}),
        });
    }
    if node == fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_SAVE {
        let scraper = settings
            .plugins
            .get("scrapers")?
            .as_array()?
            .get(settings.plugin_config?)?;
        let values: serde_json::Map<String, Value> = fluxa_ui::plugin_config_fields(scraper)
            .iter()
            .zip(&settings.plugin_config_fields)
            .filter_map(|(field, value)| {
                Some((
                    field.get("name")?.as_str()?.to_owned(),
                    Value::String(value.trim().to_owned()),
                ))
            })
            .collect();
        return Some(json!({
            "type":"pluginScraperSettingsUpdated",
            "scraperId":scraper.get("id")?.as_str()?,
            "settings":values,
        }));
    }
    if node == fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_ADD {
        let url = settings.streamkit_addon_url.trim();
        return (!url.is_empty())
            .then(|| json!({"type":"streamkitAddonAddRequested", "transportUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_BASE
        ..fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_BASE + fluxa_ui::STREAMKIT_ADDON_LIMIT as u64)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_BASE) as usize;
        let url = fluxa_ui::addon_transport_url(settings.streamkit_addons().get(index)?)?;
        return Some(json!({"type":"streamkitAddonRemoveRequested", "transportUrl":url}));
    }
    if node == fluxa_ui::NODE_SETTINGS_METADATAKIT_CLEAR_CACHE {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        return Some(
            json!({"type":"settingsChanged", "key":"metadatakitCacheEpoch", "value":stamp.to_string()}),
        );
    }
    if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL {
        let url = settings.plugin_url.trim();
        return (!url.is_empty())
            .then(|| json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    let repositories = settings
        .plugins
        .get("repositories")
        .and_then(Value::as_array);
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 20)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryRemoveRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_GROUP_BASE..fluxa_ui::NODE_SETTINGS_PLUGIN_GROUP_BASE + 20)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_GROUP_BASE) as usize;
        let repository = repositories?.get(index)?;
        let url = repository.get("manifestUrl")?.as_str()?;
        let enabled = !repository
            .get("groupResults")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return Some(
            json!({"type":"pluginRepositoryGroupToggled", "manifestUrl":url, "enabled":enabled}),
        );
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 20)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 30)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE) as usize;
        let scraper = settings.plugins.get("scrapers")?.as_array()?.get(index)?;
        let id = scraper.get("id")?.as_str()?;
        let enabled = !scraper
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return Some(json!({"type":"pluginScraperToggled", "scraperId":id, "enabled":enabled}));
    }
    None
}

pub(super) fn open_plugin_config(state: &mut RendererState, index: usize) {
    let settings = &mut state.settings;
    if settings.plugin_config == Some(index) {
        settings.plugin_config = None;
        return;
    }
    let Some(scraper) = settings
        .plugins
        .get("scrapers")
        .and_then(Value::as_array)
        .and_then(|items| items.get(index))
    else {
        return;
    };
    let mut fields: [String; fluxa_ui::PLUGIN_CONFIG_FIELDS] = Default::default();
    for (slot, field) in fluxa_ui::plugin_config_fields(scraper).iter().enumerate() {
        let name = field
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let value = scraper
            .pointer(&format!("/settings/{name}"))
            .or_else(|| field.get("default"));
        fields[slot] = match value {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Null) | None => String::new(),
            Some(other) => other.to_string(),
        };
    }
    settings.plugin_config_fields = fields;
    settings.plugin_config = Some(index);
}
