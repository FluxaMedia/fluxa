# fluxa-core robustness & performance plan

Goal: move the crate toward "if it compiles, it doesn't break at runtime" and make the
hot paths (dispatch, effect completion, stream ranking) cheap enough to never show up in
a profile. This document is the result of a full review of the crate as of 2026-07-07
(master @ f00a09a) and lays out what's already good, where runtime breakage can still
hide, and a prioritized plan to close each gap.

## Implementation status (2026-08-22)

The original review is historical; the current tree already contains several of its
proposed fixes. These are now enforced in code:

- `CoreError` plus the process-local log sink are used by the headless and routed FFI
  boundaries, so malformed JSON is recorded before the legacy `null`/`None` response.
- `EffectResultInput` has required `effectId`/typed `status` fields.
- `Profile`, `MetaItem`, `Video`, and `Stream` preserve unknown wire fields through
  flattened extras. High-volume detail/player action inputs now deserialize into those
  types before being converted to the existing state representation.
- `EffectEnvelope.kind` is `EffectKind`, and dynamic player effects must resolve through
  that enum before they can be registered.
- `StatePatch` uses dirty-domain tracking, and patch serialization happens after the
  engine mutex is released.
- Desktop verifies the generated Rust method inventory against Android's 190
  `coreInvokeValue` consumers.

The remaining items are executable wire fixtures/fuzz coverage, benchmark evidence for
the hot paths, and repository CI/lint enforcement. They are separate from the typed
wire migration above and should not be reimplemented by duplicating the same models.

## Current state, honestly assessed

What's already strong and should be preserved:

- `headless_engine` state is fully typed — `EngineState` contains zero `serde_json::Value`
  fields at the top level (`state.rs`), cross-module mutation goes through `pub(super)`
  setters, and generations use the `GenerationKey` enum.
- Every JNI entry point wraps its body in `catch_unwind`, and `core_invoke` does too, so
  a panic can't cross the FFI boundary and abort the Android process.
- Production code is nearly panic-free: outside test modules the only `expect` is a
  static regex in `addon_store.rs:748`, which can't fail for a fixed pattern.
- Poisoned-mutex recovery exists (`lock_engines` uses `into_inner()`), so one panicking
  dispatch can't brick every later call.
- `StatePatch::diff` sends only changed domains over the wire instead of the full state.
- Fuzz targets exist (`episode_matching`, `manifest_parse`, `percent_decode`).
- Effect delivery has dedup (`delivered_effect_ids`) and expiry (`EFFECT_EXPIRY`), so a
  lost `completeEffect` doesn't leak or double-execute.

What undermines the "compiles ⇒ works" goal, in order of real-world risk:

## P0 — fix immediately

### 1. Master has a failing test

`player_policy::tests::convert_dv81_hls_still_deferred_to_manifest_rewrite` fails.
The branch at `player_policy.rs:445` returns `hls_rpu_convert` for
HLS + P7 + `convert_dv81` + DV decoder, while the test still expects
`none`/`manifest_handled`. Either the new branch is a policy change and the test is
stale, or the branch regressed the "HLS is always the OkHttp interceptor's job" rule
that `sample_hls_stream_always_deferred_to_manifest_rewrite` documents. Decide which,
fix, and add a CI gate (see §8) so a red master can't happen silently again.

## P1 — silent failures at the boundary

### 2. `Option<String>` swallows every error

The dominant FFI pattern is `*_json(&str) -> Option<String>` (~295 functions), almost
always built on `serde_json::from_str(...).ok()?` (~135 sites). Any malformed or
schema-drifted input produces `None` → the host receives `null` with zero diagnostics.
This is the single biggest source of "compiles but breaks at runtime": a Kotlin-side
field rename doesn't crash anything, it just makes a feature quietly stop working.

Plan, incremental and wire-compatible:

- Introduce one internal error type:
  ```rust
  pub(crate) enum CoreError {
      BadInput { context: &'static str, detail: String },
      NotFound { context: &'static str },
  }
  ```
  Migrate module internals from `Option<T>` to `Result<T, CoreError>`, mechanical
  module by module (`?` still works). Keep the outer `Option<String>` shims so JNI
  signatures and the Kotlin side don't change.
- At each boundary shim, on `Err` route the detail through a host-pluggable logger
  before returning `None`/`null`. Add a `core.setLogSink` (JNI: one extern that stores
  a JVM callback; desktop: a `fn(String)` setter) so failures land in logcat / desktop
  logs instead of vanishing. This turns every future wire drift from "feature silently
  dead" into a one-line log with the exact field that failed to parse.
- `core_invoke` already has an error envelope — have its shims return the real
  `CoreError` kind/message instead of collapsing to `NotFound` via `Option`.

### 3. `EffectResultInput` accepts garbage as success

Every field is `#[serde(default)]` (`contracts.rs:318`), so a completely wrong-shaped
`completeEffect` payload deserializes into `effect_id: ""`, `status: ""` — the engine
then looks up effect `""`, finds nothing, and the completion is dropped without a trace.
Make `effect_id` required, make `status` an enum:

```rust
#[serde(rename_all = "camelCase")]
enum EffectStatus { Ok, Error, Cancelled }
```

Reject unknown statuses at parse time with a logged error (via §2's sink) instead of
treating them as an empty string that matches no arm.

## P2 — make invalid states unrepresentable

### 4. `Value` leaves inside the typed engine

`EngineState` is typed, but the payloads flowing through it are not: ~370 `Value` /
`Vec<Value>` / `Option<Value>` occurrences across `headless_engine/*.rs`. The repeat
offenders in `AppAction` are `profile: Option<Value>`, `meta: Value`,
`streams: Vec<Value>`, `item: Value`. Every `.get("...").and_then(as_str)` chain on
these is a stringly-typed runtime contract the compiler can't check.

Plan: define the three or four shared wire types once, in a new `types/wire.rs` (the
`types/` module dir already exists), with `#[serde(rename_all = "camelCase")]` and
`#[serde(default)]` on genuinely optional fields:

- `Profile` — `profile_prefs.rs` already does a typed read of the same JSON; promote
  that struct instead of writing a new one.
- `MetaItem` / `Video` — the detail/library/watchlist modules all re-derive the same
  fields (`id`, `type`, `name`, `poster`, `videos`, episode fields) from `Value`.
- `Stream` — `stream_policy.rs` already has typed stream structs; reuse them in
  `AppAction::PlayerStreamsLoaded` and friends.

Crucial constraint: any field the platform sends that Rust doesn't model must survive
round-trips, because effect payloads echo these objects back. Use
`#[serde(flatten)] extra: serde_json::Map<String, Value>` on each wire type so unknown
fields pass through unchanged. This lets migration happen field-by-field without ever
breaking a consumer that sends more than we model.

Migrate one action at a time; the JSON on the wire never changes, so this is invisible
to the platforms. Priority order by breakage frequency: `profile` (touched by ~20
actions), `meta`, `streams`, `item`.

### 5. `json!` payload literals defeat compile-time checking

The stated convention is "effect payloads are small `#[derive(Serialize)]` structs",
but `headless_engine/` still has ~80 `json!({...})` payload literals. A typo'd key in
one of those compiles fine and the platform-side effect executor gets a field it
doesn't recognize. Finish the migration: one payload struct per effect kind, named
after it (`FetchDetailStreamsPayload`), living next to the module that emits it.
Once done, add `json!` to the review checklist / grep gate for `headless_engine/`
(it stays fine in tests and in `ffi.rs` envelope construction).

### 6. Stringly-typed enums that already have a closed set

- `EffectEnvelope.kind` is `String` even though `EffectKind` exists and has an
  exhaustive test. Change the field to `EffectKind` with
  `#[derive(Serialize, Deserialize)] #[serde(rename_all = "camelCase")]` on the enum —
  serde then generates exactly the strings `as_str()` produces today, the 100-line
  hand-written `as_str`/`from_str` pair and its roundtrip test get deleted, and adding
  a variant can no longer be forgotten in one of three places. `EffectEnvelope::raw`
  has to go or become test-only; audit its callers first.
- `fallback_mode` in `player_policy.rs` is compared as `"convert_dv81"` / `"auto"` /
  `"off"` string literals throughout `dv_proxy_plan_json`. Parse once into an enum at
  the request boundary; a typo in a mode string then fails at the one parse site, not
  silently in whichever comparison spelled it wrong.
- Same treatment for `source_selection_mode`, scrobble `action_name`, and auth
  `provider`/`mode` — each is a closed set spread across string comparisons today.

## P3 — the three uncoordinated FFI surfaces

### 7. One method inventory instead of three hand-wired ones

`ffi.rs` routes ~115 method names by string; `jni.rs` hand-wires ~105 externs; nothing
guarantees the same domain function is exposed consistently or spelled the same. The
CLAUDE.md assessment that a full consolidation isn't worth the rewrite risk stands —
but a cheap middle ground removes most drift without touching `jni.rs`'s hot path:

- Declare the string-routed surface in one table macro:
  ```rust
  core_methods! {
      "streamPolicy.rankStreams" => stream_policy::rank_streams_json(json),
      "engine.dispatch" => (handle: u64, action: str) headless_engine::headless_engine_dispatch_json,
      ...
  }
  ```
  generating the `route_*` functions in `ffi.rs`. Adding a method becomes one line;
  the per-domain router boilerplate and its UnknownMethod-chaining protocol disappear.
- Generate `core_capabilities` from the same table so the platform can feature-detect
  instead of calling methods that may not exist in an older core build.
- Leave `jni.rs` as-is (it's Android's proven path), but add a unit test that walks the
  table and asserts every `core_invoke` method name a consumer repo references (kept in
  a checked-in fixture list) still routes. That converts "renamed a method string"
  from an Android-runtime discovery into a test failure here.

## P4 — cross-repo wire contract

### 8. The wire format has no executable contract

Field names in `dispatch`/`completeEffect`/effect payloads are read by Kotlin/TS in
other repos; today the only protection is care. Two mechanisms, both cheap:

- **Golden fixtures**: a `tests/wire/` directory of checked-in JSON files — one real
  `AppAction` input and one expected `DispatchResult` output per action family, plus
  one payload sample per `EffectKind`. A snapshot test deserializes, dispatches, and
  compares serialized output structurally. Any refactor that changes camelCase output
  now fails a test in *this* repo instead of breaking Android in production. When a
  change is intentional, the diff in the fixture file is the review artifact the
  platform PRs can be checked against.
- **Fuzz the real boundary**: the existing fuzz targets cover parsers, but the highest-
  value target is missing — `headless_engine_dispatch_json` + `complete_effect_json`
  with arbitrary bytes and with structure-aware mutation of valid actions. The engine
  holding global state behind `catch_unwind`-free internals (dispatch is only guarded
  at the JNI layer, not in the desktop path via direct calls) makes this the place a
  panic or logic corruption would actually hurt. Add `engine_dispatch.rs` to
  `fuzz/fuzz_targets/`.

## P5 — performance

The release profile is already right (`lto = true`, `codegen-units = 1`, opt-level 3).
Keep `panic = "unwind"` — the FFI `catch_unwind` guards depend on it; `panic = "abort"`
would turn every guarded panic into a process kill. The actual costs found:

### 9. Two full state clones per dispatch

`headless_engine_dispatch_json` (`mod.rs:88-100`) clones the entire `EngineState`
before the reducer runs, clones it again after, then `StatePatch::diff` does a deep
`PartialEq` walk of all 16 domains and clones each changed one a third time. Cost
scales with everything the user has ever loaded (full catalogs, episode lists), not
with what the action touched — on a TV device after a long session this is the
dispatch latency.

Fix without changing the wire format: replace clone-and-diff with dirty tracking.
Each domain module already owns its state behind `pub(super)` setters — the setters
are the natural single place to flip a `dirty: DomainMask` bit on the engine.
`StatePatch` then clones only domains whose bit is set, and the before-clone
disappears entirely. `PartialEq` on the domain structs stays for tests. This is the
single highest-leverage performance change in the crate; do it before micro-work.

Second-order: `snapshot_json` and dispatch serialize while holding the global engines
mutex. Move `serde_json::to_string` outside the lock (clone the patch domains inside,
serialize after release) so a slow serialization of a big home state can't block a
concurrent `completeEffect`.

### 10. Per-call regex compilation in ranking loops

`stream_policy.rs:548,594,608` build a `Regex` from the user's language preference on
every call — and these run inside per-stream ranking. `external_sync.rs:65` compiles
`tt\d+` per history item; `watchlist_plan.rs:216` compiles a static GitHub-URL regex
per call. `content_identity.rs:980` already shows the house pattern (`OnceLock`).
Statics move to `OnceLock`; the preference-derived ones get a one-entry cache keyed by
the preference string (a `Mutex<Option<(String, Regex)>>` is enough — the preference
changes once per settings edit, not per stream).

### 11. Desktop pays JSON tax it doesn't owe — RESOLVED, already fixed on the desktop side

Verified against `apps/desktop/src-tauri` call sites (2026-07) per the CLAUDE.md rule, and
this section's premise no longer holds. The hottest paths — `dispatchAction`,
`completeEffect`, `getSnapshot` in `src/core/engine.ts` — already go through dedicated Tauri
commands (`engine_dispatch`, `engine_complete_effect`, `engine_snapshot` in
`apps/desktop/src-tauri/src/lib.rs`) that call `FluxaCore::headless_engine_dispatch_json` etc. directly in
Rust, not through `core_invoke`. `FluxaCore` has also grown from the "8 methods" this doc
assumed to 21 (cast/AirPlay/Chromecast/Roku helpers included), all with real desktop call
sites. The one remaining `core_invoke` Tauri command is a generic fallback for the long tail
of infrequent methods (`playbackPreparePlan`, `preferencesSchema`, etc.) — writing a
dedicated Tauri command per method there would be pure boilerplate, since Tauri IPC already
serializes args as JSON crossing the JS↔Rust boundary regardless of whether the command is
generic or dedicated; there's no double-parse to eliminate. Nothing left to do here.

### 12. Measure before and after

Add a `benches/` with criterion (dev-dependency only): dispatch on a realistic large
state (fixture from §8's golden files), `StatePatch::diff` vs dirty-tracking, stream
ranking with 500 streams. Without these, §9-§11 are guesses; with them, regressions
show up in review.

## P6 — enforcement so it stays fixed

### 13. Lint and CI gates

There is no CI in this repo and no lint policy in `Cargo.toml`. Add:

```toml
[lints.rust]
unsafe_op_in_unsafe_fn = "warn"

[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
indexing_slicing = "warn"
panic = "warn"
```

`warn` not `deny` initially — test modules use these legitimately; add
`#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]` and ratchet to
`deny` once the count is zero. CI (GitHub Actions, single workflow): `cargo test --lib`,
`cargo clippy -- -D warnings` on default features, `cargo check --no-default-features
--features wasm` (allowing its known dead-code noise per CLAUDE.md), and `cargo fmt
--check`. The wasm job is what catches "compiles on Android, broken on webOS" cfg
mistakes.

## Sequencing

| Order | Work | Sections | Size |
|---|---|---|---|
| 1 | Fix failing DV test, stand up CI | §1, §13 | DV resolved; CI remains |
| 2 | Golden wire fixtures + dispatch fuzz target | §8 | remaining |
| 3 | `CoreError` + log sink at boundaries | §2, §3 | implemented incrementally |
| 4 | Dirty-flag `StatePatch`, serialize outside lock, benches | §9, §12 | dirty/lock work implemented; benches remain |
| 5 | Typed `Profile`/`MetaItem`/`Stream` wire structs | §4 | implemented incrementally |
| 6 | `EffectKind` on envelope, mode enums, payload structs | §5, §6 | effect kind implemented; mode/payload migration remains |
| 7 | `core_methods!` table + capability generation | §7 | consumer inventory guard implemented; table macro remains |
| 8 | Desktop typed fast path | §11 | resolved in current desktop architecture |

Steps 2 and 3 are what make step 5 safe: once golden fixtures exist and parse failures
are logged instead of swallowed, converting `Value` fields to typed structs can be done
aggressively — any drift shows up as a fixture diff or a logged parse error instead of
a silent feature death on a platform we can't see from this repo.
