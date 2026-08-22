# fluxa-core overview

fluxa-core is the platform-agnostic Rust brain for the Fluxa media-streaming app. It holds all domain logic — content discovery, stream selection, playback state, profiles, library, calendar, external sync — and never does I/O itself. Instead, it emits typed *effects* that the host platform executes (network calls, storage reads, player commands, etc.) and reports back via `completeEffect`.

The result is a single Rust codebase that runs on Android, desktop (Linux/macOS/Windows), iOS, and webOS, with each platform providing a thin shell that drives the effect loop.

## Architecture

```
Host  →  dispatch(action_json)
      ←  { state, effects: [{ id, type, payload }] }

Host  →  executes each effect (HTTP / storage / player / ...)
      →  completeEffect({ effectId, result })
      ←  { state, effects: [...] }
```

The core never initiates anything. Every state transition begins with the platform dispatching an action.

## Two state engines

**`headless_engine/`** is the primary, actively-developed engine. State is held in a typed `EngineState` struct composed of per-feature sub-structs (`HomeState`, `DetailState`, `PlayerState`, etc.). Cross-module mutation goes through `pub(super)` setters — never reaching across module boundaries directly.

**`app_state.rs`** is a lighter, independently-maintained engine for overlapping concerns (home/discover/calendar/library/player). It is used by Android via UniFFI (`createAppCoreStateJson` / `appCoreDispatchJson`). The two engines are intentionally separate — don't try to merge them.

## Three exposure mechanisms

| Mechanism | Used by | File |
|---|---|---|
| `FluxaCore` struct | Desktop (Tauri) | `src/core_api.rs` |
| `core_invoke(method, args_json)` | Desktop + Swift | `src/ffi.rs` |
| JNI externs | Android | `src/bindings/jni.rs` |

`FluxaCore` is intentionally minimal and contains only methods with verified desktop callers. Everything else goes through `core_invoke` (for desktop/Swift) or raw JNI bindings (for Android). `coreContractManifest` exposes the shared lifecycle/effect contract for generated bindings and drift checks.

## Module map

| Module | Responsibility |
|---|---|
| `headless_engine` | Primary state machine — typed `EngineState`, action dispatch, effect emission |
| `app_state` | Secondary state engine, Android/UniFFI only |
| `core_api` | `FluxaCore` struct, 8 methods, desktop only |
| `ffi` | `core_invoke` string-routed dispatcher (~115 methods) |
| `runtime` | `EffectKind` / `EffectEnvelope` types |
| `bindings/jni` | ~157 JNI externs for Android |
| `bindings/uniffi` | UniFFI exports (headless engine lifecycle, `coreCapabilities`, version) |
| `bindings/wasm` | WASM exports for webOS |
| `addon_protocol` | Manifest URL normalisation, resource URL construction, manifest merging |
| `addon_store` | Addon search policy, CloudStream/plugin repo URL normalisation |
| `addon_resource` | Addon HTTP response classification (success / empty / error) |
| `stream_policy` | Stream selection, magnet building, audio/subtitle preference matching |
| `player_policy` | Backend selection, track state, Dolby Vision fallback policy |
| `player_flow` | Self-contained typed playback sub-engine (load → select → play) |
| `player_scrobble` | Trakt/Simkl scrobble body construction |
| `content_identity` | ID parsing/normalisation (IMDB, TMDB, Kitsu, episode locators) |
| `home_ranking` | Billboard/shelf ordering, continue-watching dedup, personalization scoring |
| `library_state` | Continue-watching badge computation, next-episode resolution |
| `profile_contract` | Profile activation, auth token merging, settings migration |
| `profile_prefs` | Typed read of user preferences from raw profile JSON |
| `search_plan` / `discovery_plan` | Search/discover query planning, result grouping, sort |
| `calendar_plan` | Calendar item filtering, widget rows, release notifications |
| `external_sync` | Trakt and Simkl API response parsing and history mapping |
| `intro_segments` | introdb.app and AniSkip segment parsing, deduplication |
| `dolby_vision_rpu` | Dolby Vision RPU metadata extraction (`native` feature, JNI-only) |
| `platform_plan` | Season/episode navigation planning |
| `tmdb_plan` | TMDB ID resolution hints, trailer mapping |
| `watchlist_plan` | Watchlist toggle, offline grouping, progress merging |
| `offline_download` | Offline download plan construction |
| `data_policy` | Cache/data-failure policy |

## Companion crate

`fluxa-streaming-engine/` lives in the same repo. It handles the runtime streaming side — torrent via librqbit, HTTP proxying via axum, Dolby Vision bitstream rewriting. It exposes its own JNI bindings (`bindings/jni.rs`) and three CLI tools (`torrent_bench`, `torrent_serve`, `companion_server`).
