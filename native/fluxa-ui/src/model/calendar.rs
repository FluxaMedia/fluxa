use super::*;

#[derive(Clone, Debug, Default)]
pub struct CalendarEntry {
    pub date: String,
    pub show: String,
    pub episode: String,
    pub still_url: Option<String>,
    pub card: HomeCard,
}

#[derive(Clone, Debug, Default)]
pub struct CalendarModel {
    pub language: String,
    pub year: i32,
    pub month: i32,
    pub is_loading: bool,
    pub error: Option<String>,
    pub entries: Vec<CalendarEntry>,
    pub selected_day: Option<u32>,
    pub today: Option<String>,
}

impl CalendarModel {
    pub fn entries_for_day(&self, day: u32) -> impl Iterator<Item = &CalendarEntry> {
        let date = format!("{:04}-{:02}-{:02}", self.year, self.month, day);
        self.entries.iter().filter(move |entry| entry.date == date)
    }
}

pub fn calendar_model_from_core_snapshot(snapshot: &serde_json::Value) -> CalendarModel {
    let calendar = snapshot.get("calendar").unwrap_or(&serde_json::Value::Null);
    let year = calendar
        .get("year")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0) as i32;
    let month = calendar
        .get("month")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0) as i32;
    let mut raw_items = Vec::new();
    for key in ["items", "localItems", "externalItems"] {
        if let Some(items) = calendar.get(key).and_then(serde_json::Value::as_array) {
            raw_items.extend(items.iter().cloned());
        }
    }
    CalendarModel {
        language: snapshot_language(snapshot),
        year,
        month,
        selected_day: None,
        today: Some(today_iso()),
        is_loading: calendar
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        error: calendar
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        entries: raw_items
            .into_iter()
            .take(256)
            .filter_map(|item| {
                let date = first_value_string(
                    &item,
                    &["dateIso", "airDate", "released", "releaseDate", "date"],
                )?
                .chars()
                .take(10)
                .collect();
                let card = core_home_card(&item);
                let show = item
                    .get("meta")
                    .and_then(|meta| first_value_string(meta, &["name", "title"]))
                    .or_else(|| value_string(&item, "title"))
                    .unwrap_or_else(|| card.title.clone());
                let episode = first_value_string(&item, &["subtitle", "episodeTitle"])
                    .filter(|episode| *episode != show)
                    .unwrap_or_default();
                let still_url = first_value_string(
                    &item,
                    &["episodePoster", "background", "backdrop", "poster"],
                )
                .or_else(|| {
                    item.get("meta")
                        .and_then(|meta| first_value_string(meta, &["background", "poster"]))
                })
                .or_else(|| card.artwork_url.clone());
                Some(CalendarEntry {
                    date,
                    show,
                    episode,
                    still_url,
                    card,
                })
            })
            .collect(),
    }
}
