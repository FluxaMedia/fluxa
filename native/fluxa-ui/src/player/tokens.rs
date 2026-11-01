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
    pub icon: f32,
    pub stats_width: f32,
    pub stats_text: f32,
    pub stats_pad: f32,
    pub card_lift_visible: f32,
    pub card_lift_hidden: f32,
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
        Self {
            card_width: number("cardWidth"),
            pill: number("pill"),
            text: number("text"),
            title: number("title"),
            pad: number("pad"),
            menu_width: number("menuWidth"),
            list_row: number("listRow"),
            list_text: number("listText"),
            list_pad: number("listPad"),
            list_width: number("listWidth"),
            rich_row: number("richRow"),
            rich_width: number("richWidth"),
            toast_top: number("toastTop"),
            icon: number("icon"),
            stats_width: number("statsWidth"),
            stats_text: number("statsText"),
            stats_pad: number("statsPad"),
            card_lift_visible: number("cardLiftVisible"),
            card_lift_hidden: number("cardLiftHidden"),
        }
    }
}

pub fn subtitle_lift(form_factor: UiFormFactor) -> f32 {
    let tokens = PlayerTokens::for_form_factor(form_factor);
    tokens.card_lift_visible - tokens.card_lift_hidden
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
                tokens.card_width,
                tokens.pill,
                tokens.text,
                tokens.title,
                tokens.pad,
                tokens.menu_width,
                tokens.list_row,
                tokens.list_text,
                tokens.list_pad,
                tokens.list_width,
                tokens.rich_row,
                tokens.rich_width,
                tokens.toast_top,
                tokens.icon,
                tokens.stats_width,
                tokens.stats_text,
                tokens.stats_pad,
                tokens.card_lift_visible,
                tokens.card_lift_hidden,
            ] {
                assert!(value > 0.0, "{form_factor:?}");
            }
        }
    }
}
