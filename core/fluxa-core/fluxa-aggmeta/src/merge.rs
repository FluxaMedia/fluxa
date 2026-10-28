use crate::types::Meta;

fn blank(s: &str) -> bool {
    s.trim().is_empty()
}

pub fn fill_missing(primary: &mut Meta, secondary: &Meta) {
    if blank(&primary.name) {
        primary.name = secondary.name.clone();
    }
    if blank(&primary.description) {
        primary.description = secondary.description.clone();
    }
    if primary.genres.is_empty() {
        primary.genres = secondary.genres.clone();
    }
    if primary.director.is_empty() {
        primary.director = secondary.director.clone();
    }
    if primary.writer.is_empty() {
        primary.writer = secondary.writer.clone();
    }
    if primary.app_extras.cast.is_empty() {
        primary.app_extras.cast = secondary.app_extras.cast.clone();
    }
    if blank(&primary.runtime) {
        primary.runtime = secondary.runtime.clone();
    }
    if blank(&primary.year) {
        primary.year = secondary.year.clone();
        primary.release_info = secondary.release_info.clone();
    }
    if blank(&primary.country) {
        primary.country = secondary.country.clone();
    }
    if primary.released.is_none() {
        primary.released = secondary.released.clone();
    }
    if primary.trailers.is_empty() {
        primary.trailers = secondary.trailers.clone();
    }
    if primary.keywords.is_empty() {
        primary.keywords = secondary.keywords.clone();
    }
    if primary.imdb_id.is_none() {
        primary.imdb_id = secondary.imdb_id.clone();
    }
    if primary.imdb_rating == "N/A" || blank(&primary.imdb_rating) {
        primary.imdb_rating = secondary.imdb_rating.clone();
    }
}

pub fn overlay_art(
    meta: &mut Meta,
    poster: Option<String>,
    background: Option<String>,
    logo: Option<String>,
    landscape: Option<String>,
) {
    if poster.is_some() {
        meta.poster = poster;
    }
    if background.is_some() {
        meta.background = background;
    }
    if logo.is_some() {
        meta.logo = logo;
    }
    if landscape.is_some() {
        meta.landscape_poster = landscape;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_text_falls_through_to_secondary() {
        let mut a = Meta {
            description: "   ".into(),
            name: "Dune".into(),
            ..Default::default()
        };
        let b = Meta {
            description: "Sand.".into(),
            name: "Other".into(),
            ..Default::default()
        };
        fill_missing(&mut a, &b);
        assert_eq!(a.description, "Sand.");
        assert_eq!(a.name, "Dune");
    }

    #[test]
    fn lists_are_never_concatenated() {
        let mut a = Meta {
            genres: vec!["Drama".into()],
            ..Default::default()
        };
        let b = Meta {
            genres: vec!["Comedy".into()],
            ..Default::default()
        };
        fill_missing(&mut a, &b);
        assert_eq!(a.genres, vec!["Drama"]);
    }
}
