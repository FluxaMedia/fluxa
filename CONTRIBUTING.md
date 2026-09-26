# Contributing to Fluxa

Fluxa is a media hub with shared Rust application logic and platform shells for Android, Apple platforms, desktop, web, and webOS.

## Repository layout

```text
apps/android   Android and Android TV shell plus shared KMP modules
apps/apple     iOS and tvOS host code
native/        Rust renderer and shells for desktop, web, and webOS
core/          Shared Rust domain logic and streaming engine
shared/i18n    Shared English and Turkish application strings
```

Put decisions, policies, state transitions, and cross-platform contracts in `core/fluxa-core`. Keep UI, storage, networking, lifecycle, players, and OS integrations in the relevant platform shell. Do not duplicate a business rule in a platform just because calling the core is inconvenient.

## Boundary questions

Before adding code to a shell, ask:

- Is this a policy, state transition, normalization, ranking, or wire contract? If so, it belongs in `core/fluxa-core`.
- Is this only URLSession/OkHttp/fetch, storage, player, lifecycle, or OS code? If so, keep it in the platform shell.
- Is the same effect/action name or payload declared in more than one platform? Update the Rust contract and run `npm run generate:contracts`; do not hand-edit generated contract files.
- Is an existing typed resolver, loader, or adapter already responsible for this data? Extend that boundary instead of creating a second parser or network flow.
- Does a model contain both transport fields and derived playback/UI behavior? Keep the transport model small and move derived behavior into a policy or adapter.

The generated headless contract snapshots live in `shared/contracts`. Platform-specific models may still differ when their UI or SDK needs differ, but their wire names and core-owned decisions must not drift.

## Local checks

```bash
npm run check:structure
npm run check:contracts
npm run check:i18n
npm run check:native
npm run check:core
npm run check:wasm
npm run check:android
npm run check:apple
```

For Android or Apple changes, run the relevant Gradle tasks from `apps/android` on a machine with the required SDKs. Avoid dependency downloads unless you are on Wi-Fi; use offline mode where possible.

When changing user-facing copy, edit `shared/i18n` and run `npm run generate:i18n` so Android's generated resources stay synchronized.

## Pull requests

- Keep changes focused and preserve the shared/core boundary.
- Add or update a focused test for new behavior or a regression.
- Check every affected target, not only the target used during development.
- Keep user-facing strings in the platform localization files.
- Do not commit generated build output, credentials, SDK state, or downloaded media libraries.

Describe what changed, why it belongs in the chosen layer, and which checks were run.

## Legal

Fluxa is a client for user-installed addons. It does not host or distribute media. Contributions are licensed under GPLv3.
