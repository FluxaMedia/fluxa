mod genres_catalog;
mod helpers;
mod meta_conversion;
mod request_plans;

pub(crate) use genres_catalog::{tmdb_builtin_catalog_url, tmdb_builtin_manifest_json};
pub(crate) use helpers::{tmdb_content_type, tmdb_image_url, tmdb_language, tmdb_resolve_id_hint};
pub(crate) use meta_conversion::{
    merge_tmdb_enrichment_json, tmdb_bulk_metas_to_metas_json, tmdb_bulk_videos_to_trailers_json,
    tmdb_episodes_to_videos_json, tmdb_full_meta_to_meta_json, tmdb_meta_to_meta_json,
    tmdb_pick_logo_json, tmdb_video_to_trailer_json,
};
pub(crate) use request_plans::{
    tmdb_builtin_meta_request_plan_json, tmdb_builtin_meta_urls_from_find_json,
    tmdb_builtin_meta_urls_json, tmdb_credits_url_from_find, tmdb_detail_request_plan_json,
    tmdb_detail_request_urls_from_find_json, tmdb_people_images_from_credits,
    tmdb_people_request_plan, tmdb_season_request_url,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn builtin_manifest_declares_no_stream_resource() {
        let manifest: Value = serde_json::from_str(&tmdb_builtin_manifest_json()).unwrap();
        let resources: Vec<&str> = manifest["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(resources.contains(&"catalog"));
        assert!(resources.contains(&"meta"));
        assert!(!resources.contains(&"stream"));
    }

    #[test]
    fn full_meta_prefers_imdb_id_when_available() {
        let details =
            json!({"id": 550, "title": "Fight Club", "overview": "...", "vote_average": 8.4})
                .to_string();
        let credits = json!({"cast": [], "crew": []}).to_string();
        let images = json!({"logos": []}).to_string();
        let external_ids = json!({"imdb_id": "tt0137523"}).to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                "{}",
                "movie",
                "en",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["id"], "tt0137523");
        assert_eq!(result["type"], "movie");
    }

    #[test]
    fn full_meta_falls_back_to_tmdb_id_without_imdb_match() {
        let details = json!({"id": 550, "name": "Some Show"}).to_string();
        let credits = json!({}).to_string();
        let images = json!({}).to_string();
        let external_ids = json!({}).to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                "{}",
                "series",
                "en",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["id"], "tmdb:550");
        assert_eq!(result["type"], "series");
    }

    #[test]
    fn full_meta_picks_logo_matching_requested_language_first() {
        let details = json!({"id": 1, "title": "X"}).to_string();
        let credits = json!({}).to_string();
        let images = json!({"logos": [
            {"iso_639_1": "en", "file_path": "/en.png"},
            {"iso_639_1": "tr", "file_path": "/tr.png"},
        ]})
        .to_string();
        let external_ids = json!({}).to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                "{}",
                "movie",
                "tr",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["logo"], "https://image.tmdb.org/t/p/w500/tr.png");
    }

    #[test]
    fn episodes_map_still_path_to_thumbnail() {
        let season = json!({"episodes": [
            {"season_number": 1, "episode_number": 3, "name": "Ep 3", "still_path": "/s3.jpg", "air_date": "2020-01-01"},
        ]})
        .to_string();
        let result: Value =
            serde_json::from_str(&tmdb_episodes_to_videos_json(&season, "tt123").unwrap()).unwrap();
        let video = &result[0];
        assert_eq!(video["id"], "tt123:1:3");
        assert_eq!(video["thumbnail"], "https://image.tmdb.org/t/p/w300/s3.jpg");
    }

    #[test]
    fn catalog_url_maps_genre_name_to_id() {
        let url = tmdb_builtin_catalog_url("movie", &json!({"genre": "Horror"}), "KEY", "en");
        assert!(url.contains("3/discover/movie"));
        assert!(url.contains("with_genres=27"));
    }

    #[test]
    fn catalog_url_maps_skip_to_page() {
        let url = tmdb_builtin_catalog_url("movie", &json!({"skip": 40}), "KEY", "en");
        assert!(url.contains("page=3"));
    }

    #[test]
    fn tmdb_language_uses_shared_locale_fallbacks() {
        assert_eq!(tmdb_language(""), "en-US");
        assert_eq!(tmdb_language("english_us"), "en-US");
        assert_eq!(tmdb_language("tr_tr"), "tr-TR");
    }

    #[test]
    fn enrichment_skips_fields_whose_group_flag_is_off() {
        let base = json!({"logo": "addon-logo", "description": "addon-desc"}).to_string();
        let tmdb = json!({"logo": "tmdb-logo", "description": "tmdb-desc"}).to_string();
        let flags = json!({"artwork": true, "description": false}).to_string();
        let result: Value =
            serde_json::from_str(&merge_tmdb_enrichment_json(&base, &tmdb, &flags).unwrap())
                .unwrap();
        assert_eq!(result["logo"], "tmdb-logo");
        assert_eq!(result["description"], "addon-desc");
    }

    #[test]
    fn enrichment_group_flag_overrides_all_its_fields_at_once() {
        let base =
            json!({"logo": "addon-logo", "poster": "addon-poster", "background": "addon-bg"})
                .to_string();
        let tmdb = json!({"logo": "tmdb-logo", "poster": "tmdb-poster", "background": "tmdb-bg"})
            .to_string();
        let flags = json!({"artwork": true}).to_string();
        let result: Value =
            serde_json::from_str(&merge_tmdb_enrichment_json(&base, &tmdb, &flags).unwrap())
                .unwrap();
        assert_eq!(result["logo"], "tmdb-logo");
        assert_eq!(result["poster"], "tmdb-poster");
        assert_eq!(result["background"], "tmdb-bg");
    }

    #[test]
    fn enrichment_leaves_field_untouched_when_tmdb_value_is_null() {
        let base = json!({"network": "Addon Network"}).to_string();
        let tmdb = json!({"network": null}).to_string();
        let flags = json!({"network": true}).to_string();
        let result: Value =
            serde_json::from_str(&merge_tmdb_enrichment_json(&base, &tmdb, &flags).unwrap())
                .unwrap();
        assert_eq!(result["network"], "Addon Network");
    }

    #[test]
    fn keywords_use_movie_key_for_movies_and_results_key_for_series() {
        let details = json!({"id": 1, "title": "X"}).to_string();
        let credits = json!({}).to_string();
        let images = json!({}).to_string();
        let external_ids = json!({}).to_string();
        let extras = json!({"keywords": {"keywords": [{"name": "heist"}]}}).to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                &extras,
                "movie",
                "en",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["keywords"][0], "heist");

        let details = json!({"id": 2, "name": "Y", "first_air_date": "2020-01-01"}).to_string();
        let extras = json!({"keywords": {"results": [{"name": "anime"}]}}).to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                &extras,
                "series",
                "en",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["keywords"][0], "anime");
    }

    #[test]
    fn certification_reads_release_dates_for_movies_and_ratings_field_for_series() {
        let credits = json!({}).to_string();
        let images = json!({}).to_string();
        let external_ids = json!({}).to_string();

        let details = json!({"id": 1, "title": "X"}).to_string();
        let extras = json!({"contentRatings": {"results": [
            {"iso_3166_1": "US", "release_dates": [{"certification": ""}, {"certification": "R"}]},
        ]}})
        .to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                &extras,
                "movie",
                "en",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["certification"], "R");

        let details = json!({"id": 2, "name": "Y", "first_air_date": "2020-01-01"}).to_string();
        let extras = json!({"contentRatings": {"results": [
            {"iso_3166_1": "US", "rating": "TV-MA"},
        ]}})
        .to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                &extras,
                "series",
                "en",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["certification"], "TV-MA");
    }

    #[test]
    fn watch_providers_falls_back_to_us_when_region_has_no_data() {
        let details = json!({"id": 1, "title": "X"}).to_string();
        let credits = json!({}).to_string();
        let images = json!({}).to_string();
        let external_ids = json!({}).to_string();
        let extras = json!({"watchProviders": {"results": {
            "US": {"link": "https://tmdb.org/x", "flatrate": [{"provider_name": "Netflix", "logo_path": "/nf.jpg"}]},
        }}})
        .to_string();
        let result: Value = serde_json::from_str(
            &tmdb_full_meta_to_meta_json(
                &details,
                &credits,
                &images,
                &external_ids,
                &extras,
                "movie",
                "tr",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["watchProviders"]["region"], "US");
        assert_eq!(result["watchProviders"]["flatrate"][0]["name"], "Netflix");
        assert_eq!(
            result["watchProviders"]["flatrate"][0]["logo"],
            "https://image.tmdb.org/t/p/w92/nf.jpg"
        );
    }

    #[test]
    fn enrichment_overrides_episode_thumbnail_by_season_and_episode_only() {
        let base = json!({"videos": [
            {"id": "addon:1:1", "season": 1, "episode": 1, "title": "Ep 1", "thumbnail": "addon-thumb"},
            {"id": "addon:1:2", "season": 1, "episode": 2, "title": "Ep 2", "thumbnail": null},
        ]})
        .to_string();
        let tmdb = json!({"videos": [
            {"season": 1, "episode": 1, "thumbnail": "tmdb-thumb-1"},
        ]})
        .to_string();
        let flags = json!({"episodeStills": true}).to_string();
        let result: Value =
            serde_json::from_str(&merge_tmdb_enrichment_json(&base, &tmdb, &flags).unwrap())
                .unwrap();
        assert_eq!(result["videos"][0]["thumbnail"], "tmdb-thumb-1");
        assert_eq!(result["videos"][0]["title"], "Ep 1");
        assert_eq!(result["videos"][1]["thumbnail"], Value::Null);
    }
}
