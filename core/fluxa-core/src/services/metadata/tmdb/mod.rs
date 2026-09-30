mod routes;
pub(crate) use routes::*;
mod genres_catalog;
mod helpers;
mod meta_conversion;
mod request_plans;

pub(crate) use genres_catalog::{tmdb_builtin_catalog_url, tmdb_builtin_manifest_json};
pub(crate) use helpers::{tmdb_content_type, tmdb_image_url, tmdb_language, tmdb_resolve_id_hint};
pub(crate) use meta_conversion::{
    merge_tmdb_enrichment_json, tmdb_bulk_metas_to_metas_json, tmdb_bulk_videos_to_trailers_json,
    tmdb_episodes_to_videos_json, tmdb_full_meta_to_meta_json, tmdb_item_content_type,
    tmdb_meta_to_meta_json, tmdb_pick_logo_json, tmdb_video_to_trailer_json,
};
pub(crate) use request_plans::{
    tmdb_builtin_meta_request_plan_json, tmdb_builtin_meta_urls_from_find_json,
    tmdb_builtin_meta_urls_json, tmdb_collection_source_url_json, tmdb_credits_url_from_find,
    tmdb_detail_request_plan_json, tmdb_detail_request_urls_from_find_json,
    tmdb_people_images_from_credits, tmdb_people_request_plan, tmdb_recommendations_url_json,
    tmdb_season_request_url,
};

#[cfg(test)]
mod tests;
