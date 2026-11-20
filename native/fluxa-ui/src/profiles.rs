use super::*;

mod model;

pub use model::*;

pub fn profiles_data(model: &ProfilesModel, form: UiFormFactor) -> serde_json::Value {
    let language = model.language.as_str();
    let t = |key: &str| localized(key, language);
    let empty = model.profiles.is_empty();
    let items: Vec<_> = model
        .profiles
        .iter()
        .enumerate()
        .map(|(index, profile)| {
            serde_json::json!({
                "index": index.to_string(),
                "name": profile.name,
                "avatar": profile.avatar_url,
                "locked": profile.locked,
                "primary": model.primary_id.as_ref() == Some(&profile.id),
            })
        })
        .collect();
    let columns = if form == UiFormFactor::Mobile {
        2
    } else {
        (model.profiles.len() + usize::from(model.manage || empty)).clamp(1, 6)
    };
    let packs: Vec<_> = {
        let selected = model.draft.avatar_url.as_deref();
        let mut index = 0;
        model
            .all_packs()
            .into_iter()
            .map(|pack| {
                let items: Vec<_> = pack
                    .avatars
                    .iter()
                    .map(|avatar| {
                        let item = serde_json::json!({
                            "index": index.to_string(),
                            "url": avatar.url,
                            "name": avatar.name,
                            "selected": selected == Some(avatar.url.as_str()),
                        });
                        index += 1;
                        item
                    })
                    .collect();
                serde_json::json!({"title": pack.title, "count": items.len(), "items": items})
            })
            .collect()
    };
    let draft = &model.draft;
    let pin_set = draft.has_pin && !draft.remove_pin;
    let draft_name = draft.name.trim();
    let pin_message = model.pin_prompt.as_ref().and_then(|prompt| {
        prompt.error.map(|failure| match failure {
            PinFailure::Incorrect => t("profiles.pin_error"),
            PinFailure::Unavailable => t("profiles.pin_unavailable"),
            PinFailure::Locked(seconds) => t("profiles.pin_locked")
                .replace("{time}", &format!("{}:{:02}", seconds / 60, seconds % 60)),
        })
    });
    let prompt_profile = model
        .pin_prompt
        .as_ref()
        .and_then(|prompt| model.profile(&prompt.profile_id));
    let confirm_profile = model
        .confirm_delete
        .as_deref()
        .and_then(|id| model.profile(id));
    serde_json::json!({
        "isSelect": model.mode == ProfilesMode::Select,
        "isForm": model.mode == ProfilesMode::Form,
        "isAvatars": model.mode == ProfilesMode::Avatars,
        "isSettings": model.mode == ProfilesMode::Settings,
        "settings": {
            "hasBackground": model.picker.background_url.is_some(),
            "canAdd": !model.repository_input.trim().is_empty() && !model.busy,
            "backgroundHint": t("profiles.picker_background_placeholder"),
            "repositoryHint": t("profiles.avatar_pack_repository_placeholder"),
            "empty": model.picker.avatar_packs.is_empty(),
            "packs": model.picker.avatar_packs.iter().enumerate().map(|(index, pack)| serde_json::json!({
                "index": index.to_string(),
                "title": pack.title,
                "count": pack.avatars.len(),
            })).collect::<Vec<_>>(),
        },
        "form": {
            "title": t(if draft.editing.is_some() { "profiles.edit" } else { "profiles.create_new" }),
            "name": if draft_name.is_empty() { t("auto.profile") } else { draft_name.to_owned() },
            "avatar": draft.avatar_url,
            "primary": model.draft_is_primary(),
            "shared": !model.draft_is_primary(),
            "duplicate": model.duplicate_name(),
            "pinInvalid": !draft.pin.is_empty() && draft.pin.len() != 4,
            "pinSet": pin_set,
            "pinRemoved": draft.remove_pin,
            "pinHint": t(if pin_set { "profiles.pin_set_placeholder" } else { "profiles.pin_placeholder" }),
            "nameHint": t("profiles.name_placeholder"),
            "urlHint": t("profiles.image_url_placeholder"),
            "addons": draft.uses_primary_addons,
            "plugins": draft.uses_primary_plugins,
            "canSave": model.can_save(),
            "saveLabel": t(if model.busy { "common.saving" } else if draft.editing.is_some() { "profiles.save" } else { "profiles.create" }),
        },
        "avatars": {"empty": packs.is_empty(), "packs": packs},
        "pin": prompt_profile.map(|profile| serde_json::json!({
            "name": profile.name,
            "avatar": profile.avatar_url,
            "error": pin_message,
            "hint": t("profiles.enter_pin"),
            "canUnlock": model.pin_prompt.as_ref().is_some_and(|prompt| prompt.pin.len() == 4),
        })),
        "confirm": confirm_profile.map(|profile| serde_json::json!({
            "name": profile.name,
            "body": t("profiles.delete_confirm_body").replacen("%s", &profile.name, 1),
        })),
        "title": t("profiles.who_watching"),
        "manage": model.manage,
        "empty": empty,
        "items": items,
        "columns": columns,
        "gridWidth": columns * 160,
        "addIndex": model.profiles.len().to_string(),
        "showAdd": model.manage || empty,
        "addLabel": t("profiles.add_profile"),
        "manageLabel": if model.manage { t("profiles.done") } else { t("profiles.manage") },
        "primaryBadge": t("profiles.primary_badge").to_uppercase(),
    })
}
