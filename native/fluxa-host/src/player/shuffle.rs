use super::*;

pub(crate) struct Shuffle {
    item: Value,
    episodes: Vec<String>,
    played: Vec<String>,
}

impl Shuffle {
    fn current(&self) -> Option<&str> {
        self.played.last().map(String::as_str)
    }

    fn pick(&mut self) -> Option<Value> {
        let seed = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or(0);
        let plan = core_value(
            "shuffleEpisodePick",
            json!({
                "episodeIds": self.episodes,
                "playedIds": self.played,
                "currentId": self.current(),
                "seed": seed,
            }),
        )?;
        let id = plan.get("videoId")?.as_str()?.to_owned();
        if plan.get("restarted").and_then(Value::as_bool) == Some(true) {
            self.played.clear();
        }
        self.played.push(id.clone());
        let mut item = self.item.clone();
        let fields = item.as_object_mut()?;
        fields.retain(|key, _| !key.starts_with("last") && key != "timeOffset");
        fields.insert("lastVideoId".to_owned(), id.into());
        Some(item)
    }
}

pub(crate) fn start_shuffle(state: &mut RendererState) -> Option<Value> {
    let mut shuffle = Shuffle {
        item: state.detail.item.clone(),
        episodes: state
            .detail
            .episodes
            .iter()
            .filter(|episode| episode.season > 0)
            .map(|episode| episode.id.clone())
            .collect(),
        played: Vec::new(),
    };
    let item = shuffle.pick();
    state.shuffle = item.is_some().then_some(shuffle);
    item
}

pub(crate) fn keep_shuffle_for(shuffle: &mut Option<Shuffle>, item: &Value) {
    let video = item.get("lastVideoId").and_then(Value::as_str);
    if shuffle.as_ref().and_then(Shuffle::current) != video {
        *shuffle = None;
    }
}

pub(super) fn advance_shuffle(state: &mut RendererState) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let status = &player.status;
    if state.shuffle.is_none()
        || !status.has_frame
        || status.duration <= 0.0
        || status.duration - status.position > 1.0
    {
        return;
    }
    let next = state.shuffle.as_mut().and_then(Shuffle::pick);
    close(state);
    match next {
        Some(item) => state
            .pending_native_actions
            .push(crate::NativeAction::StartPlayback { item }),
        None => state.shuffle = None,
    }
}
