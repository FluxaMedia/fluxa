use super::*;

pub fn calendar_data(calendar: &CalendarModel) -> serde_json::Value {
    let language = calendar.language.as_str();
    let valid = calendar.year > 0 && (1..=12).contains(&calendar.month);
    let title = if valid {
        format!(
            "{} {}",
            localized(
                &format!(
                    "calendar.month_{}",
                    month_name(calendar.month).to_lowercase()
                ),
                language
            ),
            calendar.year
        )
    } else {
        String::new()
    };
    let weekdays: Vec<_> = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"]
        .into_iter()
        .map(|day| {
            serde_json::json!({
                "label": localized(&format!("calendar.weekday.{day}"), language).to_uppercase()
            })
        })
        .collect();
    let mut cells = Vec::new();
    if valid {
        let leading = weekday_sunday_zero(calendar.year, calendar.month, 1) as usize;
        cells.extend(
            (0..leading)
                .map(|_| serde_json::json!({"blank": true, "class": "day-blank", "day": 0})),
        );
        for day in 1..=days_in_month(calendar.year, calendar.month) as u32 {
            let entries: Vec<_> = calendar.entries_for_day(day).collect();
            let date = format!("{:04}-{:02}-{:02}", calendar.year, calendar.month, day);
            cells.push(serde_json::json!({
                "blank": false,
                "class": "day surface soft",
                "day": day,
                "count": entries.len(),
                "today": calendar.today.as_deref() == Some(date.as_str()),
                "numClass": if calendar.today.as_deref() == Some(date.as_str()) { "num-today" } else { "num" },
                "numTextClass": match calendar.today.as_deref() {
                    Some(today) if today == date.as_str() => "num-label-today",
                    Some(today) if date.as_str() < today => "num-label-past",
                    _ => "num-label",
                },
                "selected": calendar.selected_day == Some(day),
                "still": entries.first().and_then(|entry| entry.still_url.clone()),
                "show": entries.first().map(|entry| entry.show.clone()).unwrap_or_default(),
            }));
        }
    }
    let selected: Vec<_> = calendar
        .selected_day
        .map(|day| {
            calendar
                .entries_for_day(day)
                .map(|entry| {
                    let mut card = serde_json::to_value(&entry.card).unwrap_or_default();
                    card["title"] = entry.show.clone().into();
                    card["subtitle"] = entry.episode.clone().into();
                    if let Some(still) = &entry.still_url {
                        card["artworkUrl"] = still.clone().into();
                    }
                    card
                })
                .collect()
        })
        .unwrap_or_default();
    serde_json::json!({
        "valid": valid,
        "title": title,
        "loading": calendar.is_loading,
        "error": calendar.error,
        "weekdays": weekdays,
        "cells": cells,
        "selectedDay": calendar.selected_day,
        "selected": selected,
    })
}

fn month_name(month: i32) -> &'static str {
    [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ]
    .get(month.saturating_sub(1) as usize)
    .copied()
    .unwrap_or("Month")
}

pub(super) fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub(super) fn weekday_sunday_zero(year: i32, month: i32, day: i32) -> i32 {
    let (mut year, mut month) = (year, month);
    if month < 3 {
        year -= 1;
        month += 12;
    }
    (day + (13 * (month + 1)) / 5 + year + year / 4 - year / 100 + year / 400 + 6).rem_euclid(7)
}
