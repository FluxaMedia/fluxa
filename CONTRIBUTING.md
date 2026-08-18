# Contributing to Fluxa for Android

Thanks for wanting to help. Fluxa is the Android client — a Kotlin/Compose shell over
[fluxa-core](https://github.com/FluxaMedia/fluxa-core), the shared Rust brain. The shell
handles all platform I/O (HTTP, Room, ExoPlayer, audio, notifications); the decisions live
in Rust and reach Kotlin as typed effects.

## The shell does I/O, the core decides

Rust never touches the network or storage directly. It updates domain state and emits a
`NativeHeadlessEffect`; Kotlin executes it with OkHttp, Room, auth, etc., then completes it
so Rust can update state.

```text
Kotlin -> dispatchHeadless(action)
       -> Rust updates state, emits NativeHeadlessEffect
       -> Kotlin performs the I/O (OkHttp, Room, auth, …)
       -> Kotlin -> completeEffect(result)
       -> Rust updates state
```

A `FluxaCoreNative.fetchSomething()` that calls the network inside Rust is the wrong shape.
If a feature needs Rust to do network or storage work, model it as an effect:
`FluxaHeadlessAppRuntime.dispatch()` emits it, `FluxaAndroidHeadlessEnvironment.execute()`
performs the I/O, and `complete()` returns the result. Domain logic — stream policy, home
ranking, library rules, scrobble decisions — belongs in fluxa-core; wiring, rendering, and
platform SDKs stay in Kotlin. See [`CLAUDE.md`](CLAUDE.md) and [`ARCHITECTURE.md`](ARCHITECTURE.md)
for the full boundary.

## Keep Compose leaves light

Rendering composables receive small immutable UI models and primitives — not `Meta`,
`Stream`, manifests, repositories, or ViewModels. Route/orchestrator composables may touch
domain objects; effects and coordinators do the heavy work; by the time data reaches a
reusable leaf it should be a `PlayerContentUiModel`, `CatalogCardUiModel`, or a
section-specific row model. If a component needs one field off a domain object, add that
field to a UI model instead of passing the whole object.

## Getting set up

```bash
git clone https://github.com/FluxaMedia/fluxa.git
cd fluxa
./gradlew :app:assembleMobileDebug   # phones & tablets
./gradlew :app:assembleTvDebug       # Android TV
```

**Prerequisites**

- JDK 17+
- Rust stable with the Android NDK targets:
  `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android`
- Android NDK (version pinned in `app/build.gradle.kts`)

The Rust libraries (`fluxa_core`, `fluxa_streaming_engine`) cross-compile for every ABI as
part of the Gradle build — no manual `cargo build` step. If you're changing Rust behaviour,
you can iterate faster against fluxa-core's own `cargo test --lib` before rebuilding the app.

## Before you open a PR

- `./gradlew :app:assembleMobileDebug` builds clean
- Run the unit tests for the modules you touched (`./gradlew :<module>:test`)
- If you changed Rust, `cargo fmt` / `cargo clippy` / `cargo test --lib` in the crate first

## i18n

Never hardcode user-facing text. Every visible string loads with `AppStrings.t(lang, "key")`,
and the key must exist in **both** language files before you use it:

- `core/src/commonMain/resources/i18n/english_us.json`
- `core/src/commonMain/resources/i18n/tr_tr.json`

Key prefixes: `auto.*` (shared), `settings.*`, `nav.*`, `common.*` (repeated labels/buttons).

## Style

- **No comments in code.** Not inline, block, or doc comments — Kotlin or Rust. If something
  needs explaining, it goes in the commit message or PR description, not the source.
- **English-first, strictly.** Source names, identifiers, comments, docs, logs, test names,
  commit messages, PR and issue text, and CI configs must all be English. The *only*
  non-English text allowed is user-facing localized strings, and those live in the i18n JSON
  files — never inline.
- Match the surrounding code's conventions; don't introduce a new pattern where an existing
  one fits.
- Keep changes focused — one logical change per commit, no unrelated refactors folded in.

## Commits and pull requests

- Write a real commit message: what changed and *why*, imperative mood, English.
- In the PR, note whether you tested mobile, TV, or both, and on what API level if it
  matters.
- New behaviour needs a test where it's testable; a bug fix should include the test that
  would have caught it.

## Reporting bugs

Open an issue with your device, Android version, whether it's the mobile or TV build, the
Fluxa version, and steps to reproduce. A logcat excerpt around the failure helps a lot.

## Legal

Fluxa is a client for user-installed Stremio addons — it hosts and distributes no content.
Don't add anything that changes that. Contributions are licensed under the repository's GPLv3.

Questions are welcome on [Discord](https://discord.gg/wan9FeDEfe).
