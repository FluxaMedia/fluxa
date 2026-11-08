use super::*;

#[derive(Clone, Copy)]
pub(super) struct PlayerTokens {
    pub pill: f32,
    pub icon: f32,
    pub volume_width: f32,
    pub margin: crate::metrics::Scaled,
}

impl PlayerTokens {
    pub fn for_form_factor(form_factor: UiFormFactor) -> Self {
        static CACHE: std::sync::RwLock<[Option<(u64, PlayerTokens)>; 3]> =
            std::sync::RwLock::new([None; 3]);
        let slot = match form_factor {
            UiFormFactor::Mobile => 0,
            UiFormFactor::Desktop => 1,
            UiFormFactor::Tv => 2,
        };
        let epoch = fluxa_theme::theme::active_epoch();
        if let Some((cached, tokens)) = CACHE.read().ok().and_then(|cache| cache[slot])
            && cached == epoch
        {
            return tokens;
        }
        let tokens = Self::load(form_factor);
        if let Ok(mut cache) = CACHE.write() {
            cache[slot] = Some((epoch, tokens));
        }
        tokens
    }

    fn load(form_factor: UiFormFactor) -> Self {
        let platform = match form_factor {
            UiFormFactor::Mobile => "mobile",
            UiFormFactor::Desktop => "desktop",
            UiFormFactor::Tv => "tv",
        };
        let layout = fluxa_theme::theme::active_shared_ui_tokens()
            .map(|tokens| tokens.layout)
            .unwrap_or_default();
        let number = |key: &str| {
            layout["platforms"][platform]["player"][key]
                .as_f64()
                .unwrap_or(0.0) as f32
        };
        let margin = layout["platforms"][platform]["scaled"]["playerMargin"]
            .as_array()
            .and_then(|values| {
                let at = |index: usize| values.get(index)?.as_f64().map(|v| v as f32);
                Some(crate::metrics::Scaled {
                    ratio: at(0)?,
                    min: at(1)?,
                    max: at(2)?,
                    offset: 0.0,
                })
            })
            .unwrap_or(crate::metrics::Scaled {
                ratio: 0.0,
                min: 0.0,
                max: 0.0,
                offset: 0.0,
            });
        Self {
            pill: number("pill"),
            icon: number("icon"),
            volume_width: number("volumeWidth"),
            margin,
        }
    }
}

pub(super) fn controls_top_id() -> Id {
    Id::new("fluxa-player-controls-top")
}

pub fn subtitle_lift(form_factor: UiFormFactor) -> f32 {
    PlayerTokens::for_form_factor(form_factor).icon * 2.0 + Space::S32.px(&crate::metrics::active())
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
                tokens.margin.max,
            ] {
                assert!(value > 0.0, "{form_factor:?}");
            }
        }
    }
}
