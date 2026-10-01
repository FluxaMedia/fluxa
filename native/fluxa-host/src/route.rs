use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum HomeScrollTarget {
    Vertical,
    Horizontal(usize),
    ScreenVertical,
    DetailRow(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Route {
    #[default]
    Home,
    Library,
    Discover,
    Calendar,
    Detail,
    Settings,
    Profiles,
    Player,
}

impl Route {
    pub(crate) fn parse(route: &str) -> Self {
        match route {
            "library" => Self::Library,
            "discover" => Self::Discover,
            "calendar" => Self::Calendar,
            "detail" => Self::Detail,
            "settings" => Self::Settings,
            "profiles" => Self::Profiles,
            "player" => Self::Player,
            _ => Self::Home,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Library => "library",
            Self::Discover => "discover",
            Self::Calendar => "calendar",
            Self::Detail => "detail",
            Self::Settings => "settings",
            Self::Profiles => "profiles",
            Self::Player => "player",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(super) enum NativeAction {
    CoreCommand {
        command: Value,
    },
    Navigate {
        destination: Route,
    },
    Detail {
        id: String,
        item_type: String,
        #[serde(default)]
        preview: Value,
    },
    Play {
        id: String,
        item_type: String,
    },
    PlayTrailer {
        item: Value,
        urls: Vec<String>,
    },
    StartPlayback {
        item: Value,
    },
    ToggleWatchlist {
        item: Value,
    },
    SettingsChange {
        key: String,
        value: Value,
    },
    SettingsSection {
        index: usize,
    },
    AccountToggle {
        provider: String,
    },
    MediaServer {
        node: u64,
    },
    MediaCommand {
        command: String,
        #[serde(default)]
        value: f64,
    },
    OauthCallback {
        url: String,
    },
    OpenFile {
        uri: String,
        title: String,
    },
    DiscoverType {
        content_type: String,
    },
    DiscoverCatalog {
        content_type: String,
        catalog_key: String,
        extra_name: String,
        extra_value: String,
        query: String,
    },
    DiscoverFilters {
        content_type: String,
        catalog_key: String,
        extra_name: String,
        extra_value: String,
        query: String,
    },
    CalendarMonth {
        year: i32,
        month: i32,
    },
    Back,
    LoadMore {
        row_id: String,
    },
}

pub(super) const NODE_HOME: u64 = 10;
pub(super) const NODE_LIBRARY: u64 = 11;
pub(super) const NODE_DISCOVER: u64 = 12;
pub(super) const NODE_CALENDAR: u64 = 13;
pub(super) const NODE_PROFILE: u64 = 14;
pub(super) const NODE_PLAY: u64 = 20;
pub(super) const NODE_MORE_INFO: u64 = 21;
pub(super) const NODE_CARD_BASE: u64 = 100;
