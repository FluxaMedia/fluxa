use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use fluxa_effects::{EffectExecutor, SessionHandle, Storage};
use fluxa_ui::{
    PinFailure, PinPrompt, PinPurpose, ProfileAvatarPack, ProfileEntry, ProfilePickerSettings,
    ProfilesMode, ProfilesModel, ProfilesRequest, localized,
};
use serde_json::{Value, json};

use crate::{RendererState, core_value, discover_command, host_log, profile_language};

pub type ImagePicker = Box<dyn Fn() -> Option<PathBuf> + Send>;

pub(crate) struct PinJob {
    prompt: PinPrompt,
    receiver: Receiver<Result<Value, String>>,
}

pub(crate) struct PinChangeJob {
    receiver: Receiver<Result<Value, String>>,
}

pub(crate) struct PackJob {
    adding: bool,
    receiver: Receiver<Vec<(String, Result<Vec<ProfileAvatarPack>, String>)>>,
}

pub(crate) fn read(storage: &Storage, key: &str) -> Option<Value> {
    storage.read_json(key).ok().flatten()
}

pub(crate) fn write(storage: &Storage, key: &str, value: &Value) {
    if let Err(error) = storage.write_json(key, value) {
        host_log(format!("Storage write {key} failed: {error}"));
    }
}

pub(crate) fn stored_profiles(storage: &Storage) -> Vec<Value> {
    read(storage, "profiles")
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
}

pub(crate) fn picker_settings(storage: &Storage) -> ProfilePickerSettings {
    read(storage, "profile_picker_settings")
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

fn entry(profile: &Value) -> ProfileEntry {
    let text = |key: &str| {
        profile
            .get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    };
    let flag = |key: &str| profile.get(key).and_then(Value::as_bool).unwrap_or(false);
    ProfileEntry {
        id: text("id").unwrap_or_default(),
        name: text("name").or_else(|| text("email")).unwrap_or_default(),
        avatar_url: text("avatarUrl").filter(|url| !url.starts_with("data:")),
        locked: text("pinHash").is_some() || flag("nuvioPinEnabled"),
        uses_primary_addons: flag("usesPrimaryAddons"),
        uses_primary_plugins: flag("usesPrimaryPlugins"),
    }
}

const HIDDEN_FLAG: &str = "hiddenByAccount";

pub(crate) fn visible_profiles(storage: &Storage) -> Vec<Value> {
    stored_profiles(storage)
        .into_iter()
        .filter(|profile| !flag(profile, HIDDEN_FLAG))
        .collect()
}

pub(crate) fn hide_local_profiles(storage: &Storage, hidden: bool) {
    let mut profiles = stored_profiles(storage);
    for profile in &mut profiles {
        let signed = profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .is_some_and(|token| !token.is_empty());
        let Some(fields) = profile.as_object_mut() else {
            continue;
        };
        if hidden && !signed {
            fields.insert(HIDDEN_FLAG.to_owned(), json!(true));
        } else if !hidden {
            fields.remove(HIDDEN_FLAG);
        }
    }
    write(storage, "profiles", &Value::Array(profiles));
}

fn load_model(storage: &Storage, language: String) -> ProfilesModel {
    let profiles = visible_profiles(storage);
    let primary_id = core_value("primaryProfileId", Value::Array(profiles.clone()))
        .and_then(|value| value.as_str().map(ToOwned::to_owned));
    let picker = picker_settings(storage);
    let catalog = read(storage, "nuvio_avatar_packs")
        .map(catalog_packs)
        .unwrap_or_default();
    ProfilesModel {
        catalog,
        profiles: profiles.iter().map(entry).collect(),
        primary_id,
        background_input: picker.background_url.clone().unwrap_or_default(),
        picker,
        language,
        ..ProfilesModel::default()
    }
}

pub(crate) fn should_pick_on_start(storage: &Storage) -> bool {
    let profiles = visible_profiles(storage);
    let active = read(storage, "active_profile_id")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_default();
    !profiles.is_empty()
        && !profiles
            .iter()
            .any(|profile| profile.get("id").and_then(Value::as_str) == Some(active.as_str()))
}

fn catalog_packs(value: Value) -> Vec<ProfileAvatarPack> {
    value
        .get("packs")
        .and_then(|packs| serde_json::from_value::<Vec<ProfileAvatarPack>>(packs.clone()).ok())
        .unwrap_or_default()
}

pub(crate) fn poll_catalog(state: &mut RendererState) {
    let Some(job) = state.catalog_job.as_ref() else {
        return;
    };
    let Ok(outcome) = job.try_recv() else {
        return;
    };
    state.catalog_job = None;
    let Ok(value) = outcome else {
        return;
    };
    if let Some(session) = state.session.as_ref() {
        write(session.storage(), "nuvio_avatar_packs", &value);
    }
    if let Some(model) = state.profiles.as_mut() {
        model.catalog = catalog_packs(value);
        model.fill_default_avatar();
    }
}

pub(crate) fn open(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let language = profile_language(&session.active_profile());
    let model = load_model(session.storage(), language);
    state.picker_background = model.picker.background_url.clone();
    state.profiles = Some(model);
    if state.catalog_job.is_none() {
        state.catalog_job = Some(session.executor().nuvio_avatar_catalog());
    }
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    let Some(model) = state.profiles.as_mut() else {
        return false;
    };
    if let Some(request) = model.activate(node) {
        handle(state, request);
    }
    true
}

pub(crate) fn back(state: &mut RendererState) -> bool {
    state.profiles.as_mut().is_some_and(|model| model.back())
}

pub(crate) fn handle(state: &mut RendererState, request: ProfilesRequest) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let storage = session.storage().clone();
    let executor = session.executor();
    let Some(model) = state.profiles.as_mut() else {
        return;
    };
    match request {
        ProfilesRequest::Select(id) => {
            if model.profile(&id).is_some_and(|profile| profile.locked) {
                model.pin_prompt = Some(PinPrompt {
                    profile_id: id,
                    purpose: PinPurpose::Enter,
                    pin: String::new(),
                    error: None,
                });
            } else {
                activate(state, &storage, &id);
            }
        }
        ProfilesRequest::SubmitPin => {
            let Some(prompt) = model.pin_prompt.take() else {
                return;
            };
            let profile = stored_profiles(&storage).into_iter().find(|profile| {
                profile.get("id").and_then(Value::as_str) == Some(&prompt.profile_id)
            });
            if let Some(remote) = profile
                .as_ref()
                .filter(|profile| flag(profile, "nuvioPinEnabled"))
                .cloned()
            {
                let locked_until = remote
                    .get("nuvioPinLockedUntil")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                let now = chrono::Utc::now().timestamp();
                if locked_until > now {
                    model.pin_prompt = Some(PinPrompt {
                        pin: String::new(),
                        error: Some(PinFailure::Locked((locked_until - now) as u64)),
                        ..prompt
                    });
                    return;
                }
                model.busy = true;
                let receiver = executor.nuvio_verify_pin(remote, prompt.pin.clone());
                state.pin_job = Some(PinJob { prompt, receiver });
                return;
            }
            let matches = profile.is_some_and(|profile| {
                core_value(
                    "profilePinMatches",
                    json!({"profileJson": profile.to_string(), "pin": prompt.pin}),
                )
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
            });
            if !matches {
                model.pin_prompt = Some(PinPrompt {
                    pin: String::new(),
                    error: Some(PinFailure::Incorrect),
                    ..prompt
                });
                return;
            }
            unlock(state, &storage, prompt);
        }
        ProfilesRequest::Save => {
            let change = save(model, &storage);
            if let Some(change) = change {
                let current = state
                    .edit_pin
                    .take()
                    .filter(|(id, _)| *id == change.profile_id)
                    .map(|(_, pin)| pin);
                let receiver = match change.pin {
                    pin if change.create => executor.nuvio_create_profile(change.profile, pin),
                    _ if !change.wants_pin => executor.nuvio_push_profile(change.profile),
                    Some(pin) => executor.nuvio_set_pin(change.profile, pin, current),
                    None => executor.nuvio_clear_pin(change.profile, current),
                };
                state.pin_change_job = Some(PinChangeJob { receiver });
            }
        }
        ProfilesRequest::Delete(id) => {
            let profiles = stored_profiles(&storage);
            let remote = profiles
                .iter()
                .find(|profile| profile.get("id").and_then(Value::as_str) == Some(id.as_str()))
                .filter(|profile| profile.get("nuvioAccessToken").is_some())
                .cloned();
            if let Some(next) = core_value(
                "profileMutationPlan",
                json!({"operation": "delete", "profiles": profiles, "id": id}),
            ) {
                write(&storage, "profiles", &next);
            }
            if let Some(profile) = remote {
                state.pin_change_job = Some(PinChangeJob {
                    receiver: executor.nuvio_delete_profile(profile),
                });
            }
            model.confirm_delete = None;
            reload(model, &storage);
        }
        ProfilesRequest::AddRepository => {
            let repository = model.repository_input.trim().to_owned();
            if repository.is_empty() {
                return;
            }
            model.busy = true;
            state.pack_job = Some(spawn_discovery(executor, vec![repository], true));
        }
        ProfilesRequest::RefreshRepository(repository) => {
            model.busy = true;
            state.pack_job = Some(spawn_discovery(executor, vec![repository], false));
        }
        ProfilesRequest::RefreshAllPacks => {
            let mut repositories = Vec::<String>::new();
            for pack in &model.picker.avatar_packs {
                if !repositories.contains(&pack.repository_url) {
                    repositories.push(pack.repository_url.clone());
                }
            }
            if repositories.is_empty() {
                return;
            }
            model.busy = true;
            state.pack_job = Some(spawn_discovery(executor, repositories, false));
        }
        ProfilesRequest::RemovePack(id) => {
            model.picker.avatar_packs.retain(|pack| pack.id != id);
            save_picker(model, &storage);
        }
        ProfilesRequest::SaveBackground => {
            let url = model.background_input.trim();
            model.picker.background_url = (!url.is_empty()).then(|| url.to_owned());
            save_picker(model, &storage);
            state.picker_background = model.picker.background_url.clone();
        }
        ProfilesRequest::PickAvatarImage => {
            if let Some(url) = pick_image(state, &storage) {
                if let Some(model) = state.profiles.as_mut() {
                    model.draft.avatar_url = Some(url);
                }
            }
        }
        ProfilesRequest::PickBackgroundImage => {
            if let Some(url) = pick_image(state, &storage) {
                if let Some(model) = state.profiles.as_mut() {
                    model.background_input = url.clone();
                    model.picker.background_url = Some(url);
                    save_picker(model, &storage);
                    state.picker_background = model.picker.background_url.clone();
                }
            }
        }
    }
}

fn reload(model: &mut ProfilesModel, storage: &Storage) {
    let fresh = load_model(storage, model.language.clone());
    model.profiles = fresh.profiles;
    model.primary_id = fresh.primary_id;
}

fn save_picker(model: &ProfilesModel, storage: &Storage) {
    if let Ok(value) = serde_json::to_value(&model.picker) {
        write(storage, "profile_picker_settings", &value);
    }
}

struct PinChange {
    profile_id: String,
    profile: Value,
    pin: Option<String>,
    wants_pin: bool,
    create: bool,
}

fn account_session(profiles: &[Value], active_id: Option<&str>) -> Option<Value> {
    let signed_in = |profile: &&Value| {
        profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .is_some_and(|token| !token.is_empty())
    };
    profiles
        .iter()
        .filter(signed_in)
        .find(|profile| profile.get("id").and_then(Value::as_str) == active_id)
        .or_else(|| profiles.iter().find(signed_in))
        .cloned()
}

fn save(model: &mut ProfilesModel, storage: &Storage) -> Option<PinChange> {
    if !model.can_save() {
        return None;
    }
    let profiles = stored_profiles(storage);
    let draft = model.draft.clone();
    if draft.editing.is_none()
        && let Some(mut session) = account_session(
            &profiles,
            read(storage, "active_profile_id")
                .as_ref()
                .and_then(Value::as_str),
        )
        && let Some(fields) = session.as_object_mut()
    {
        fields.insert("name".into(), json!(draft.name.trim()));
        match &draft.avatar_url {
            Some(url) => fields.insert("avatarUrl".into(), json!(url)),
            None => fields.remove("avatarUrl"),
        };
        fields.insert("usesPrimaryAddons".into(), json!(draft.uses_primary_addons));
        fields.insert(
            "usesPrimaryPlugins".into(),
            json!(draft.uses_primary_plugins),
        );
        model.mode = ProfilesMode::Select;
        return Some(PinChange {
            profile_id: String::new(),
            profile: session,
            pin: Some(draft.pin).filter(|pin| !pin.is_empty()),
            wants_pin: false,
            create: true,
        });
    }
    let existing = draft.editing.as_ref().and_then(|id| {
        profiles
            .iter()
            .find(|profile| profile.get("id").and_then(Value::as_str) == Some(id))
            .cloned()
    });
    let mut profile = match existing {
        Some(profile) => profile,
        None => {
            let id = new_profile_id();
            core_value(
                "createProfilePlan",
                json!({"id": id, "name": draft.name, "color": "#3F7CFF"}),
            )
            .unwrap_or_else(|| json!({"id": id, "name": draft.name.trim()}))
        }
    };
    let Some(fields) = profile.as_object_mut() else {
        return None;
    };
    fields.insert("name".into(), json!(draft.name.trim()));
    match &draft.avatar_url {
        Some(url) => fields.insert("avatarUrl".into(), json!(url)),
        None => fields.remove("avatarUrl"),
    };
    let remote = fields.contains_key("nuvioAccessToken");
    let wants_change = !draft.pin.is_empty() || draft.remove_pin;
    if remote {
        fields.remove("pinHash");
    } else if !draft.pin.is_empty() {
        if let Some(hash) = core_value("profilePinHash", json!({"pin": draft.pin})) {
            fields.insert("pinHash".into(), hash);
        }
    } else if draft.remove_pin {
        fields.remove("pinHash");
    }
    fields.insert("usesPrimaryAddons".into(), json!(draft.uses_primary_addons));
    fields.insert(
        "usesPrimaryPlugins".into(),
        json!(draft.uses_primary_plugins),
    );
    if let Some(next) = core_value(
        "profileMutationPlan",
        json!({"operation": "save", "profiles": profiles, "profile": profile}),
    ) {
        write(storage, "profiles", &next);
    }
    reload(model, storage);
    model.mode = ProfilesMode::Select;
    remote.then(|| PinChange {
        profile_id: profile["id"].as_str().unwrap_or_default().to_owned(),
        profile,
        pin: Some(draft.pin).filter(|pin| !pin.is_empty()),
        wants_pin: wants_change,
        create: false,
    })
}

pub(crate) fn new_profile_id() -> String {
    let nanos = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("{nanos:x}")
}

pub(crate) fn clear_typed_settings(state: &mut RendererState) {
    let settings = &mut state.settings;
    settings.poster_fields = Default::default();
    settings.server_fields = Default::default();
    settings.badge_fields = Default::default();
    settings.collection_fields = Default::default();
    settings.addon_url.clear();
    settings.plugin_url.clear();
    settings.plugin_config = None;
    settings.plugin_config_fields = Default::default();
}

pub(crate) fn activate(state: &mut RendererState, storage: &Storage, id: &str) {
    write(storage, "active_profile_id", &json!(id));
    clear_typed_settings(state);
    match SessionHandle::open(storage.clone()) {
        Ok(session) => {
            if let Err(error) = load_profile(&session, true) {
                host_log(format!("Profile load failed: {error}"));
            }
            state.session = Some(session);
            state.session_revision = None;
            state.last_snapshot_revision = None;
            state.core_snapshot = None;
            state.route = crate::Route::Home;
            state.profiles = None;
            state.pack_job = None;
            state.card_menu = None;
            state.account_auth = None;
            state.screen_scroll_offsets.clear();
            state.library_tab = fluxa_ui::LibraryTab::Watchlist;
            state.library_query.clear();
            state.library_sort = "recent".to_owned();
            state.library_type = "all".to_owned();
            state.library_list = false;
            state.library_downloads = false;
            state.library_download_group = None;
            state.folder = Default::default();
            state.folder_tab = 0;
            state.calendar = Default::default();
            state.detail = Default::default();
        }
        Err(error) => host_log(format!("Profile switch failed: {error}")),
    }
}

pub(crate) fn save_profile(storage: &Storage, profile: &Value) {
    let profiles = stored_profiles(storage);
    if let Some(next) = core_value(
        "profileMutationPlan",
        json!({"operation": "save", "profiles": profiles, "profile": profile}),
    ) {
        write(storage, "profiles", &next);
    }
}

pub(crate) fn load_profile(session: &SessionHandle, force: bool) -> Result<(), String> {
    load_profile_with(session, session.active_profile(), force)
}

pub(crate) fn load_profile_with(
    session: &SessionHandle,
    profile: Value,
    force: bool,
) -> Result<(), String> {
    session.dispatch(
        json!({"type": "addonsRefreshRequested", "profile": profile, "forceRefresh": true}),
    )?;
    session.dispatch(json!({
        "type": "homeLoadRequested",
        "profile": profile,
        "language": profile_language(&profile),
        "force": force,
    }))?;
    session.dispatch(discover_command(&profile, "movie", "", "", "", true))?;
    session.dispatch(json!({"type": "libraryHydrateRequested", "profileId": profile.get("id")}))?;
    let Some(id) = profile.get("id").and_then(Value::as_str) else {
        return Ok(());
    };
    let key = legacy_addons_key(id);
    let storage = session.storage();
    let urls = read(storage, &key)
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    if urls.is_empty() {
        return Ok(());
    }
    for url in urls.iter().filter_map(Value::as_str) {
        session.dispatch(
            json!({"type": "addonInstallRequested", "transportUrl": url, "forceRefresh": true}),
        )?;
    }
    write(storage, &key, &json!([]));
    Ok(())
}

fn legacy_addons_key(profile_id: &str) -> String {
    format!("legacy_addons_{profile_id}")
}

pub fn import_legacy(data_dir: PathBuf, legacy: &str) -> Result<bool, String> {
    let storage = Storage::open(data_dir)?;
    if storage
        .read_json("profiles")?
        .is_some_and(|profiles| profiles.is_array())
    {
        return Ok(false);
    }
    let legacy: Value = serde_json::from_str(legacy).map_err(|error| error.to_string())?;
    let legacy = if legacy.get("prefs").is_some() {
        from_android_prefs(&legacy)
    } else {
        legacy
    };
    let profiles = legacy
        .get("profiles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for profile in &profiles {
        let (Some(id), Some(addons)) = (
            profile.get("id").and_then(Value::as_str),
            profile
                .get("localAddons")
                .filter(|addons| addons.is_array()),
        ) else {
            continue;
        };
        storage.write_json(&legacy_addons_key(id), addons)?;
    }
    storage.write_json("profiles", &Value::Array(profiles))?;
    if let Some(active) = legacy.get("activeProfileId").filter(|id| id.is_string()) {
        storage.write_json("active_profile_id", active)?;
    }
    if let Some(mut picker) = legacy
        .get("pickerSettings")
        .filter(|picker| picker.is_object())
        .cloned()
    {
        for pack in picker
            .get_mut("avatarPacks")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
        {
            if pack.get("manifestUrl").is_none() {
                let id = pack.get("id").cloned().unwrap_or_default();
                pack["manifestUrl"] = id;
            }
        }
        storage.write_json("profile_picker_settings", &picker)?;
    }
    Ok(true)
}

fn from_android_prefs(raw: &Value) -> Value {
    let prefs = &raw["prefs"];
    let text = |value: &Value| {
        value
            .as_str()
            .and_then(|text| serde_json::from_str::<Value>(text).ok())
    };
    let profiles = text(&prefs["profiles_list"])
        .and_then(|list| list.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|profile| {
            let schema = profile
                .get("schemaVersion")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let mut profile = core_value(
                "profileSettingsMigrationPlan",
                json!({"raw": profile, "schemaVersion": schema}),
            )
            .and_then(|plan| plan.get("migratedProfile").cloned())
            .filter(Value::is_object)?;
            let id = profile
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if let Some(Value::Object(credentials)) = text(&raw["credentials"][&id]) {
                for (key, value) in credentials
                    .into_iter()
                    .filter(|(_, value)| !value.is_null())
                {
                    profile[key] = value;
                }
            }
            let key = core_value("profileLocalAddonsKey", profile.clone());
            if let Some(addons) = key
                .as_ref()
                .and_then(Value::as_str)
                .and_then(|key| text(&prefs[key]))
            {
                profile["localAddons"] = addons;
            }
            Some(profile)
        })
        .collect::<Vec<_>>();
    json!({
        "profiles": profiles,
        "activeProfileId": prefs["last_active_profile_id"],
        "pickerSettings": text(&raw["picker"]),
    })
}

fn pick_image(state: &mut RendererState, storage: &Storage) -> Option<String> {
    let source = (state.image_picker.as_ref()?)()?;
    let directory = storage.dir().join("profile-images");
    std::fs::create_dir_all(&directory).ok()?;
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("png");
    let target = directory.join(format!("{}.{extension}", new_profile_id()));
    std::fs::copy(&source, &target).ok()?;
    Some(format!("file://{}", target.display()))
}

fn spawn_discovery(executor: EffectExecutor, repositories: Vec<String>, adding: bool) -> PackJob {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let fetch = |url: &str| executor.fetch_json(url.to_owned()).recv().ok().flatten();
        let results = repositories
            .into_iter()
            .map(|repository| {
                let packs = discover(&fetch, &repository);
                (repository, packs)
            })
            .collect();
        let _ = sender.send(results);
    });
    PackJob { adding, receiver }
}

fn parse_pack(
    fetch: &(impl Fn(&str) -> Option<Value> + Sync),
    manifest_url: &str,
    repository: &str,
) -> Option<ProfileAvatarPack> {
    let pack = fetch(manifest_url)?;
    let parsed = core_value(
        "profileAvatarPackParse",
        json!({"manifestUrl": manifest_url, "pack": pack}),
    )?;
    let mut pack: ProfileAvatarPack = serde_json::from_value(json!({
        "id": parsed.get("manifestUrl"),
        "repositoryUrl": repository,
        "title": parsed.get("title"),
        "manifestUrl": parsed.get("manifestUrl"),
        "avatars": parsed.get("avatars"),
    }))
    .ok()?;
    pack.avatars.retain(|avatar| !avatar.url.is_empty());
    (!pack.avatars.is_empty()).then_some(pack)
}

fn discover(
    fetch: &(impl Fn(&str) -> Option<Value> + Sync),
    repository: &str,
) -> Result<Vec<ProfileAvatarPack>, String> {
    if let Some(plan) = core_value(
        "profileAvatarPackManifestPlan",
        json!({"repositoryUrl": repository}),
    ) && let Some(manifest) = plan.get("manifestUrl").and_then(Value::as_str)
    {
        return parse_pack(fetch, manifest, repository)
            .map(|pack| vec![pack])
            .ok_or_else(|| "no_valid_avatars".to_owned());
    }
    let plan = core_value(
        "profileAvatarPackRepositoryPlan",
        json!({"repositoryUrl": repository}),
    )
    .ok_or("invalid_url")?;
    let api = plan
        .get("repositoryApiUrl")
        .and_then(Value::as_str)
        .ok_or("invalid_url")?;
    let info = fetch(api).ok_or("repository_not_found")?;
    let plan = core_value(
        "profileAvatarPackDiscoveryPlan",
        json!({"repositoryUrl": repository, "repository": info}),
    )
    .ok_or("repository_not_found")?;
    let reference = plan.get("reference").cloned().unwrap_or(Value::Null);
    let tree_url = plan
        .get("treeApiUrl")
        .and_then(Value::as_str)
        .ok_or("repository_not_found")?;
    let tree = fetch(tree_url).ok_or("repository_not_found")?;
    let catalog = core_value(
        "profileAvatarPackCatalog",
        json!({"repositoryUrl": repository, "reference": reference, "tree": tree}),
    )
    .ok_or("no_packs_found")?;
    let manifests = catalog
        .get("categories")
        .and_then(Value::as_array)
        .map(|categories| {
            categories
                .iter()
                .filter_map(|category| category.get("manifestUrl").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if manifests.is_empty() {
        return Err("no_packs_found".to_owned());
    }
    let packs = std::thread::scope(|scope| {
        manifests
            .iter()
            .map(|manifest| scope.spawn(|| parse_pack(fetch, manifest, repository)))
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|handle| handle.join().ok().flatten())
            .collect::<Vec<_>>()
    });
    if packs.is_empty() {
        return Err("no_valid_avatars".to_owned());
    }
    Ok(packs)
}

fn flag(profile: &Value, key: &str) -> bool {
    profile.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn unlock(state: &mut RendererState, storage: &Storage, prompt: PinPrompt) {
    let Some(model) = state.profiles.as_mut() else {
        return;
    };
    match prompt.purpose {
        PinPurpose::Enter => activate(state, storage, &prompt.profile_id),
        PinPurpose::Edit => {
            let profile = model.profile(&prompt.profile_id).cloned();
            model.open_form(profile.as_ref());
            state.edit_pin = Some((prompt.profile_id, prompt.pin));
        }
        PinPurpose::Delete => model.confirm_delete = Some(prompt.profile_id),
    }
}

fn offline_check(storage: &Storage, prompt: &PinPrompt) -> Result<(), PinFailure> {
    let mut profiles = stored_profiles(storage);
    let Some(stored) = profiles
        .iter_mut()
        .find(|profile| profile.get("id").and_then(Value::as_str) == Some(&prompt.profile_id))
    else {
        return Err(PinFailure::Unavailable);
    };
    let Some(result) = core_value(
        "nuvioPinOffline",
        json!({
            "profile": stored.clone(),
            "pin": prompt.pin,
            "now": chrono::Utc::now().timestamp(),
        }),
    ) else {
        return Err(PinFailure::Unavailable);
    };
    if let Some(next) = result.get("profile") {
        *stored = next.clone();
        write(storage, "profiles", &Value::Array(profiles));
    }
    let retry_after = result
        .get("retryAfter")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    match result.get("outcome").and_then(Value::as_str) {
        Some("unlocked") => Ok(()),
        Some("incorrect") => Err(PinFailure::Incorrect),
        Some("locked") => Err(PinFailure::Locked(retry_after)),
        _ => Err(PinFailure::Unavailable),
    }
}

pub(crate) fn poll_pin(state: &mut RendererState) {
    poll_pin_change(state);
    let Some(job) = state.pin_job.as_ref() else {
        return;
    };
    let Ok(outcome) = job.receiver.try_recv() else {
        return;
    };
    let Some(PinJob { prompt, .. }) = state.pin_job.take() else {
        return;
    };
    let Some(storage) = state
        .session
        .as_ref()
        .map(|session| session.storage().clone())
    else {
        return;
    };
    let Some(model) = state.profiles.as_mut() else {
        return;
    };
    model.busy = false;
    let verdict = match outcome {
        Ok(result) if result.get("unlocked").and_then(Value::as_bool) == Some(true) => Ok(()),
        Ok(result) => {
            let wait = result
                .get("retry_after_seconds")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            Err(if wait > 0 {
                PinFailure::Locked(wait)
            } else {
                PinFailure::Incorrect
            })
        }
        Err(reason) if reason == fluxa_effects::NUVIO_OFFLINE => offline_check(&storage, &prompt),
        Err(_) => Err(PinFailure::Incorrect),
    };
    match verdict {
        Ok(()) => unlock(state, &storage, prompt),
        Err(failure) => {
            model.pin_prompt = Some(PinPrompt {
                pin: String::new(),
                error: Some(failure),
                ..prompt
            });
        }
    }
}

fn poll_pin_change(state: &mut RendererState) {
    let Some(job) = state.pin_change_job.as_ref() else {
        return;
    };
    let Ok(outcome) = job.receiver.try_recv() else {
        return;
    };
    state.pin_change_job = None;
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let storage = session.storage().clone();
    let Some(model) = state.profiles.as_mut() else {
        return;
    };
    reload(model, &storage);
    if let Err(key) = outcome {
        model.notice = Some(localized(&key, &model.language));
    }
}

pub(crate) fn poll(state: &mut RendererState) {
    poll_catalog(state);
    poll_pin(state);
    let Some(job) = state.pack_job.as_ref() else {
        return;
    };
    let Ok(results) = job.receiver.try_recv() else {
        return;
    };
    let adding = job.adding;
    state.pack_job = None;
    let Some(storage) = state
        .session
        .as_ref()
        .map(|session| session.storage().clone())
    else {
        return;
    };
    let Some(model) = state.profiles.as_mut() else {
        return;
    };
    model.busy = false;
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut added = 0;
    let mut duplicate = false;
    let mut failure = None;
    for (repository, result) in results {
        match result {
            Ok(packs) => {
                if adding {
                    let fresh = packs
                        .into_iter()
                        .filter(|pack| {
                            !model
                                .picker
                                .avatar_packs
                                .iter()
                                .any(|known| known.id == pack.id)
                        })
                        .collect::<Vec<_>>();
                    duplicate = fresh.is_empty();
                    added += fresh.len();
                    model.picker.avatar_packs.extend(fresh);
                } else {
                    model
                        .picker
                        .avatar_packs
                        .retain(|pack| pack.repository_url != repository);
                    model.picker.avatar_packs.extend(packs);
                }
            }
            Err(reason) => failure = Some(reason),
        }
    }
    save_picker(model, &storage);
    model.notice = if let Some(reason) = failure.filter(|_| adding) {
        let key = format!("profiles.avatar_pack_error_{reason}");
        let message = t(&key);
        Some(if message == key {
            t("profiles.avatar_pack_error_generic")
        } else {
            message
        })
    } else if adding && duplicate {
        Some(t("profiles.avatar_pack_already_added_message"))
    } else if adding {
        model.repository_input.clear();
        Some(t("profiles.avatar_pack_added_message").replacen("%s", &added.to_string(), 1))
    } else {
        None
    };
}
