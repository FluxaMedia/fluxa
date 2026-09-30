use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Personal {
    pub watched: bool,
    pub saved: bool,
    pub progress: f32,
    pub quality: [bool; 4],
}

pub(super) const QUALITY_LABELS: [&str; 4] = ["4K", "DV", "HDR", "REMUX"];

pub(super) fn stream_quality(text: &str) -> [bool; 4] {
    let text = text.to_lowercase();
    let tokens: Vec<&str> = text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();
    let has = |words: &[&str]| tokens.iter().any(|token| words.contains(token));
    let dv = has(&["dv", "dovi", "dvhe"]) || text.contains("dolby vision");
    [
        has(&["2160p", "4k", "uhd"]),
        dv,
        !dv && (has(&["hdr", "hdr10", "hlg"]) || text.contains("hdr10+")),
        has(&["remux"]),
    ]
}

#[derive(Debug, Default, PartialEq)]
pub struct PersonalIndex(pub(super) HashMap<String, Personal>);

impl PersonalIndex {
    pub fn get(&self, id: &str) -> Option<Personal> {
        self.0.get(id).copied()
    }
}

pub(crate) fn personal_index(library: &serde_json::Value) -> PersonalIndex {
    let mut index = HashMap::<String, Personal>::new();
    let ids = |key: &str| {
        library
            .get(key)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| item.get("id").and_then(serde_json::Value::as_str))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let watched = library
        .get("watched")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, value)| value.as_bool() == Some(true))
        .map(|(id, _)| id.clone());
    for id in ids("completed").into_iter().chain(watched) {
        index.entry(id).or_default().watched = true;
    }
    for id in ids("watchlist").into_iter().chain(ids("liked")) {
        index.entry(id).or_default().saved = true;
    }
    if let Some(progress) = library
        .get("progress")
        .and_then(serde_json::Value::as_object)
    {
        for (key, entry) in progress {
            let duration = entry.get("duration").and_then(number).unwrap_or(0.0);
            if duration <= 0.0 {
                continue;
            }
            let id = entry
                .pointer("/meta/id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(key);
            let personal = index.entry(id.to_owned()).or_default();
            let stream = ["lastStreamTitle", "lastStreamUrl"]
                .iter()
                .filter_map(|key| entry.get(*key).and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join(" ");
            personal.quality = stream_quality(&stream);
        }
    }
    for item in library
        .get("continueWatching")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(id) = item.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let percent = item
            .get("resumeProgressPercent")
            .and_then(number)
            .map(|percent| percent / 100.0)
            .or_else(|| {
                let offset = item.get("timeOffset").and_then(number)?;
                let duration = item.get("duration").and_then(number)?;
                (duration > 0.0).then(|| offset / duration)
            });
        if let Some(percent) = percent {
            index.entry(id.to_owned()).or_default().progress = percent.clamp(0.0, 1.0) as f32;
        }
    }
    PersonalIndex(index)
}

pub fn set_poster_personal(context: &egui::Context, index: Arc<PersonalIndex>) {
    context.data_mut(|data| {
        let current = data.get_temp::<Arc<PersonalIndex>>(personal_id());
        if !current.is_some_and(|current| Arc::ptr_eq(&current, &index)) {
            data.insert_temp(personal_id(), index);
        }
    });
}

pub(crate) fn personal_for(context: &egui::Context, id: &str) -> Option<Personal> {
    context
        .data(|data| data.get_temp::<Arc<PersonalIndex>>(personal_id()))?
        .get(id)
}

pub(super) fn personal_id() -> Id {
    Id::new("fluxa-poster-personal")
}

pub(super) fn current(context: &egui::Context) -> Option<Arc<PosterOverlays>> {
    context.data(|data| data.get_temp::<Arc<PosterOverlays>>(overlays_id()))
}

thread_local! {
    pub(super) static URLS: RefCell<(String, HashMap<String, Option<String>>)> = RefCell::default();
}
