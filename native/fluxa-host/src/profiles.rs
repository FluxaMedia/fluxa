use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use fluxa_effects::{EffectExecutor, SessionHandle, Storage};
use fluxa_ui::{
    PinPrompt, PinPurpose, ProfileAvatarPack, ProfileEntry, ProfilePickerSettings, ProfilesMode,
    ProfilesModel, ProfilesRequest, localized,
};
use serde_json::{Value, json};

use crate::{RendererState, core_value, discover_command, host_log, profile_language};

pub type ImagePicker = Box<dyn Fn() -> Option<PathBuf> + Send>;

pub(crate) struct PackJob {
    adding: bool,
    receiver: Receiver<Vec<(String, Result<Vec<ProfileAvatarPack>, String>)>>,
}

fn read(storage: &Storage, key: &str) -> Option<Value> {
    storage.read_json(key).ok().flatten()
}

fn write(storage: &Storage, key: &str, value: &Value) {
    if let Err(error) = storage.write_json(key, value) {
        host_log(format!("Storage write {key} failed: {error}"));
    }
}

fn stored_profiles(storage: &Storage) -> Vec<Value> {
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
        locked: text("pinHash").is_some(),
        uses_primary_addons: flag("usesPrimaryAddons"),
        uses_primary_plugins: flag("usesPrimaryPlugins"),
    }
}

fn load_model(storage: &Storage, language: String) -> ProfilesModel {
    let profiles = stored_profiles(storage);
    let primary_id = core_value("primaryProfileId", Value::Array(profiles.clone()))
        .and_then(|value| value.as_str().map(ToOwned::to_owned));
    let picker = picker_settings(storage);
    ProfilesModel {
        profiles: profiles.iter().map(entry).collect(),
        primary_id,
        background_input: picker.background_url.clone().unwrap_or_default(),
        picker,
        language,
        ..ProfilesModel::default()
    }
}

pub(crate) fn should_pick_on_start(storage: &Storage) -> bool {
    let profiles = stored_profiles(storage);
    let active = read(storage, "active_profile_id")
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_default();
    !profiles.is_empty()
        && !profiles
            .iter()
            .any(|profile| profile.get("id").and_then(Value::as_str) == Some(active.as_str()))
}

pub(crate) fn open(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let language = profile_language(&session.active_profile());
    let model = load_model(session.storage(), language);
    state.picker_background = model.picker.background_url.clone();
    state.profiles = Some(model);
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
                    error: false,
                });
            } else {
                activate(state, &storage, &id);
            }
        }
        ProfilesRequest::SubmitPin => {
            let Some(prompt) = model.pin_prompt.take() else {
                return;
            };
            let profile = stored_profiles(&storage)
                .into_iter()
                .find(|profile| profile.get("id").and_then(Value::as_str) == Some(&prompt.profile_id));
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
                    error: true,
                    ..prompt
                });
                return;
            }
            match prompt.purpose {
                PinPurpose::Enter => activate(state, &storage, &prompt.profile_id),
                PinPurpose::Edit => {
                    let profile = model.profile(&prompt.profile_id).cloned();
                    model.open_form(profile.as_ref());
                }
                PinPurpose::Delete => model.confirm_delete = Some(prompt.profile_id),
            }
        }
        ProfilesRequest::Save => save(model, &storage),
        ProfilesRequest::Delete(id) => {
            let profiles = stored_profiles(&storage);
            if let Some(next) = core_value(
                "profileMutationPlan",
                json!({"operation": "delete", "profiles": profiles, "id": id}),
            ) {
                write(&storage, "profiles", &next);
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

fn save(model: &mut ProfilesModel, storage: &Storage) {
    if !model.can_save() {
        return;
    }
    let profiles = stored_profiles(storage);
    let draft = model.draft.clone();
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
        return;
    };
    fields.insert("name".into(), json!(draft.name.trim()));
    match &draft.avatar_url {
        Some(url) => fields.insert("avatarUrl".into(), json!(url)),
        None => fields.remove("avatarUrl"),
    };
    if !draft.pin.is_empty() {
        if let Some(hash) = core_value("profilePinHash", json!({"pin": draft.pin})) {
            fields.insert("pinHash".into(), hash);
        }
    } else if draft.remove_pin {
        fields.remove("pinHash");
    }
    fields.insert("usesPrimaryAddons".into(), json!(draft.uses_primary_addons));
    fields.insert("usesPrimaryPlugins".into(), json!(draft.uses_primary_plugins));
    if let Some(next) = core_value(
        "profileMutationPlan",
        json!({"operation": "save", "profiles": profiles, "profile": profile}),
    ) {
        write(storage, "profiles", &next);
    }
    reload(model, storage);
    model.mode = ProfilesMode::Select;
}

fn new_profile_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("{nanos:x}-{:x}", std::process::id())
}

fn activate(state: &mut RendererState, storage: &Storage, id: &str) {
    write(storage, "active_profile_id", &json!(id));
    match SessionHandle::open(storage.clone()) {
        Ok(session) => {
            if let Err(error) = load_profile(&session) {
                host_log(format!("Profile load failed: {error}"));
            }
            state.session = Some(session);
            state.session_revision = None;
            state.last_snapshot_revision = None;
            state.core_snapshot = None;
            state.route = "home".to_owned();
            state.profiles = None;
            state.pack_job = None;
        }
        Err(error) => host_log(format!("Profile switch failed: {error}")),
    }
}

pub(crate) fn load_profile(session: &SessionHandle) -> Result<(), String> {
    let profile = session.active_profile();
    session.dispatch(json!({
        "type": "homeLoadRequested",
        "profile": profile,
        "language": profile_language(&profile),
        "force": true,
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
        session.dispatch(json!({"type": "addonInstallRequested", "transportUrl": url, "forceRefresh": true}))?;
    }
    write(storage, &key, &json!([]));
    Ok(())
}

fn legacy_addons_key(profile_id: &str) -> String {
    format!("legacy_addons_{profile_id}")
}

pub fn import_legacy(data_dir: PathBuf, legacy: &str) -> Result<bool, String> {
    let storage = Storage::open(data_dir)?;
    if storage.read_json("profiles")?.is_some() {
        return Ok(false);
    }
    let legacy: Value = serde_json::from_str(legacy).map_err(|error| error.to_string())?;
    let profiles = legacy
        .get("profiles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for profile in &profiles {
        let (Some(id), Some(addons)) = (
            profile.get("id").and_then(Value::as_str),
            profile.get("localAddons").filter(|addons| addons.is_array()),
        ) else {
            continue;
        };
        storage.write_json(&legacy_addons_key(id), addons)?;
    }
    storage.write_json("profiles", &Value::Array(profiles))?;
    if let Some(active) = legacy.get("activeProfileId").filter(|id| id.is_string()) {
        storage.write_json("active_profile_id", active)?;
    }
    if let Some(mut picker) = legacy.get("pickerSettings").cloned() {
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
    fetch: &impl Fn(&str) -> Option<Value>,
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
    fetch: &impl Fn(&str) -> Option<Value>,
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
    let packs = manifests
        .iter()
        .filter_map(|manifest| parse_pack(fetch, manifest, repository))
        .collect::<Vec<_>>();
    if packs.is_empty() {
        return Err("no_valid_avatars".to_owned());
    }
    Ok(packs)
}

pub(crate) fn poll(state: &mut RendererState) {
    let Some(job) = state.pack_job.as_ref() else {
        return;
    };
    let Ok(results) = job.receiver.try_recv() else {
        return;
    };
    let adding = job.adding;
    state.pack_job = None;
    let Some(storage) = state.session.as_ref().map(|session| session.storage().clone()) else {
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
                        .filter(|pack| !model.picker.avatar_packs.iter().any(|known| known.id == pack.id))
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

