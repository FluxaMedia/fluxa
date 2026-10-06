use super::*;

#[derive(Clone, Copy)]
pub(super) struct PlayerTokens {
    pub card_width: f32,
    pub pill: f32,
    pub text: f32,
    pub title: f32,
    pub pad: f32,
    pub menu_width: f32,
    pub list_row: f32,
    pub list_text: f32,
    pub list_pad: f32,
    pub list_width: f32,
    pub rich_row: f32,
    pub rich_width: f32,
    pub toast_top: f32,
}

impl PlayerTokens {
    pub fn for_form_factor(form_factor: UiFormFactor) -> Self {
        match form_factor {
            UiFormFactor::Tv => Self {
                card_width: 500.0,
                pill: 60.0,
                text: 22.0,
                title: 24.0,
                pad: 24.0,
                menu_width: 560.0,
                list_row: 58.0,
                list_text: 21.0,
                list_pad: 26.0,
                list_width: 400.0,
                rich_row: 128.0,
                rich_width: 760.0,
                toast_top: 48.0,
            },
            UiFormFactor::Mobile => Self {
                card_width: 300.0,
                pill: 42.0,
                text: 14.0,
                title: 15.0,
                pad: 14.0,
                menu_width: 320.0,
                list_row: 44.0,
                list_text: 15.0,
                list_pad: 14.0,
                list_width: 250.0,
                rich_row: 88.0,
                rich_width: 420.0,
                toast_top: 30.0,
            },
            UiFormFactor::Desktop => Self {
                card_width: 380.0,
                pill: 44.0,
                text: 15.0,
                title: 17.0,
                pad: 18.0,
                menu_width: 400.0,
                list_row: 38.0,
                list_text: 14.0,
                list_pad: 12.0,
                list_width: 270.0,
                rich_row: 92.0,
                rich_width: 460.0,
                toast_top: 28.0,
            },
        }
    }
}
