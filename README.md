<div align="center">

<img src="src-tauri/icons/icon.png" alt="Fluxa" width="96" />

# Fluxa

A cross-platform media client for Windows, macOS, Linux, the web, and webOS.<br/>
Browse catalogs, track what you watch, and play anything the Stremio addon ecosystem exposes.

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stars][stars-shield]][stars-url]
[![Releases][releases-shield]][releases-url]
[![License][license-shield]][license-url]
[![Discord][discord-shield]][discord-url]

[Download](#download) · [Features](#features) · [Building from source](#building-from-source) · [Discord][discord-url]

</div>

---

## What it does

Fluxa connects to any Stremio-compatible addon and provides the same core experience across native desktop, browser, and webOS builds: a home feed with genre and category browsing, a calendar of upcoming episodes, a library with continue-watching and resume positions, and two-way watch tracking with Trakt, MyAnimeList, and Simkl.

The native desktop build uses Tauri and `libmpv`. The web and webOS builds use browser playback and the WebAssembly build of [`fluxa-core`](https://github.com/FluxaMedia/fluxa-core), with platform-specific capabilities handled at the frontend boundary.

## Features

- **Catalogs & discovery** — home feed, genre/category grids, search across every installed addon, and a calendar of upcoming episodes for what you're following
- **Library** — watchlist, continue watching with resume position, and custom collections, with import support for existing lists
- **Watch tracking** — two-way sync with Trakt, MyAnimeList, and Simkl
- **Playback** — subtitle and audio track selection, intro/outro/recap skip, and direct torrent/magnet support
- **Profiles** — multiple local profiles on one install, each with its own library, addons, and sync accounts
- **Addons** — install and manage Stremio-compatible addons directly from the app
- **Auto-update** — checks for and installs new versions in-app
- **Watch Together** — synchronize playback through the standalone WebSocket server or directly through Supabase Realtime Broadcast and Presence

## Download

Grab the latest build from [Releases](https://github.com/FluxaMedia/fluxa-desktop/releases/latest).

| Platform | Package |
| --- | --- |
| Windows 10+ | `.exe` — NSIS installer |
| macOS 11+ | `.dmg` — Universal (Intel + Apple Silicon) |
| Linux (Debian / Ubuntu) | `.deb` |
| Linux (Fedora / RHEL) | `.rpm` |
| Linux (portable) | `.AppImage` |

### Web

The browser build is published to [Fluxa Pages](https://fluxamedia.github.io/fluxa-desktop/). It can also be built locally with `npm run build:web`.

### webOS

The webOS build targets the Chromium version available on supported LG webOS devices and is emitted to `dist-webos`. Build it locally with `npm run build:webos`, then package or serve that directory using your webOS deployment workflow.

Web and webOS builds do not include Tauri or native `libmpv`. Features that require native desktop integration, such as native window controls or filesystem integration, are therefore unavailable on those targets.

When the desktop application is running, it also starts a loopback companion service on `127.0.0.1:19876`. The web build can use this service for desktop-assisted torrent streaming, HTTP proxying, FFmpeg probing/transcoding, and OAuth operations. The service is bound to loopback and only allows configured web origins; set `FLUXA_WEB_ORIGIN` when serving the web client from another origin.

### Watch Together transports

The web player supports two Watch Together transports:

- **WebSocket** — connect to the public [`watch-together-server`](https://github.com/KhooLy/watch-together-server) or another compatible deployment.
- **Supabase Realtime** — enter a Supabase project URL and public anon key when prompted. Fluxa uses Realtime Broadcast for playback messages and Presence for room members; no Supabase service-role key is ever required.

Supabase rooms use the room code as the capability to join. Configure Realtime access policies in the Supabase project if rooms need private access controls.

## Building from source

```bash
git clone https://github.com/FluxaMedia/fluxa-desktop.git
cd fluxa-desktop
npm install
npm run tauri dev
```

**Prerequisites**

- Node.js 22+
- Rust stable
- [`fluxa-core`](https://github.com/FluxaMedia/fluxa-core) checked out as a sibling directory (`../fluxa-core`) — it also provides `fluxa-streaming-engine`
- `libmpv` — either install it system-wide, or run `./src-tauri/fetch-libmpv.sh` to pull the prebuilt gpu-next fork used for release builds
  - Linux: `sudo apt install libmpv-dev` or `sudo pacman -S mpv`
  - macOS: `brew install mpv`
  - Windows: handled by the fetch script

```bash
npm run build   # production frontend build
npm run check   # typecheck + cargo check
npm run build:web       # browser build in dist-web
npm run build:web:pages # browser build with the GitHub Pages base path
npm run build:webos     # webOS build in dist-webos
```

Web builds require the sibling `fluxa-core` checkout and its WASM package. The GitHub Pages workflow builds that package automatically; for a local web build, follow the WASM build instructions in [`fluxa-core`](https://github.com/FluxaMedia/fluxa-core) first.

## Stack

[Tauri](https://tauri.app/) · [React](https://react.dev/) · [TypeScript](https://www.typescriptlang.org/) · [Vite](https://vitejs.dev/) · [Rust](https://www.rust-lang.org/) · [mpv](https://mpv.io/) · [librqbit](https://github.com/ikatson/rqbit)

---

**Legal** — Fluxa Desktop is a client-side interface for user-installed Stremio addons. It does not host, serve, or distribute any media content. All streams come from third-party addons chosen by the user. Fluxa is not affiliated with any addon developer, repository, or content provider. Users are responsible for ensuring they have the right to access what they stream.

## Community

Questions, bug reports, feature requests, or just want to hang out — join the [Fluxa Discord][discord-url].

## Related projects

- [Fluxa for Android](https://github.com/KhooLy/Fluxa) — the Android counterpart to this app
- [fluxa-core](https://github.com/FluxaMedia/fluxa-core) — the shared Rust library powering the desktop, web, and webOS targets

<!-- MARKDOWN LINKS -->
[contributors-shield]: https://img.shields.io/github/contributors/FluxaMedia/fluxa-desktop.svg?style=for-the-badge
[contributors-url]: https://github.com/FluxaMedia/fluxa-desktop/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/FluxaMedia/fluxa-desktop.svg?style=for-the-badge
[forks-url]: https://github.com/FluxaMedia/fluxa-desktop/network/members
[stars-shield]: https://img.shields.io/github/stars/FluxaMedia/fluxa-desktop.svg?style=for-the-badge
[stars-url]: https://github.com/FluxaMedia/fluxa-desktop/stargazers
[releases-shield]: https://img.shields.io/github/v/release/FluxaMedia/fluxa-desktop.svg?style=for-the-badge
[releases-url]: https://github.com/FluxaMedia/fluxa-desktop/releases/latest
[license-shield]: https://img.shields.io/github/license/FluxaMedia/fluxa-desktop.svg?style=for-the-badge
[license-url]: https://github.com/FluxaMedia/fluxa-desktop/blob/master/LICENSE
[discord-shield]: https://img.shields.io/badge/Discord-Join-5865F2.svg?style=for-the-badge&logo=discord&logoColor=white
[discord-url]: https://discord.gg/wan9FeDEfe
