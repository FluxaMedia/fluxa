# Bugs found by the scenario harness

Found while writing `native/fluxa-effects/tests/scenarios`. Each one has an `#[ignore = "..."]` test that documents it; remove the ignore once fixed and the test should pass.

Run with `npm run check:scenarios`.

## Effects with no native implementation

These fall through to `effect <name> has no native implementation yet` in `native/fluxa-effects/src/executor.rs`.

| Effect | Symptom | Ignored test |
| --- | --- | --- |
| `clearPlaybackProgress` | Clearing progress does nothing; the continue watching entry stays | `app::clearing_progress_removes_the_continue_watching_entry` |
| `fetchSeasonEpisodes` | `detailSeasonRequested` sets `/detail/error` | `playback::detail_season_selects_that_seasons_episodes` |
| `fetchDetailSecondary` | TMDB similar titles and trailers never load on detail | `tmdb::detail_secondary_loads_tmdb_similar_titles_and_trailers` |
| `fetchPluginManifest` | Plugin repositories can't be added | `gaps::plugin_repository_can_be_added_from_a_manifest_url` |
| `enqueueOfflineDownload` | Offline downloads never queue | `gaps::offline_download_is_queued_for_the_chosen_stream` |

Also unimplemented, not covered by a test: `updateCalendarWidget`, `notifyReleasedEpisodes`.

## Removing the last addon keeps its rows on home

An empty home reload falls back to the cached `home_bootstrap_v1_*`, so rows from the last removed addon stay visible.

Test: `app::removing_the_last_addon_empties_home`.

## Unverified

- Nuvio token refresh only saves the new tokens if the profile already exists in the `profiles` storage key (`store_nuvio_tokens` in `executor/account.rs`). The harness profile isn't stored, so saving is not asserted.
- `profileActivated` was not persisted across a restart in the harness. Unclear whether the shell is meant to persist it.
- `trailerResolveRequested` is not covered: the mock TLS certificate has no YouTube host.
