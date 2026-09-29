# Wiring map

What each domain does today and what is still missing to reach it through the `Engine`. The renderer rewrite left much of the backend without a caller in the shells, so code listed as unwired below is waiting to be connected, not dead.

Two ways in exist right now:

- `Engine` (`dispatch(action) -> effects`, `complete(result)`). This is where everything is heading.
- `ffi::core_invoke` / `ffi::call`, a string-routed shim with 616 routes. `native/fluxa-effects` reaches about 87 of them through `core_value`. Each route moves to the engine when its domain is wired, then the route is removed.

"Routes called" counts routes whose name appears in `native/` against the routes of that family.

## Domains

| Domain | Modules | Reached today | Routes called |
|---|---|---|---|
| `home` | `ranking`, `recommendation` | router only | ranking through `core_value`; `home_hero_plan` is re-exported at the crate root |
| `library` | `state`, `continue_watching`, `watchlist`, `provider`, `calendar`, `persistence`, `offline_download`, `local_media`, `release_date` | `state` in the engine; the rest by router | library state 10/55, provider 6/6, watchlist 3/22, calendar 1/13, local media 0/9 |
| `catalog` | `identity`, `search`, `tmdb`, `mdblist`, `publicmetadb`, `warnings`, `anime` | `identity` and `search` in the engine; the rest by router | search 6/23, tmdb 6/26, mdblist 1/52, publicmetadb 0/40, warnings 2/2, anime 0/2 |
| `player` | `flow`, `policy`, `stream_policy`, `stream_badges`, `scrobble`, `subtitle_sync`, `desktop`, `cast`, `intro_segments`, `trailer_subtitles`, `watch_together`, `dolby_vision` | `flow` and `stream_policy` in the engine; `cast` and `dolby_vision` have no route | policy 3/19, stream policy 1/18, scrobble 0/11, intro segments 0/23, watch together 0/10, badges 0/6 |
| `addons` | `protocol`, `resource`, `store`, `uptime`, `plugins`, `plugin_network`, `discovery`, `repository`, `headless_adapter`, `platform` | `store` in the engine; the rest by router | protocol 7/17, store 3/17, resource plan 6/19, adapter plan 2/4, plugins 0/8, resource 0/6 |
| `accounts` | `external_sync`, `nuvio_sync`, `fluxa_sync`, `nuvio_pin`, `device_auth`, `integrations`, `oauth` | router only; `oauth` has no route | nuvio sync 11/30, trakt 1/47, simkl 0/19, anilist 0/6, fluxa sync 0/5, device auth 0/2 |
| `profile` | `contract`, `prefs`, `avatar_pack` | router only | contract 9/13, avatar pack 5/5, prefs 0/4 |
| `settings` | `contract`, `data_policy`, `device_resource`, `version`, `checksum`, `runtime_label`, `discord_presence` | router only | data policy 0/3, device resource 0/2, discord 0/1, version 0/1 |

## Engine coverage

Actions the engine handles today: navigation, detail (load, local state, secondary, prefetch, streams, selected addon), meta detail, direct and continue-watching playback, intro segments, player stream loading and telemetry, scrobble, profile activation, home load, continue-watching refresh, library hydrate, watchlist and library status toggles, feedback, playback progress save and clear, mark watched, addon install, remove and move.

Effects the engine can emit are listed in `EffectKind` (`src/runtime`) and described in `effects.md`.

## Still to wire

Each item names the action the shell would dispatch and the effect the shell would run. The domain function already exists in every case.

- **Trakt, Simkl, AniList sync** (`accounts::external_sync`): an action to start and finish a sync, and `RunExternalSync` / `SyncExternalIntegration` already exist as effects; the request and response planning still goes through the router.
- **PublicMetaDB** (`catalog::publicmetadb`): a detail-load action that asks for ratings, and a fetch effect for its endpoints.
- **Intro segments** (`player::intro_segments`): `IntroSegmentsRequested` and `FetchIntroSegments` exist; the remaining parsing and dedupe routes should move behind them.
- **Watch together** (`player::watch_together`): room create, join and sync actions, and a socket effect. Nothing in the engine models a room yet.
- **Casting** (`player::cast`): DLNA, Chromecast, FCast, Roku and AirPlay message builders. They need a device discovery effect and a cast session action; today they are only reachable as plain Rust functions.
- **Subtitle sync** (`player::subtitle_sync`): actions for estimate, capture and apply, and a subtitle fetch effect (`FetchSubtitles` exists).
- **Scrobble** (`player::scrobble`): `ScrobbleRequested` and `EnqueueTraktScrobble` exist; the scrobble body routes are not called by any shell.
- **Plugins** (`addons::plugins`, `fluxa-plugin-runtime`): `ExecutePlugin` and `FetchPluginManifest` exist; scraper settings and repository management have no action.
- **Local media** (`library::local_media`), **offline downloads** (`library::offline_download`): `EnqueueOfflineDownload` exists; local library scanning has no action.
- **Settings and data policy** (`settings::*`): `WriteSettings` exists; cache trimming and failure policy have no caller.
- **Device auth, Nuvio PIN, Fluxa sync** (`accounts`): `RunAuthFlow` and `RefreshAuthToken` exist; the PIN and Fluxa account flows need their own actions.
- **Dolby Vision** (`player::dolby_vision`): the playback plan is used by `fluxa-media`, which does the RPU and sample rewriting. The engine needs a playback-prepare step that returns the plan.

## Moved out of the core crate

- `fluxa-media`: Matroska demux, fMP4 and WebM muxers, Dolby Vision RPU and sample rewriting, and their wasm exports.
- `fluxa-plugin-runtime`: the QuickJS plugin sandbox with its crypto, DOM and network bridges.
- `fluxa-streaming-engine`: unchanged; it now calls `player::stream_policy` directly.

The `dolbyVisionConvertRpu` and `dolbyVisionRpuInfo` routes are no longer in the string router. Their functions live in `fluxa_media` and are reachable only as Rust calls.
