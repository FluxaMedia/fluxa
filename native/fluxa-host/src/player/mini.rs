use super::*;

impl PlayerSession {
    pub(crate) fn set_volume_pin(&mut self, volume: f64) {
        self.volume_pin = Some(volume);
    }

    pub(super) fn pin_volume(&mut self) {
        if let Some(volume) = self.volume_pin {
            self.status.volume = volume;
        }
    }

    pub(super) fn filmstrip_step(&self) -> f64 {
        self.trickplay
            .as_ref()
            .and_then(|trickplay| trickplay.step())
            .or_else(|| {
                self.seek_previews
                    .as_ref()
                    .map(|previews| previews.interval())
            })
            .unwrap_or(10.0)
    }
}

pub(crate) fn want_filmstrip(state: &mut RendererState, slots: Option<Vec<f64>>) {
    let enabled = state.settings.bool_value("seekThumbnailEnabled");
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let chapters_open = matches!(player.panel, Some(Panel::Menu(menu::Menu::Chapters)));
    if chapters_open {
        let starts = player.status.chapters.iter().map(|(start, _)| *start);
        player.filmstrip.want_chapters(starts.collect());
    } else if enabled {
        player.filmstrip.want(slots);
    }
}

pub(crate) fn hover_seek(state: &mut RendererState, time: Option<f64>) {
    if !state.settings.bool_value("seekThumbnailEnabled") {
        return;
    }
    let (Some(player), Some(video)) = (state.player.as_mut(), state.video.as_mut()) else {
        return;
    };
    let Some(time) = time else {
        return;
    };
    player.touch();
    if player.filmstrip.active() {
        return;
    }
    if let Some(trickplay) = player
        .trickplay
        .as_mut()
        .filter(|trickplay| trickplay.usable())
    {
        trickplay.request(time);
        return;
    }
    let time = (time * 2.0).round() / 2.0;
    if player
        .thumbnail_requested
        .is_some_and(|(last, at)| last == time || at.elapsed() < THUMBNAIL_INTERVAL)
    {
        return;
    }
    player.thumbnail_requested = Some((time, Instant::now()));
    video.request_thumbnail(time);
}

pub(crate) fn is_mini_node(node: u64) -> bool {
    matches!(
        node,
        fluxa_ui::NODE_PLAYER_TOGGLE | fluxa_ui::NODE_PLAYER_CLOSE | fluxa_ui::NODE_PLAYER_EXPAND
    )
}

impl PlayerSession {
    pub(crate) fn can_pip(&self) -> bool {
        self.status.has_frame && self.trailer.is_none() && self.error.is_none()
    }

    pub(crate) fn can_minimize(&self) -> bool {
        self.status.has_frame
            && self.trailer.is_none()
            && self.recommendations.is_empty()
            && self.panel.is_none()
    }
}

pub(crate) fn minimize(state: &mut RendererState) {
    let Some(player) = state.player.as_mut().filter(|player| player.can_minimize()) else {
        return;
    };
    player.minimized = true;
    player.hide();
    crate::reset_ui(state);
}

pub(crate) fn expand(state: &mut RendererState) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.minimized = false;
    player.touch();
    crate::reset_ui(state);
}
