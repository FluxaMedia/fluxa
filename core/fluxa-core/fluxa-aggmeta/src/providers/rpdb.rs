const API_BASE: &str = "https://api.ratingposterdb.com";

pub fn poster_url(key: &str, imdb: &str) -> Option<String> {
    let key = key.trim();
    if key.is_empty() || imdb.is_empty() {
        return None;
    }
    Some(format!(
        "{API_BASE}/{key}/imdb/poster-default/{imdb}.jpg?fallback=true"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_both_a_key_and_an_imdb_id() {
        assert!(poster_url("", "tt1").is_none());
        assert!(poster_url("k", "").is_none());
        assert!(
            poster_url("k", "tt1")
                .unwrap()
                .contains("/k/imdb/poster-default/tt1.jpg")
        );
    }
}
