use super::*;

pub(super) const FINGERPRINT_CARDS: usize = 60;

pub(super) fn hash_json(value: &Value, hasher: &mut impl Hasher) {
    match value {
        Value::Null => 0u8.hash(hasher),
        Value::Bool(flag) => (1u8, *flag).hash(hasher),
        Value::Number(number) => (2u8, number.as_f64().map(f64::to_bits)).hash(hasher),
        Value::String(text) => (3u8, text).hash(hasher),
        Value::Array(items) => {
            (4u8, items.len()).hash(hasher);
            for item in items {
                hash_json(item, hasher);
            }
        }
        Value::Object(map) => {
            (5u8, map.len()).hash(hasher);
            for (key, item) in map {
                key.hash(hasher);
                hash_json(item, hasher);
            }
        }
    }
}

pub(super) fn json_key(value: &Value) -> u64 {
    let mut hasher = fluxa_ui::FastHasher::default();
    hash_json(value, &mut hasher);
    hasher.finish()
}

pub(super) fn manager_fingerprint(hasher: &mut impl Hasher) {
    if let Ok(slot) = ACTIVE.read() {
        slot.as_ref().map(|skin| &skin.manifest.id).hash(hasher);
    }
    if let Ok(manager) = MANAGER.lock() {
        (&manager.installed, &manager.message, manager.busy).hash(hasher);
    }
}

pub(super) fn fingerprint(
    screen: &str,
    skin: &Skin,
    home: &HomeModel,
    settings: &SettingsModel,
    fields: &HashMap<String, String>,
    nav: &Value,
    extra: &(u64, u64, u64),
) -> u64 {
    let mut hasher = fluxa_ui::FastHasher::default();
    (screen, &skin.manifest.id, &skin.manifest.version).hash(&mut hasher);
    (
        &home.title,
        &home.eyebrow,
        &home.description,
        &home.background_url,
        &home.logo_url,
        &home.item_id,
        &home.profile_name,
        &home.profile_avatar_url,
        home.is_loading,
        home.show_hero_section,
    )
        .hash(&mut hasher);
    std::mem::discriminant(&home.form_factor).hash(&mut hasher);
    extra.hash(&mut hasher);
    for row in &home.rows {
        (&row.id, &row.title, row.cards.len()).hash(&mut hasher);
        for card in row.cards.iter().take(FINGERPRINT_CARDS) {
            (
                &card.id,
                &card.title,
                &card.subtitle,
                &card.artwork_url,
                card.progress.to_bits(),
            )
                .hash(&mut hasher);
        }
    }
    hash_json(&settings.values, &mut hasher);
    (settings.active_section, settings.page_open).hash(&mut hasher);
    hash_json(nav, &mut hasher);
    manager_fingerprint(&mut hasher);
    let mut names: Vec<_> = fields.iter().collect();
    names.sort();
    names.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
pub(crate) enum Extra<'a> {
    None,
    Profiles(&'a fluxa_ui::ProfilesModel),
    Player(&'a fluxa_ui::PlayerModel),
    Folder(&'a fluxa_ui::FolderModel, usize),
    Guide(&'a fluxa_ui::GuideModel),
    Welcome(&'a fluxa_ui::WelcomeModel),
    Menu(&'a crate::card_menu::MenuView),
    Toast(&'a str),
    PlayerToast(&'a Value),
    Tip(&'a fluxa_ui::HoverTip),
    Update(&'a str),
    Window(bool),
    Mini(&'a fluxa_ui::PlayerModel),
    Watch(&'a fluxa_ui::WatchTogetherModel),
    Modal,
    Dialog(&'a Value),
    Sources(&'a fluxa_ui::PlayerModel),
    Recommendations(&'a fluxa_ui::PlayerModel),
}

impl<'a> Extra<'a> {
    pub(crate) fn for_route(
        route: Route,
        folder: &'a fluxa_ui::FolderModel,
        tab: usize,
        guide: &'a fluxa_ui::GuideModel,
    ) -> Self {
        if route == Route::Folder {
            Self::Folder(folder, tab)
        } else if route == Route::Live {
            Self::Guide(guide)
        } else {
            Self::None
        }
    }
}
