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
    pub stacked_row: f32,
    pub rich_row: f32,
    pub rich_width: f32,
    pub toast_top: f32,
    pub icon: f32,
    pub volume_width: f32,
    pub play_boost: f32,
    pub stats_width: f32,
    pub stats_text: f32,
    pub stats_pad: f32,
    pub card_lift_visible: f32,
    pub card_lift_hidden: f32,
    pub header_top: f32,
    pub header_title: f32,
    pub header_sub: f32,
    pub seek_track: f32,
    pub seek_thickness: f32,
    pub times_width: f32,
    pub times_text: f32,
    pub time_row: f32,
    pub label_text: f32,
    pub volume_track: f32,
    pub volume_reach: f32,
    pub center_play: f32,
    pub center_skip: f32,
    pub skip_text: f32,
    pub controls_bottom: f32,
    pub icon_small: f32,
    pub side_icons: f32,
    pub glow: f32,
    pub scrim_top_alpha: f32,
    pub scrim_bottom_alpha: f32,
    pub margin: crate::metrics::Scaled,
    pub scrim_top: crate::metrics::Scaled,
    pub scrim_bottom: crate::metrics::Scaled,
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
        let scaled = |key: &str| {
            layout["platforms"][platform]["scaled"][key]
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
                })
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
            stacked_row: number("stackedRow"),
            rich_row: number("richRow"),
            rich_width: number("richWidth"),
            toast_top: number("toastTop"),
            icon: number("icon"),
            volume_width: number("volumeWidth"),
            play_boost: number("playBoost"),
            stats_width: number("statsWidth"),
            stats_text: number("statsText"),
            stats_pad: number("statsPad"),
            card_lift_visible: number("cardLiftVisible"),
            card_lift_hidden: number("cardLiftHidden"),
            header_top: number("headerTop"),
            header_title: number("headerTitle"),
            header_sub: number("headerSub"),
            seek_track: number("seekTrack"),
            seek_thickness: number("seekThickness"),
            times_width: number("timesWidth"),
            times_text: number("timesText"),
            time_row: number("timeRow"),
            label_text: number("labelText"),
            volume_track: number("volumeTrack"),
            volume_reach: number("volumeReach"),
            center_play: number("centerPlay"),
            center_skip: number("centerSkip"),
            skip_text: number("skipText"),
            controls_bottom: number("controlsBottom"),
            icon_small: number("iconSmall"),
            side_icons: number("sideIcons"),
            glow: number("glow"),
            scrim_top_alpha: number("scrimTopAlpha"),
            scrim_bottom_alpha: number("scrimBottomAlpha"),
            margin: scaled("playerMargin"),
            scrim_top: scaled("playerScrimTop"),
            scrim_bottom: scaled("playerScrimBottom"),
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
                tokens.stacked_row,
                tokens.rich_row,
                tokens.rich_width,
                tokens.toast_top,
                tokens.icon,
                tokens.volume_width,
                tokens.play_boost,
                tokens.stats_width,
                tokens.stats_text,
                tokens.stats_pad,
                tokens.card_lift_visible,
                tokens.card_lift_hidden,
                tokens.header_top,
                tokens.header_title,
                tokens.header_sub,
                tokens.seek_track,
                tokens.seek_thickness,
                tokens.times_width,
                tokens.times_text,
                tokens.time_row,
                tokens.label_text,
                tokens.volume_track,
                tokens.volume_reach,
                tokens.center_play,
                tokens.center_skip,
                tokens.skip_text,
                tokens.controls_bottom,
                tokens.icon_small,
                tokens.side_icons,
                tokens.glow,
                tokens.scrim_top_alpha,
                tokens.scrim_bottom_alpha,
            ] {
                assert!(value > 0.0, "{form_factor:?}");
            }
        }
    }
}
