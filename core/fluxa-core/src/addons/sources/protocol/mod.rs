mod assets;
mod catalogs;
mod manifest_parse;
mod meta_links;
mod url;

pub(crate) use assets::{merge_live_manifest_json, resolve_manifest_assets_json};
pub(crate) use catalogs::{
    catalog_has_required_extra_except, catalog_requires_extra, catalog_search_eligible,
    catalog_supports_extra, supports_resource,
};
pub(crate) use manifest_parse::normalize_addon_descriptor_json;
pub use manifest_parse::parse_manifest;
pub(crate) use meta_links::classify_meta_links_json;
pub(crate) use url::{
    base_url, build_resource_url, identity, is_http_url, manifest_candidates,
    manifest_fetch_plan_json, normalize_manifest_url, prefer_https_asset_url,
};

#[cfg(test)]
mod tests;
