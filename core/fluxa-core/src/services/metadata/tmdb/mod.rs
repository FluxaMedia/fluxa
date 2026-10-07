mod routes;
pub(crate) use routes::*;
mod genres_catalog;
mod helpers;
mod meta_conversion;
mod request_plans;

pub(crate) use genres_catalog::{tmdb_builtin_catalog_url, tmdb_builtin_manifest_json};
pub(crate) use meta_conversion::{
    tmdb_bulk_metas_to_metas_json, tmdb_bulk_videos_to_trailers_json,
};
pub(crate) use request_plans::{
    tmdb_detail_request_plan_json, tmdb_detail_request_urls_from_find_json,
};

#[cfg(test)]
mod tests;
