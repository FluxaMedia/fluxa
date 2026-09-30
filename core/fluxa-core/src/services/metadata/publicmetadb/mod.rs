mod routes;
pub(crate) use routes::*;
mod anime_seasons;
mod catalogs;
mod episode_ratings;
mod helpers;
mod highlights;
mod lists;
mod mappings;
mod ratings;
mod resume;
mod skips;
mod votes;
mod watched;

pub(crate) use anime_seasons::{
    publicmetadb_anime_seasons_delete_chunk_plan, publicmetadb_anime_seasons_delete_mapping_plan,
    publicmetadb_anime_seasons_submit_plan, publicmetadb_anime_seasons_url,
};
pub(crate) use catalogs::{publicmetadb_catalog_items_url, publicmetadb_catalogs_url};
pub(crate) use episode_ratings::{
    publicmetadb_episode_ratings_batch_create_plan, publicmetadb_episode_ratings_batch_delete_plan,
    publicmetadb_episode_ratings_batch_url, publicmetadb_episode_ratings_create_plan,
    publicmetadb_episode_ratings_delete_plan, publicmetadb_episode_ratings_url,
};
pub(crate) use helpers::publicmetadb_bearer;
pub(crate) use highlights::{
    publicmetadb_highlights_create_plan, publicmetadb_highlights_delete_plan,
    publicmetadb_highlights_url,
};
pub(crate) use lists::{
    publicmetadb_list_items_add_plan, publicmetadb_list_items_remove_plan,
    publicmetadb_list_items_url, publicmetadb_lists_create_plan, publicmetadb_lists_delete_plan,
    publicmetadb_lists_url,
};
pub(crate) use mappings::{
    publicmetadb_mappings_create_plan, publicmetadb_mappings_delete_plan,
    publicmetadb_mappings_lookup_url, publicmetadb_mappings_url,
};
pub(crate) use ratings::{
    publicmetadb_ratings_create_plan, publicmetadb_ratings_delete_plan, publicmetadb_ratings_url,
};
pub(crate) use resume::{
    publicmetadb_resume_batch_plan, publicmetadb_resume_delete_plan, publicmetadb_resume_save_plan,
    publicmetadb_resume_url,
};
pub(crate) use skips::{
    publicmetadb_skips_create_plan, publicmetadb_skips_delete_plan, publicmetadb_skips_url,
};
pub(crate) use votes::{
    publicmetadb_votes_create_plan, publicmetadb_votes_delete_plan, publicmetadb_votes_url,
};
pub(crate) use watched::{
    publicmetadb_watched_bulk_delete_plan, publicmetadb_watched_delete_plan,
    publicmetadb_watched_edit_date_plan, publicmetadb_watched_mark_plan, publicmetadb_watched_url,
};

#[cfg(test)]
mod tests;
