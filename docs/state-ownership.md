# State ownership

Fluxa desktop is a presentation and platform-integration shell. `fluxa-core` owns application decisions and durable application state. The desktop owns native runtime resources. React owns transient UI state.

| Concern | Owner | Other layers may do |
| --- | --- | --- |
| Catalog, search, library, watched decisions, playback policy and effect scheduling | `fluxa-core` | Render snapshots and complete effects |
| mpv/libVLC handles, native surfaces, torrent server handles, download workers and OS sleep inhibition | Rust desktop | Expose narrow Tauri commands and events |
| Routes, dialogs, focus, layout, overlays and local input state | React desktop | Dispatch intents to core or invoke native commands |
| Playback position and decoder telemetry | Native player | Emit telemetry; React must not decide progress or watched state |
| Profile-scoped library persistence | `fluxa-core` and profile storage | React must reset its snapshot before activating another profile |

Only one layer may make a decision for a concept. React can request playback but cannot mark an item watched. Rust can report mpv position but cannot select a next episode. Core can select an effect but cannot retain native player handles.

Desktop runtime state is grouped by domain. `player_overlay` is the atomic snapshot used by native player surfaces and overlay commands. `torrent` is the atomic identity of the active torrent server and stream. Commands must copy a domain snapshot before awaiting or doing I/O.
