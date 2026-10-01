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
};
pub(crate) use episode_ratings::{
    publicmetadb_episode_ratings_batch_create_plan, publicmetadb_episode_ratings_batch_delete_plan,
};
pub(crate) use helpers::publicmetadb_bearer;
pub(crate) use skips::{
    publicmetadb_skips_create_plan, publicmetadb_skips_url,
};

#[cfg(test)]
mod tests;
