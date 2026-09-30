use super::*;

#[derive(Clone, Debug, Default)]
pub struct ProfileEntry {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub locked: bool,
    pub uses_primary_addons: bool,
    pub uses_primary_plugins: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ProfileAvatar {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAvatarPack {
    pub id: String,
    pub repository_url: String,
    pub title: String,
    pub manifest_url: String,
    pub avatars: Vec<ProfileAvatar>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProfilePickerSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_url: Option<String>,
    pub avatar_packs: Vec<ProfileAvatarPack>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfilesMode {
    #[default]
    Select,
    Form,
    Settings,
}

#[derive(Clone, Debug, Default)]
pub struct ProfileDraft {
    pub editing: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub pin: String,
    pub has_pin: bool,
    pub remove_pin: bool,
    pub uses_primary_addons: bool,
    pub uses_primary_plugins: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinPurpose {
    Enter,
    Edit,
    Delete,
}

#[derive(Clone, Debug)]
pub struct PinPrompt {
    pub profile_id: String,
    pub purpose: PinPurpose,
    pub pin: String,
    pub error: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ProfilesModel {
    pub profiles: Vec<ProfileEntry>,
    pub primary_id: Option<String>,
    pub mode: ProfilesMode,
    pub draft: ProfileDraft,
    pub pin_prompt: Option<PinPrompt>,
    pub confirm_delete: Option<String>,
    pub picker: ProfilePickerSettings,
    pub background_input: String,
    pub repository_input: String,
    pub busy: bool,
    pub notice: Option<String>,
    pub language: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProfilesRequest {
    Select(String),
    SubmitPin,
    Save,
    Delete(String),
    AddRepository,
    RefreshRepository(String),
    RefreshAllPacks,
    RemovePack(String),
    SaveBackground,
    PickAvatarImage,
    PickBackgroundImage,
}

impl ProfilesModel {
    pub fn profile(&self, id: &str) -> Option<&ProfileEntry> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn open_form(&mut self, profile: Option<&ProfileEntry>) {
        self.draft = match profile {
            Some(profile) => ProfileDraft {
                editing: Some(profile.id.clone()),
                name: profile.name.clone(),
                avatar_url: profile.avatar_url.clone(),
                has_pin: profile.locked,
                uses_primary_addons: profile.uses_primary_addons,
                uses_primary_plugins: profile.uses_primary_plugins,
                ..ProfileDraft::default()
            },
            None => ProfileDraft::default(),
        };
        self.mode = ProfilesMode::Form;
    }

    pub(super) fn duplicate_name(&self) -> bool {
        let name = self.draft.name.trim().to_lowercase();
        !name.is_empty()
            && self.profiles.iter().any(|profile| {
                Some(&profile.id) != self.draft.editing.as_ref()
                    && profile.name.trim().to_lowercase() == name
            })
    }

    pub(super) fn draft_is_primary(&self) -> bool {
        match &self.draft.editing {
            Some(id) => self.primary_id.as_ref() == Some(id),
            None => self.profiles.is_empty(),
        }
    }

    pub fn can_save(&self) -> bool {
        !self.draft.name.trim().is_empty()
            && !self.duplicate_name()
            && !self.busy
            && (self.draft.pin.is_empty() || self.draft.pin.len() == 4)
    }
}
