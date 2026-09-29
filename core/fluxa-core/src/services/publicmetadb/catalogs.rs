use super::helpers::{build_url, extract_query, parse_args};

pub(crate) fn publicmetadb_catalogs_url() -> String {
    build_url("/catalogs", &[])
}

pub(crate) fn publicmetadb_catalog_items_url(id: &str, query_json: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    let params = extract_query(&parse_args(query_json), &["page"]);
    Some(build_url(&format!("/catalogs/{id}/items"), &params))
}
