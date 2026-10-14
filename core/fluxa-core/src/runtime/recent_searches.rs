pub const LIMIT: usize = 10;
const MIN_CHARS: usize = 2;

pub fn record(history: &[String], query: &str) -> Option<Vec<String>> {
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if query.chars().count() < MIN_CHARS {
        return None;
    }
    if history.first().is_some_and(|latest| *latest == query) {
        return None;
    }
    let mut next = Vec::with_capacity(LIMIT);
    next.push(query.clone());
    let lowered = query.to_lowercase();
    next.extend(
        history
            .iter()
            .filter(|entry| entry.to_lowercase() != lowered)
            .cloned(),
    );
    next.truncate(LIMIT);
    Some(next)
}

pub fn matching<'a>(history: &'a [String], typed: &str) -> Vec<&'a str> {
    let typed = typed.trim().to_lowercase();
    history
        .iter()
        .map(String::as_str)
        .filter(|entry| {
            let entry = entry.to_lowercase();
            entry != typed && entry.contains(&typed)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn repeat_moves_to_front_ignoring_case() {
        let next = record(&list(&["dune", "alien"]), "  ALIEN ").unwrap();
        assert_eq!(next, list(&["ALIEN", "dune"]));
    }

    #[test]
    fn single_character_is_not_kept() {
        assert!(record(&list(&["dune"]), "a").is_none());
    }

    #[test]
    fn oldest_entry_drops_at_the_limit() {
        let history: Vec<String> = (0..LIMIT).map(|n| format!("query {n}")).collect();
        let next = record(&history, "fresh").unwrap();
        assert_eq!(next.len(), LIMIT);
        assert_eq!(next[0], "fresh");
        assert!(!next.contains(&format!("query {}", LIMIT - 1)));
    }

    #[test]
    fn typing_filters_and_hides_exact_match() {
        let history = list(&["dune", "dune part two", "alien"]);
        assert_eq!(matching(&history, "dune"), vec!["dune part two"]);
        assert_eq!(
            matching(&history, ""),
            vec!["dune", "dune part two", "alien"]
        );
    }
}
