# Effects reference

When the headless engine needs the platform to do something, it emits an effect. The platform matches on the `type` field, runs the appropriate I/O, and calls `completeEffect` with the result.

All types are defined in `src/runtime/effects.rs` (`EffectKind`). The string values below are what appears in the `type` field on the wire.

## Effect envelope

```json
{
  "id": "fx-7",
  "type": "fetchAddonResource",
  "generation": 4,
  "payload": { ... },
  "groupId": "addon",
  "priority": 50,
  "dedupeKey": "fetchAddonResource:4:<sha256>",
  "cachePolicy": "default",
  "timeoutMs": 15000
}
```

`id` is echoed back verbatim in `completeEffect`. `generation` is for staleness detection — the platform can discard completions whose generation is lower than the current engine generation.

Scheduling fields are optional and are present for effects with a policy. `dedupeKey` hashes the canonical payload; it never embeds payload contents. Hosts must not place effect payloads in persistent UI state.

## Effect types

### Fetch / network

| type | What to do |
|---|---|
| `fetchAddonManifest` | GET the addon manifest URL in `payload.url` |
| `fetchAddonResource` | GET the addon resource URL in `payload.url` |
| `fetchCatalogPage` | GET a catalog page (URL in `payload`) |
| `fetchDetailSecondary` | Fetch secondary metadata for the detail screen |
| `fetchDetailStreams` | Fetch streams from one or more addons |
| `fetchMetaDetail` | Fetch full metadata for a content item |
| `fetchMetaDetailLookup` | Fetch metadata by lookup (TMDB ID → IMDB ID, etc.) |
| `fetchSeasonEpisodes` | Fetch episode list for a season |
| `fetchSubtitles` | Fetch subtitle tracks |
| `fetchIntroSegments` | Fetch intro/outro skip segments from introdb.app or AniSkip |
| `prefetchDetailStreams` | Background prefetch of streams before user opens detail |
| `prefetchNextEpisodeStreams` | Prefetch streams for the next episode while current is playing |

### Playback

| type | What to do |
|---|---|
| `loadStreams` | Resolve and load stream list for the player |
| `prepareDirectPlayback` | Prepare direct playback (no stream resolution needed) |
| `startTorrentStream` | Start a torrent stream via fluxa-streaming-engine |
| `stopTorrent` | Stop and clean up an active torrent stream |
| `resolveIntroImdbId` | Resolve the IMDB ID needed for intro segment lookup |

### Storage / state

| type | What to do |
|---|---|
| `readHomeBootstrap` | Read local home bootstrap data (cached rows, continue-watching) |
| `readDetailLocalState` | Read locally cached state for a detail screen item |
| `readLibraryState` | Read the full library state from storage |
| `readPlaybackProgress` | Read saved playback position for an item |
| `readCalendarMonth` | Read calendar release data for a given month |
| `readDiscoverCatalogFilters` | Read saved discover/browse filter state |
| `writePlaybackProgress` | Persist the user's current playback position |
| `clearPlaybackProgress` | Remove a saved playback position |
| `writeFeedback` | Persist a like/dislike feedback signal |
| `writeLibraryCommand` | Execute a library mutation (add, remove, update) |
| `writeSettings` | Persist updated user settings |
| `updateCalendarWidget` | Refresh the OS home-screen calendar widget |

### Auth / sync

| type | What to do |
|---|---|
| `runAuthFlow` | Launch an OAuth flow (opens browser/WebView) |
| `exchangeAuthCode` | Exchange an auth code for tokens |
| `refreshAuthToken` | Refresh an expired auth token |
| `refreshInstalledAddons` | Re-fetch manifests for all installed addons |
| `runExternalSync` | Sync watched state with an external service (Trakt/Simkl) |
| `syncExternalIntegration` | Run a full bidirectional sync with an external integration |
| `syncWatchedState` | Push local watched state to an external service |
| `replaceExternalContinueWatching` | Replace the external continue-watching list |
| `enqueueTraktScrobble` | Queue a Trakt scrobble request |

### Content operations

| type | What to do |
|---|---|
| `runSearch` | Execute a search query across installed addons |
| `runDiscover` | Run a discover/browse query |
| `enqueueOfflineDownload` | Add an item to the offline download queue |
| `notifyReleasedEpisodes` | Push a notification for newly released episodes |

## completeEffect result shapes

Result payloads are effect-specific. The general pattern:

```json
{
  "effectId": "fx-7",
  "result": {
    "ok": true,
    "data": { ... }
  }
}
```

For network fetches, `data` typically contains the raw response body or a parsed object. For storage reads, it contains the stored value or `null` if nothing was saved. For write operations, `result` can be `{}` on success.

The exact shape for each effect type is consumed by the engine's `completeEffect` handler in `src/headless_engine/` — check the relevant submodule (e.g. `home.rs`, `detail.rs`, `player.rs`) for the `EffectResultInput` variant that handles it.
