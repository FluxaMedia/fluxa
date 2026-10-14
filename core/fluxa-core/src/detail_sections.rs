use serde_json::Value;

pub const ORDER_KEY: &str = "detailSectionOrder";
pub const SECTIONS: [&str; 5] = [
    "cast",
    "episodes",
    "trailers",
    "recommendations",
    "show_info",
];

pub fn toggle_key(section: &str) -> Option<&'static str> {
    match section {
        "episodes" => Some("detailShowEpisodes"),
        "trailers" => Some("detailShowTrailers"),
        "show_info" => Some("detailShowInfo"),
        "cast" => Some("detailShowCast"),
        "recommendations" => Some("detailShowRecommendations"),
        _ => None,
    }
}

pub fn order(values: &Value) -> Vec<&'static str> {
    let mut sections = Vec::with_capacity(SECTIONS.len());
    let saved = values
        .get(ORDER_KEY)
        .and_then(Value::as_str)
        .unwrap_or_default();
    for name in saved.split(',').map(str::trim) {
        if let Some(known) = SECTIONS.iter().find(|known| **known == name)
            && !sections.contains(known)
        {
            sections.push(*known);
        }
    }
    for known in SECTIONS {
        if !sections.contains(&known) {
            sections.push(known);
        }
    }
    sections
}

pub fn enabled(values: &Value, section: &str) -> bool {
    toggle_key(section)
        .and_then(|key| values.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

pub fn rows(values: &Value) -> Vec<(&'static str, bool)> {
    order(values)
        .into_iter()
        .map(|section| (section, enabled(values, section)))
        .collect()
}

pub fn visible(values: &Value) -> Vec<&'static str> {
    rows(values)
        .into_iter()
        .filter_map(|(section, on)| on.then_some(section))
        .collect()
}

pub fn moved(values: &Value, section: &str, delta: isize) -> Option<String> {
    let mut sections = order(values);
    let from = sections.iter().position(|known| *known == section)?;
    let to = from
        .checked_add_signed(delta)
        .filter(|to| *to < sections.len())?;
    sections.swap(from, to);
    Some(sections.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_setting_keeps_default_order_with_everything_on() {
        assert_eq!(visible(&json!({})), SECTIONS);
    }

    #[test]
    fn unknown_and_repeated_names_are_dropped_and_gaps_appended() {
        let values = json!({ORDER_KEY: "cast,bogus,cast,episodes"});
        assert_eq!(
            order(&values),
            [
                "cast",
                "episodes",
                "trailers",
                "recommendations",
                "show_info"
            ]
        );
    }

    #[test]
    fn disabled_sections_leave_the_visible_list_but_keep_their_slot() {
        let values = json!({"detailShowTrailers": false});
        assert_eq!(
            visible(&values),
            ["cast", "episodes", "recommendations", "show_info"]
        );
        assert_eq!(order(&values)[2], "trailers");
    }

    #[test]
    fn moving_past_either_end_is_refused() {
        let values = json!({});
        assert_eq!(moved(&values, "cast", -1), None);
        assert_eq!(moved(&values, "show_info", 1), None);
        assert_eq!(
            moved(&values, "trailers", -1).as_deref(),
            Some("cast,trailers,episodes,recommendations,show_info")
        );
    }
}
