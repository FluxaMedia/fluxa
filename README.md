<div align="center">

<img src="app/src/mobile/res/mipmap-nodpi/ic_launcher.png" alt="Fluxa" width="96" />

# Fluxa

A fast, native media client for Android phones, tablets, and TV.<br/>
Browse catalogs, track what you watch, and play anything the Stremio addon ecosystem exposes.

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stars][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![License][license-shield]][license-url]

[Download](#download) · [Features](#features) · [Building from source](#building-from-source)

</div>

---

## What it does

Fluxa connects to any Stremio-compatible addon and turns it into a proper Android app: a home feed with genre and category browsing, a calendar of upcoming episodes, a library with continue-watching and resume positions, and two-way watch tracking with Trakt, MyAnimeList, and Simkl. Playback runs through Media3/ExoPlayer and MPV, including direct torrent/magnet support, with no telemetry.

The Android shell handles all platform I/O — HTTP, Room, ExoPlayer, audio, notifications — but the actual decision-making lives in **[fluxa-core](https://github.com/FluxaMedia/fluxa-core)**, a headless Rust library that runs the same logic across Android and desktop targets. Rust never touches the network directly; it emits typed effects that the Kotlin layer fulfills.

## Features

- **Catalogs & discovery** — home feed, genre/category grids, search across every installed addon, and a calendar of upcoming episodes for what you're following
- **Library** — watchlist, continue watching with resume position, and custom collections, with import support for existing lists
- **Watch tracking** — two-way sync with Trakt, MyAnimeList, and Simkl
- **Playback** — subtitle and audio track selection, intro/outro/recap skip, and direct torrent/magnet support
- **Profiles** — multiple local profiles on one install, each with its own library, addons, and sync accounts
- **Addons** — install and manage Stremio-compatible addons directly from the app
- **TV-ready** — a dedicated Android TV / Google TV interface, not a stretched phone layout
- **Auto-update** — checks for and installs new versions in-app

## Download

Grab the latest build from [Releases](https://github.com/KhooLy/Fluxa/releases/latest). Requires Android 8.0+ (API 26).

| Variant | Package | Target |
| --- | --- | --- |
| `mobile` | `com.fluxa.app.mobile` | Phone & tablet |
| `tv` | `com.fluxa.app.tv` | Android TV / Google TV |

Each variant ships per-ABI APKs — pick the one matching your device, or let the in-app updater do it for you:

| ABI | Devices |
| --- | --- |
| `arm64-v8a` | Most phones/tablets/TVs from the last ~6 years |
| `armeabi-v7a` | Older 32-bit ARM devices |
| `x86` | Intel-based Android (emulators, some set-top boxes) |

## Building from source

```bash
git clone https://github.com/KhooLy/Fluxa.git
cd Fluxa
./gradlew :app:assembleMobileDebug
# or for TV
./gradlew :app:assembleTvDebug
```

**Prerequisites**

- JDK 17+
- Rust stable with the Android NDK targets (`aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`)
- Android NDK (version pinned in `app/build.gradle.kts`)
- [`fluxa-core`](https://github.com/FluxaMedia/fluxa-core) checked out as a sibling directory (`../fluxa-core`) — it also provides `fluxa-streaming-engine` (`../fluxa-streaming-engine`)

The Rust libraries (`fluxa_core`, `fluxa_streaming_engine`) are cross-compiled for all Android ABIs automatically as part of the Gradle build; no manual `cargo build` step is needed.

## Architecture

Fluxa is split into two native Rust libraries and an Android Kotlin shell:

```
┌─────────────────────────────────────────────────────────┐
│                  Android (Kotlin + Compose)              │
│   UI · ViewModel · Repository · OkHttp · Room · Player  │
├───────────────────────────┬─────────────────────────────┤
│      fluxa_core           │    fluxa_streaming_engine    │
│  Headless brain: state,   │  Video proxy, Dolby Vision   │
│  policy, stream planning  │  rewrite, torrent engine     │
└───────────────────────────┴─────────────────────────────┘
```

Rust never calls the network directly. Instead it emits typed effects that Kotlin executes:

```
Kotlin  →  dispatch(action)
        ←  { state, effects: [{ id, type, payload }] }
Kotlin  →  executes each effect (OkHttp / Room / audio / ...)
        →  completeEffect({ effectId, result })
        ←  { state, effects: [...] }
```

This keeps `fluxa_core` fully portable — the same crate compiles for Android (JNI), desktop (native Rust), and future targets without any platform-specific code inside Rust.

## Stack

[Kotlin](https://kotlinlang.org/) · [Jetpack Compose](https://developer.android.com/compose) · [Rust](https://www.rust-lang.org/) · [AndroidX Media3 / ExoPlayer](https://developer.android.com/media/media3) · [MPV](https://mpv.io/) · [librqbit](https://github.com/ikatson/rqbit) · [UniFFI](https://mozilla.github.io/uniffi-rs/) · [Hilt](https://dagger.dev/hilt/) · [Room](https://developer.android.com/training/data-storage/room) · [OkHttp](https://square.github.io/okhttp/) + [Retrofit](https://square.github.io/retrofit/) · [Coil](https://coil-kt.github.io/coil/)

---

**Legal** — Fluxa is a client-side interface for user-installed Stremio addons. It does not host, serve, or distribute any media content. All streams come from third-party addons chosen by the user. Fluxa is not affiliated with any addon developer, repository, or content provider. Users are responsible for ensuring they have the right to access what they stream.

## Related projects

- [Fluxa Desktop](https://github.com/FluxaMedia/fluxa-desktop) — the desktop counterpart to this app
- [fluxa-core](https://github.com/FluxaMedia/fluxa-core) — the shared Rust library powering both

<!-- MARKDOWN LINKS -->
[contributors-shield]: https://img.shields.io/github/contributors/KhooLy/Fluxa.svg?style=for-the-badge
[contributors-url]: https://github.com/KhooLy/Fluxa/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/KhooLy/Fluxa.svg?style=for-the-badge
[forks-url]: https://github.com/KhooLy/Fluxa/network/members
[stars-shield]: https://img.shields.io/github/stars/KhooLy/Fluxa.svg?style=for-the-badge
[stars-url]: https://github.com/KhooLy/Fluxa/stargazers
[issues-shield]: https://img.shields.io/github/issues/KhooLy/Fluxa.svg?style=for-the-badge
[issues-url]: https://github.com/KhooLy/Fluxa/issues
[license-shield]: https://img.shields.io/github/license/KhooLy/Fluxa.svg?style=for-the-badge
[license-url]: https://github.com/KhooLy/Fluxa/blob/master/LICENSE
