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

## Entry points

| Entry | Used by | File |
|---|---|---|
| `Engine` (`dispatch`, `complete`) | Every shell | `src/headless_engine` |
| `core_invoke(method, args_json)` | Desktop plan calls, Swift, WASM | `src/ffi.rs` |

`Engine` is the way in. `core_invoke` is a string-routed shim over the domain modules that shrinks as each domain is wired to the engine; see [`wiring-map.md`](wiring-map.md). `coreContractManifest` exposes the lifecycle and effect contract for generated bindings and drift checks.

## Module map

| Module | Responsibility |
|---|---|
| `headless_engine` | Primary state machine: typed `EngineState`, action dispatch, effect emission |
| `app_state` | Secondary state engine |
| `ffi` | `core_invoke` string router |
| `runtime` | `EffectKind` / `EffectEnvelope` types |
| `bindings/wasm` | WASM exports for webOS |
| `home` | Shelf ordering, hero plan, recommendations |
| `library` | Library state, continue watching, watchlist, calendar, persistence, offline downloads |
| `catalog` | ID parsing, search, TMDB, MDBList, PublicMetaDB, content warnings |
| `player` | Playback flow and policy, stream selection, scrobble, subtitles, casting, intro segments, Dolby Vision plan |
| `addons` | Addon protocol, store, resources, plugins, repository flows |
| `accounts` | Trakt, Simkl, AniList, Nuvio and Fluxa sync, device auth, OAuth plans |
| `profile` | Profile contract, preferences, avatar packs |
| `settings` | Settings contract, data policy, device resources, version and checksum policy |

## Companion crate

`fluxa-streaming-engine/` lives in the same repo. It handles the runtime streaming side: torrent via librqbit, HTTP proxying via axum, Dolby Vision bitstream rewriting (with `fluxa-media`). It has three CLI tools (`torrent_bench`, `torrent_serve`, `companion_server`).
