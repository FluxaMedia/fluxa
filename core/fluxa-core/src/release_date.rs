pub(crate) fn is_upcoming(released: &str, today_iso: &str) -> bool {
    let released = released.trim();
    let today = today_iso.trim();
    if released.is_empty() || today.len() < 4 {
        return false;
    }

    let date = released.get(..10).unwrap_or("");
    if date.len() == 10
        && date.as_bytes().get(4) == Some(&b'-')
        && date.as_bytes().get(7) == Some(&b'-')
    {
        return date > today.get(..10).unwrap_or(today);
    }

    let year = released.get(..4).unwrap_or("");
    year.len() == 4
        && year.chars().all(|value| value.is_ascii_digit())
        && year > today.get(..4).unwrap_or(today)
}

pub(crate) fn is_released(released: &str, today_iso: &str) -> bool {
    let Some(released_date) = parse_date(released) else {
        return false;
    };
    let Some(today_date) = parse_date(today_iso) else {
        return false;
    };
    released_date <= today_date
}

pub(crate) fn is_recently_released(released: &str, today_iso: &str, window_days: i64) -> bool {
    if window_days < 0 {
        return false;
    }
    let Some(released) = parse_date(released) else {
        return false;
    };
    let Some(today) = parse_date(today_iso) else {
        return false;
    };
    let days_since = today - released;
    (0..=window_days).contains(&days_since)
}

fn parse_date(value: &str) -> Option<i64> {
    let date = value.trim().get(..10)?;
    let year = date.get(..4)?.parse::<i64>().ok()?;
    let month = date.get(5..7)?.parse::<i64>().ok()?;
    let day = date.get(8..10)?.parse::<i64>().ok()?;
    if date.as_bytes().get(4) != Some(&b'-')
        || date.as_bytes().get(7) != Some(&b'-')
        || !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
    {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let adjusted_year = year - i64::from(month <= 2);
    let era = (if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    }) / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_offset = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_offset + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146097 + day_of_era
}

#[cfg(test)]
mod tests {
    use super::{is_recently_released, is_released, is_upcoming};

    #[test]
    fn released_date_is_inclusive_and_rejects_invalid_dates() {
        assert!(is_released("2026-09-07T08:00:00Z", "2026-09-08"));
        assert!(is_released("2026-09-08", "2026-09-08"));
        assert!(!is_released("2026-09-09", "2026-09-08"));
        assert!(!is_released("unknown", "2026-09-08"));
    }

    #[test]
    fn compares_full_release_dates_against_the_supplied_local_day() {
        assert!(is_upcoming("2026-09-09T00:00:00Z", "2026-09-08"));
        assert!(!is_upcoming("2026-09-08", "2026-09-08"));
        assert!(!is_upcoming("2026-09-07T00:00:00Z", "2026-09-08"));
    }

    #[test]
    fn supports_year_only_release_labels() {
        assert!(is_upcoming("2027", "2026-09-08"));
        assert!(!is_upcoming("2026", "2026-09-08"));
        assert!(!is_upcoming("unknown", "2026-09-08"));
    }

    #[test]
    fn recently_released_uses_the_supplied_local_day_and_window() {
        assert!(is_recently_released(
            "2026-09-01T00:00:00Z",
            "2026-09-08",
            7
        ));
        assert!(!is_recently_released("2026-08-31", "2026-09-08", 7));
        assert!(!is_recently_released("2026-09-09", "2026-09-08", 7));
    }

    #[test]
    fn recently_released_handles_month_and_leap_year_boundaries() {
        assert!(is_recently_released("2024-02-28", "2024-03-01", 2));
        assert!(!is_recently_released("2023-02-28", "2023-03-01", 0));
    }
}
