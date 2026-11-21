use super::*;

#[derive(Clone, Debug)]
pub struct ChoiceRequest {
    pub key: String,
    pub title: String,
    pub options: Vec<(String, String)>,
    pub selected: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerGesture {
    Volume(f32),
    VolumeSet(f32),
    Brightness(f32),
}

#[derive(Clone, Debug, Default)]
pub struct HomeLayout {
    pub choices: Vec<(u64, ChoiceRequest)>,
    pub focusable: Vec<(u64, Rect)>,
    pub neighbors: Vec<(u64, crate::Dir, u64)>,
    pub activated: Option<u64>,
    pub hovered: Option<(u64, Rect)>,
    pub text_input: Option<String>,
    pub text_input_node: Option<u64>,
    pub setting_change: Option<(String, serde_json::Value)>,
    pub filter_change: Option<(String, String)>,
    /// Core page requests generated when a shelf approaches its loaded end.
    pub load_more: Vec<serde_json::Value>,
    pub seek_to: Option<f64>,
    pub seek_hover: Option<f64>,
    pub filmstrip: Option<Vec<f64>>,
    pub player_gesture: Option<PlayerGesture>,
    pub player_speed_hold: bool,
    pub profiles: Option<ProfilesRequest>,
    pub welcome: Option<WelcomeRequest>,
    pub scroll_max: Option<f32>,
}

pub fn home_row_of_card(home: &HomeModel, card: usize) -> Option<usize> {
    let mut start = 0;
    for (index, (_, cards, _)) in home.content_rows_with_kind().enumerate() {
        if card < start + cards.len() {
            return Some(index);
        }
        start += cards.len();
    }
    None
}
