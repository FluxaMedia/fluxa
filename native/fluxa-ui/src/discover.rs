use super::*;
use crate::dp::Dp;
use crate::gfx::Ui;

fn all_extra_label(name: &str, language: &str) -> String {
    if name.eq_ignore_ascii_case("genre") {
        return localized("search.all_genres", language);
    }
    let readable_name = name
        .split(['_', '-', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ");
    localized("discover.all_filter_values", language).replacen("%s", &readable_name, 1)
}

pub(crate) struct FilterOptions {
    pub types: Vec<(String, String)>,
    pub catalogs: Vec<(String, String)>,
    pub catalog_key: String,
    pub extras: Vec<(String, String)>,
    pub has_extra: bool,
    pub extra_required: bool,
}

pub(crate) fn filter_options(discover: &DiscoverModel) -> FilterOptions {
    let catalogs = discover
        .catalogs
        .iter()
        .filter(|catalog| catalog.content_type == discover.content_type)
        .collect::<Vec<_>>();
    let selected_catalog = catalogs
        .iter()
        .find(|catalog| catalog.key == discover.selected_catalog_key)
        .copied()
        .or_else(|| catalogs.first().copied());
    let catalog_options = catalogs
        .iter()
        .map(|catalog| (catalog.key.clone(), catalog.label.clone()))
        .collect::<Vec<_>>();
    let mut content_types = if discover.content_types.is_empty() {
        discover
            .catalogs
            .iter()
            .map(|catalog| catalog.content_type.as_str())
            .collect::<Vec<_>>()
    } else {
        discover
            .content_types
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    };
    content_types.sort_unstable();
    content_types.dedup();
    if content_types.is_empty() {
        content_types.extend(["movie", "series"]);
    }
    if !content_types.contains(&discover.content_type.as_str()) {
        content_types.push(discover.content_type.as_str());
    }
    let types = content_types
        .iter()
        .map(|content_type| {
            let label = match *content_type {
                "movie" => localized("auto.movies", &discover.language),
                "series" => localized("auto.series", &discover.language),
                other => {
                    let mut label = other.to_owned();
                    if let Some(first) = label.get_mut(0..1) {
                        first.make_ascii_uppercase();
                    }
                    label
                }
            };
            ((*content_type).to_owned(), label)
        })
        .collect::<Vec<_>>();
    let selected_extra = selected_catalog.and_then(|catalog| {
        catalog
            .extras
            .iter()
            .find(|(name, _)| name == &discover.selected_extra_name)
            .or_else(|| catalog.extras.first())
    });
    let extra_required = selected_extra.is_some_and(|(name, _)| {
        selected_catalog.is_some_and(|catalog| catalog.required_extras.contains(name))
    });
    let extras = selected_extra
        .map(|(name, values)| {
            let mut options = Vec::with_capacity(values.len() + usize::from(!extra_required));
            if !extra_required {
                options.push((String::new(), all_extra_label(name, &discover.language)));
            }
            options.extend(values.iter().cloned().map(|value| (value.clone(), value)));
            options
        })
        .unwrap_or_default();
    FilterOptions {
        types,
        catalogs: catalog_options,
        catalog_key: selected_catalog
            .map(|catalog| catalog.key.clone())
            .unwrap_or_default(),
        extras,
        has_extra: selected_extra.is_some(),
        extra_required,
    }
}
