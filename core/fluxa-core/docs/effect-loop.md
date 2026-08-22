# The effect loop

The effect loop is how the core communicates with the host platform. The core never performs I/O — it returns a list of effects describing what it needs done, and the host executes them and reports results back.

## Basic cycle

Every interaction follows this pattern:

```
1. Host calls dispatch(handle, actionJson)
2. Core updates state, produces a list of effects
3. Core returns { revision, state, effects }
4. Host executes each effect
5. Host calls completeEffect(handle, { effectId, result })
6. Core may produce more effects in response — goto 4
```

Effects from a single dispatch can chain: completing effect A may produce effects B and C, completing B may produce effect D, and so on until the operation is fully resolved.

## DispatchResult shape

Every `dispatch` and `completeEffect` call returns the same JSON envelope:

```json
{
  "state": { ... },
  "effects": [
    {
      "id": "fx-1",
      "type": "fetchAddonResource",
      "generation": 3,
      "payload": { ... }
    }
  ]
}
```

- `state` is a `StatePatch`: only UI/domain branches changed by this operation. Platforms merge it into their current UI state. Use `snapshot` when a complete state is required.
- `effects` is the list of work the platform must execute. An empty array means no I/O is needed.
- `id` is an opaque monotonically-increasing string (`"fx-N"`). The platform echoes it back in `completeEffect`.
- `generation` lets the platform discard completions that arrived after a newer dispatch superseded them.
- Pending effects are runtime-only. They are never included in `state` or snapshots because effect payloads may contain credentials.

The machine-readable source for lifecycle methods, effect names and envelope fields is `core_invoke("coreContractManifest", {})`.

## completeEffect shape

```json
{
  "effectId": "fx-1",
  "result": { ... }
}
```

The `result` structure is effect-specific. See [effects.md](effects.md) for each type's expected payload.

## Stale effects

Effects that the platform never completes (dropped exceptions, IPC failures) are expired after 5 minutes. The core will stop waiting for them and they will not produce further state changes.

## Actions

Actions are dispatched as tagged JSON objects. The `type` field is camelCase and selects which handler runs. Example:

```json
{
  "type": "homeLoadRequested",
  "profile": { ... },
  "language": "en",
  "force": false
}
```

All action types are defined in `src/headless_engine/contracts.rs` (`AppAction` enum). The serde tag is the wire name.

### Common actions

| type | When to dispatch |
|---|---|
| `navigationRequested` | User navigates to a new screen |
| `homeLoadRequested` | Home screen loads or refreshes |
| `detailLoadRequested` | Detail screen opens for a content item |
| `detailStreamsRequested` | Stream list needs to be fetched from addons |
| `playerLoadStreamsRequested` | Player opens and needs stream resolution |
| `playerResolvePlaybackRequested` | A specific URL/stream needs playback resolution |
| `savePlaybackProgressRequested` | User position should be persisted |
| `toggleWatchlistRequested` | User adds/removes an item from watchlist |
| `profileActivated` | Active profile changed |
| `introSegmentsRequested` | Intro skip segments needed for an episode |
| `scrobbleRequested` | Trakt/Simkl scrobble should be sent |
| `homeSearchRequested` | User typed a search query |
| `discoverLoadRequested` | Discover/browse screen loads |
| `libraryHydrateRequested` | Library state should be refreshed |

## Engine lifecycle

The headless engine is accessed through an integer handle. Multiple engine instances can coexist (though typically one exists per app session).

### Via `FluxaCore` (desktop/Tauri)

```rust
let handle = FluxaCore::create_headless_engine(initial_json);
let result = FluxaCore::headless_engine_dispatch_json(handle, action_json);
let result = FluxaCore::headless_engine_complete_effect_json(handle, effect_result_json);
let snapshot = FluxaCore::headless_engine_snapshot_json(handle);
```

### Via `core_invoke` (desktop/Swift)

```json
// Create
core_invoke("engine.create", "<initial_json>")
→ { "ok": true, "value": 1 }

// Dispatch
core_invoke("engine.dispatch", { "handle": 1, "action": { "type": "homeLoadRequested" } })
→ { "ok": true, "value": { "state": {...}, "effects": [...] } }

// Complete an effect
core_invoke("engine.completeEffect", { "handle": 1, "result": { "effectId": "fx-1", "result": {...} } })
→ { "ok": true, "value": { "state": {...}, "effects": [...] } }

// Destroy
core_invoke("engine.destroy", 1)
```

### Via JNI (Android)

Android calls the corresponding `Java_com_fluxa_app_core_rust_FluxaCoreNative_*` native methods from `FluxaCoreNative.kt`. The JNI functions wrap the same underlying `headless_engine::*` functions.

## Error envelope

`core_invoke` always returns a string. On failure:

```json
{
  "ok": false,
  "error": {
    "kind": "unknown_method",
    "message": "no such method `foo`",
    "method": "foo"
  }
}
```

Error kinds: `unknown_method`, `invalid_args`, `not_found`, `internal`.
