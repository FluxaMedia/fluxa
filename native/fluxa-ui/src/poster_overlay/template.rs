use super::*;

pub(crate) fn custom_url(context: &egui::Context, card: &HomeCard) -> Option<String> {
    if card.row_kind == HomeRowKind::Collection {
        return None;
    }
    let overlays = current(context)?;
    let template = overlays.template.as_deref()?;
    let id = card.id.as_deref()?;
    let shape = if landscape() { "landscape" } else { "poster" };
    URLS.with_borrow_mut(|(cached, urls)| {
        if cached != template {
            *cached = template.to_owned();
            urls.clear();
        }
        urls.entry(format!("{shape}:{id}"))
            .or_insert_with(|| {
                fill_template(
                    template,
                    id,
                    card.item_type.as_deref().unwrap_or_default(),
                    shape,
                    &card.raw,
                )
            })
            .clone()
    })
}

pub(super) fn fill_template(
    template: &str,
    id: &str,
    item_type: &str,
    shape: &str,
    raw: &serde_json::Value,
) -> Option<String> {
    let field = |keys: &[&str]| {
        keys.iter().find_map(|key| {
            let value = raw.get(*key)?;
            value
                .as_str()
                .map(str::to_owned)
                .or_else(|| value.as_i64().map(|number| number.to_string()))
                .filter(|value| !value.is_empty())
        })
    };
    let prefixed = |prefix: &str| {
        id.strip_prefix(prefix)
            .map(|rest| rest.split(':').next().unwrap_or(rest).to_owned())
    };
    let lookup = |name: &str| match name {
        "imdb_id" => field(&["imdb_id", "imdbId"]).or_else(|| {
            id.starts_with("tt")
                .then(|| id.split(':').next().unwrap_or(id).to_owned())
        }),
        "tmdb_id" => field(&["tmdb_id", "tmdbId", "moviedb_id"]).or_else(|| prefixed("tmdb:")),
        "anilist_id" => prefixed("anilist:"),
        "kitsu_id" => prefixed("kitsu:"),
        "type" => Some(match item_type {
            "series" | "tv" | "show" => "series".to_owned(),
            _ => "movie".to_owned(),
        }),
        "id" => Some(id.to_owned()),
        "shape" => Some(shape.to_owned()),
        _ => None,
    };
    let mut url = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        url.push_str(&rest[..start]);
        let end = rest[start..].find('}')? + start;
        let name = &rest[start + 1..end];
        let (name, optional) = name
            .strip_suffix('?')
            .map_or((name, false), |name| (name, true));
        match lookup(name) {
            Some(value) => url.push_str(&value),
            None if optional => {}
            None => return None,
        }
        rest = &rest[end + 1..];
    }
    url.push_str(rest);
    Some(url)
}
