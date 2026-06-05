<div align="center">

# Fluxa

  [![Contributors][contributors-shield]][contributors-url]
  [![Forks][forks-shield]][forks-url]
  [![Stars][stars-shield]][stars-url]
  [![Issues][issues-shield]][issues-url]
  [![License][license-shield]][license-url]

  <p>
    A native Android media hub powered by a platform-agnostic Rust core.<br/>
    Stremio addon ecosystem · Mobile & Android TV
  </p>

</div>

---

## What is Fluxa?

Fluxa is a content discovery and playback app for Android. It connects to the Stremio addon ecosystem, letting you browse catalogs, track your watch history, and play media from any source those addons expose.

The Android shell handles all platform I/O — HTTP, Room, ExoPlayer, audio, notifications — but the actual decision-making lives in **[fluxa-core](https://github.com/KhooLy/fluxa-core)**, a headless Rust library that runs the same logic across Android and desktop targets. Rust never touches the network directly; it emits typed effects that the Kotlin layer fulfills.

**Requires Android 8.0+ (API 26). Android TV / Google TV supported.**

---

## Installation

Download the latest release from [GitHub Releases](https://github.com/KhooLy/Fluxa/releases/latest).

Two APK variants are available:

| Variant | Package | Target |
|---------|---------|--------|
| `mobile` | `com.fluxa.app.mobile` | Phone & tablet |
| `tv` | `com.fluxa.app.tv` | Android TV / Google TV |

Each variant ships per-ABI splits: `arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`.

---

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

### The Effect Loop

Rust never calls the network. Instead it emits typed effects that Kotlin executes:

```
Kotlin  →  dispatch(action)
        ←  { state, effects: [{ id, type, payload }] }
Kotlin  →  executes each effect (OkHttp / Room / audio / ...)
        →  completeEffect({ effectId, result })
        ←  { state, effects: [...] }
```

This keeps `fluxa_core` fully portable — the same crate compiles for Android (JNI), desktop (native Rust), and future targets (WASM) without any platform-specific code inside Rust.

---

## Project Structure

```
Fluxa/
├── app/          ← Android application module
├── core/         ← Shared Kotlin modules
├── player/       ← Player module
└── build-logic/  ← Convention plugins
```

---

## Development

```bash
git clone https://github.com/KhooLy/Fluxa.git
cd Fluxa
./gradlew :app:assembleMobileDebug
# or for TV
./gradlew :app:assembleTvDebug
```

The Rust libraries ship as prebuilt `.so` files bundled with the project. Rebuilding `fluxa_core` or `fluxa_streaming_engine` from source requires a Rust toolchain with the Android NDK targets installed.

---

## Built With

- [Kotlin](https://kotlinlang.org/) + [Jetpack Compose](https://developer.android.com/compose)
- [Rust](https://www.rust-lang.org/) — headless core and streaming engine
- [AndroidX Media3 / ExoPlayer](https://developer.android.com/media/media3)
- [MPV](https://mpv.io/) — advanced format and subtitle support
- [librqbit](https://github.com/ikatson/rqbit) — BitTorrent engine
- [UniFFI](https://mozilla.github.io/uniffi-rs/) — Rust ↔ Kotlin FFI
- [Hilt](https://dagger.dev/hilt/) — dependency injection
- [Room](https://developer.android.com/training/data-storage/room) — local database
- [OkHttp](https://square.github.io/okhttp/) + [Retrofit](https://square.github.io/retrofit/)
- [Coil](https://coil-kt.github.io/coil/) — image loading

---

## Legal & DMCA

Fluxa is a client-side interface that connects to user-installed Stremio addons. It does not host, serve, or distribute any media content. All streams are sourced from third-party addons chosen by the user.

Fluxa is not affiliated with any addon developer, repository, or content provider. Users are responsible for ensuring they have the right to access any content they stream or download.

---

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
