use crate::ids::IdSet;
use crate::types::Meta;

#[derive(Clone, Debug)]
pub struct SearchHit {
    pub ids: IdSet,
    pub meta: Meta,
}

pub fn cache_query(query: &str) -> String {
    query.trim().to_lowercase()
}

pub fn merge_results(provider_results: Vec<Vec<SearchHit>>) -> Vec<Meta> {
    let mut accepted: Vec<SearchHit> = Vec::new();
    for hit in provider_results.into_iter().flatten() {
        let duplicate = accepted.iter().any(|a| {
            a.ids.shares_any(&hit.ids) || (!hit.meta.id.is_empty() && a.meta.id == hit.meta.id)
        });
        if !duplicate {
            accepted.push(hit);
        }
    }
    accepted.into_iter().map(|h| h.meta).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: &str, name: &str, extra: impl FnOnce(&mut IdSet)) -> SearchHit {
        let mut ids = IdSet::from_prefixed(id);
        extra(&mut ids);
        SearchHit {
            ids,
            meta: Meta {
                id: id.into(),
                name: name.into(),
                ..Default::default()
            },
        }
    }

    #[test]
    fn primary_provider_order_wins_on_duplicates() {
        let primary = vec![hit("tmdb:1", "Primary", |i| i.imdb = Some("tt1".into()))];
        let secondary = vec![hit("tt1", "Secondary", |_| {}), hit("tt2", "Other", |_| {})];
        let merged = merge_results(vec![primary, secondary]);
        let names: Vec<_> = merged.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["Primary", "Other"]);
    }

    #[test]
    fn empty_results_yield_an_empty_page() {
        assert!(merge_results(vec![Vec::new(), Vec::new()]).is_empty());
    }

    #[test]
    fn cache_query_trims_and_lowercases() {
        assert_eq!(cache_query("  Dune "), "dune");
    }
}
