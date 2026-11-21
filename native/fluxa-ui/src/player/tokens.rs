use super::*;

#[derive(Clone, Copy)]
pub(super) struct PlayerTokens {
    pub pill: f32,
    pub icon: f32,
    pub volume_width: f32,
    pub stats_rows: f32,
    pub stats_columns: usize,
}

impl PlayerTokens {
    pub fn for_form_factor(form_factor: UiFormFactor) -> Self {
        let form = match form_factor {
            UiFormFactor::Mobile => "mobile",
            UiFormFactor::Desktop => "desktop",
            UiFormFactor::Tv => "tv",
        };
        let number = |key: &str| crate::vars::px(&format!("player-{key}-{form}"));
        Self {
            pill: number("pill"),
            icon: number("icon"),
            volume_width: number("volume-width"),
            stats_rows: number("stats-rows"),
            stats_columns: number("stats-columns") as usize,
        }
    }

    pub fn margin(form_factor: UiFormFactor, basis: f32) -> f32 {
        let form = match form_factor {
            UiFormFactor::Mobile => "mobile",
            UiFormFactor::Desktop => "desktop",
            UiFormFactor::Tv => "tv",
        };
        crate::vars::scaled(&format!("player-margin-{form}"), basis)
    }
}

pub fn subtitle_lift(form_factor: UiFormFactor) -> f32 {
    PlayerTokens::for_form_factor(form_factor).icon * 2.0 + Space::S32.px()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_platform_defines_every_player_token() {
        for form_factor in [
            UiFormFactor::Mobile,
            UiFormFactor::Desktop,
            UiFormFactor::Tv,
        ] {
            let tokens = PlayerTokens::for_form_factor(form_factor);
            for value in [
                tokens.pill,
                tokens.icon,
                tokens.volume_width,
                tokens.stats_rows,
                tokens.stats_columns as f32,
            ] {
                assert!(value > 0.0, "{form_factor:?}");
            }
        }
    }
}
