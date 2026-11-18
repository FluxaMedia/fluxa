#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelRow {
    pub label: String,
    pub value: Option<String>,
    pub primary: bool,
    pub selected: bool,
    pub heading: bool,
    pub group: bool,
    pub switch: Option<bool>,
    pub thumbnail: Option<String>,
    pub detail: Option<String>,
    pub frame: Option<f64>,
    pub badge: Option<String>,
    pub icon: Option<String>,
    pub chevron: bool,
    pub step: Option<PanelStep>,
    pub sync: Option<SyncRole>,
    pub subline: Option<String>,
    pub separated: bool,
    pub back: bool,
    pub chips: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncRole {
    Back,
    Reset,
    Reading,
    Minus,
    Plus,
    Auto,
    Method,
    Choice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelStep {
    Value,
    Minus,
    Plus,
    Swatches(usize),
    Swatch,
    Choices(usize),
    Choice,
}

impl PanelStep {
    pub(super) fn follows_lead(self) -> bool {
        matches!(self, Self::Minus | Self::Plus | Self::Swatch | Self::Choice)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerPanel {
    pub title: String,
    pub message: Option<String>,
    pub rows: Vec<PanelRow>,
    pub list: bool,
    pub look: Option<crate::SubtitleLook>,
    pub chips: bool,
}
