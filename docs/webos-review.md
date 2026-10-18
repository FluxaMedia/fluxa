# webOS native review

Three working webOS implementations compared with `native/fluxa-webos`.

| Reference | What it is | How it plays video | License | Evidence |
| --- | --- | --- | --- | --- |
| [nuvio-native-legacy](https://github.com/iqui27/nuvio-native-legacy) 2.0.7 | C99, SDL2, GLES2 | TV media service `com.webos.media` with a URL (the TV fetches and demuxes). Starfish buffer feed only for DTS and Dolby Vision in MKV, through a C++ adapter | GPL-3.0 | One owned TV (C9, webOS 4), user reports for 3.x and 5+, logs uploaded by users |
| Kodi webOS stack | C++, Wayland | Starfish BUFFERSTREAM, same as Fluxa | GPL-2.0-or-later | Years of users on webOS 5+ |
| [plx-native](https://github.com/GLinnik21/plx-native) | Rust UI, C Starfish layer | Starfish BUFFERSTREAM, same as Fluxa | GPL-3.0-or-later | Measured on webOS 3.9, 4.10 and 4.5 TVs, firmware symbol tables for 14 firmwares |

Fluxa is the only one of the three with no TV run behind it. Everything marked "verify on TV" below is still a guess.

Ground truth used where the references disagree: the 14 firmware dumps of `libplayerAPIs.so.1` in the webosbrew compat-checker data (W15 to W23, webOS 3.x to 8). Every mangled name in `ffi.rs` was checked against them.

## 1. Bugs found in Fluxa

Each is verified against the firmware dumps or a reference's measured behaviour. Items 1 to 3 stop playback from starting at all.

| # | Finding | Evidence | Fix |
| --- | --- | --- | --- |
| 1 | `setTimeToDecode` is looked up as `…13setTimeToDecodeEPKc`. The name is 15 characters, so it is `…15setTimeToDecodeEPKc`. The symbol is required, so `Starfish::open` fails with a missing symbol on every TV | `13` form absent in all 14 dumps, `15` form present in all TV dumps | Correct the name in `ffi.rs`, in both the bind and the probe list |
| 2 | `setHdrInfo` is required, but it only exists from webOS 5 (W20L and newer). On webOS 3 and 4 `Starfish::open` fails | Absent in W15M, W16N, W17H, W18R, W19P; plx `webos5-port.md` says the same | Make it optional. On webOS < 5 do not call it |
| 3 | `Feed` is bound only as the `cxx11` variant. webOS 3.x (W16N, W17H) only has the plain `Feed`, which returns the old copy-on-write `std::string` (a single pointer, not 32 bytes) | Dumps: `FeedB5cxx11` missing there, `Feed` present. Nuvio ships a separate adapter for the old ABI for the same reason | Bind whichever exists. The old variant needs its own return layout, so webOS 3 stays unsupported until that is done and tested |
| 4 | `changeAudioCodec` is bound with the suffix `…EES8_`. The real name ends `…EES7_` | Present with `S7_` on W21P and newer | Correct the suffix. It is already optional |
| 5 | On webOS 4 HDR10 is delivered by merging `hdrType`, `mediaSei` and `mediaVui` into the ACB `setMediaVideoData` payload. We forward the `videoInfo` event text unchanged and call the absent `setHdrInfo` instead, so HDR would never switch on | plx `webos5-port.md`, `kodi-parity-42-plan.md` (probed on 4.5), Nuvio comments (C9 stayed SDR until the payload was patched) | In `StarfishPipeline::on_event`, build the ACB payload from the event plus `hdr_payload` on webOS < 5 |
| 6 | The Back hint `SDL_WEBOS_ACCESS_POLICY_KEYS_BACK` is never set. Without it LG's compositor keeps the Back key and opens the app bar | Nuvio `main.c` ("read only at window creation"), plx `main.c:189`; our grep finds no hint | Set the environment variable before `SDL_Init`. `SDL_WEBOS_ACCESS_POLICY_KEYS_EXIT` is the only other hint in the SDK |
| 7 | `libplayerAPIs` is `dlclose`d when the pipeline drops. Nuvio saw SIGSEGV on the second playback on webOS 3 (65SJ800V, OLED55B7P) and fixed it with `RTLD_NODELETE` | Nuvio `starfish.cpp` comment | Keep the library loaded for the life of the process |

## 2. Review by area

Verdict: adopt, adapt, skip, verify on TV.

| Area | Nuvio | Kodi | plx | Fluxa today | Verdict |
| --- | --- | --- | --- | --- | --- |
| Starfish symbol binding | Probes at run time, missing symbols disable the feature, never crash | Links against headers | Weak symbols, missing ones are no-ops | Required symbols, wrong names (section 1) | Adopt: optional everything beyond the core set, tested against the dumps |
| Load race | n/a | n/a | Never drive Starfish while `Load` is in flight (issue 74, fixed in `6f3486d0`) | Load and later calls run on one thread, not checked | Verify: confirm no verb can run during `Load` |
| Realtek k5lp/k3lp jail | n/a | n/a | Developer Mode jail lacks `/dev/rtkmem` there, so `Load` hangs. They probe and refuse to play | Not handled | Adapt: add the probe and a clear error |
| Seek | `flush({"ptsToDecode":ns})`, then Play or Pause again, no second LOADCOMPLETED. Seeks in the DTS path recreate the pipeline | `flush()`, `setTimeToDecode`, `sendSegmentEvent` | `flush()` plus `sendSegmentEvent` through a private symbol, or public `Seek(ms)` | `flush()` plus `setTimeToDecode` (broken name), no segment event | Verify on TV. Adopt first the public `flush(const char*)` with `ptsToDecode`, exported on every TV dump. Needs no C++ shim |
| Feed back-pressure | Keeps the buffer until the reply, handles `Ok`, `BufferFull`, `Pending` | Per-stream retry, 10 ms | Two lanes (video about 8 MB, audio about 1 MB), feeder threads at 10 ms | One interleaved loop | Adapt later: measure on a TV first |
| End of stream and unload | `pushEOS`, waits for unload | same | `pushEOS` at EOF, waits for `UNLOADCOMPLETED` before destruct | `pushEOS` and `Unload`, no wait | Adapt |
| Load payload | n/a (URI mode) | Full `esInfo` | Real width, height and fps rational, `ac3PlusInfo`, `aacInfo`, `adaptiveResolution` | Has width, height, fps, adaptive, per-codec info | Done |
| Video plane webOS 4 | ACB with `connectDass` in URI mode | n/a | ACB without `connectDass` in buffer-feed mode, measured | Same as plx | Skip `connectDass`. It belongs to URI mode |
| Video plane webOS 5+ | SDL exported window, id in `windowId` | same | same | same | Done |
| ACB details | Callback must be non-NULL, `setMediaAudioData` optional, Atmos through it | n/a | Atmos through `setMediaAudioData` with the same media id | No audio data call | Adapt later (Atmos label only) |
| HDR10 / HLG | Patches `hdrType` and VUI into the ACB payload, re-sends as SDR if DV fails | `setHdrInfo` on webOS 5+, skips when SEI is empty | In-band SEI plus ACB payload on webOS 4 | `setHdrInfo` only (section 1, item 5) | Fix |
| Dolby Vision | `DolbyHdrInfo` in URI mode left off by default (blank picture on many files). Own demux path for MKV, profile 7 refused | Profile 7 converted to 8 | RPU rewrite to 8.1, P5 and P8 direct, DV declared false on webOS 3 | Same as plx | Done. Verify on TV |
| Audio | DTS converted to AAC when the TV refuses it | DTS only if EDID says so, E-AC3 when eARC reports DD+ | Same family | Done in the Kodi pass | Done. Verify on TV |
| Seek failure policy | Three refused seeks, then keep playing | n/a | n/a | One attempt | Adapt: small |
| Subtitles | libass for ASS from MKV, read over HTTP ranges; PGS/VobSub hidden; plx decodes them with FFmpeg | n/a | FFmpeg decode of PGS/VobSub/DVB, side reader | Text and libass handled in the core, image subtitles not | Later, own plan |
| Back key | Hint before window creation, scancode 482, only key-down counts | Wayland key handling | Hint in `main.c`, Back at the root asks SAM to go Home (`go_home`) | Not set (section 1, item 6) | Fix. Root-Back behaviour: adapt later |
| Colour keys | Scancodes 486 to 489, unknown scancodes logged once | n/a | Measured 486 red, 487 green, 488 yellow, 489 blue; standard evdev codes are wrong | Unbound | Adapt when something needs them |
| Key repeat | Press timing handled in the app | webOS 7+ repeats at 40 cps after 400 ms, breaks long-press | Auto-repeat flag in `state & 0x100` | Flag decoded, not used | Adapt: time long-press ourselves |
| Screensaver | `registerScreenSaverRequest`, ack:false with the original timestamp token | same | n/a | Done, with the same token | Done |
| Lifecycle | Focus lost cancels armed actions; minimise then 120 s releases the player; restore resumes | Relaunch event, minimised means stop | Lifecycle matrix of 16 cases | Relaunch raises the window, minimise stops | Adapt: grace period before stopping, lifecycle matrix |
| Memory | Playback floor scaled by `MemTotal`, texture cache variants (normal, 300 MB) | `requiredMemory` 500 | n/a | `requiredMemory` 500 | Verify: measure on TV |
| Diagnostics | Crash marker file, send-log card, Diagnostic that tunes settings | n/a | Event log, firmware context on every error | Sentry, symbol inventory | Adopt the crash marker and send-log card |
| Packaging | Executable-bit check on the binary (issue 225), two ipks, Homebrew Channel manifest with sha256 | n/a | CI allow lists, `webosbrew.manifest.json` | `verify-webos.sh` | Adapt: executable-bit check, manifest |
| UI render budget | Home at 1080p on a 4K panel, upload budget per frame | n/a | Lookahead of posters, per-class upload budget, warm textures | Not reviewed | Later, own plan |

## 3. Kodi, read in full

Files read: `MediaPipelineWebOS.cpp/.h`, `VideoPlayer*WebOS`, `RendererStarfish`, `VideoBufferStarfish`, `WebOSTVPlatformConfig`, `WinSystemWaylandWebOS`, `ShellSurfaceWebOSShell`, `SeatWebOS`, `OSScreenSaverWebOS`, `LunaPowerManagement`, `settings/webos.xml`, `README.webOS.md`.

Pipeline life cycle (the part Kodi has had years to get right):

1. `Load` with callback. On `LOADCOMPLETED`: ACB `setSinkType(MAIN)`, `setMediaId`, `setState(FOREGROUND, LOADED)`, then `Play()`, mark the first video packet as "flushed", start the video and audio threads. Nothing is fed before `LOADCOMPLETED`.
2. `PAUSED`, `PLAYING` and `UNLOADCOMPLETED` are mirrored into ACB `setState`. Fluxa does the same.
3. `UNLOADCOMPLETED` stops and joins both feed threads before the pipeline is freed. Fluxa unloads without waiting.
4. `AUDIO_INFO` and `VIDEO_INFO` events are forwarded to ACB `setMediaAudioData` and `setMediaVideoData` (the payload text unchanged). Fluxa forwards the video one only.
5. Feed rule: after a flush the first packet triggers `setTimeToDecode {position}` and `sendSegmentEvent`; if `setTimeToDecode` fails on webOS < 11 it patches `ptsToDecode` through `CustomPipeline::setContentInfo`. `FRAMEREADY` sets "started".
6. Feed throttle: stop feeding once the fed PTS is 1600 ms ahead of the played PTS. `Ok` advances, `BufferFull` retries the same packet after 10 ms, anything else is logged and the packet is dropped. Video PTS is shifted back by the renderer delay.
7. End of stream: `VIDEO_DRAIN` message becomes `pushEOS`.
8. Dropped frames are counted from `DROPPED_FRAME` events; `INT_ERROR` and `STR_ERROR` are only logged (no recovery path).

Window and input (Wayland, so only the lessons carry over to SDL):

| Kodi does | Fluxa (SDL) |
| --- | --- |
| Shell surface properties `appId`, `displayAffinity` (`DISPLAY_ID`), `_WEBOS_ACCESS_POLICY_KEYS_BACK` and `_GUIDE` set to `true` | Back set through the SDL hint (done). Guide has no SDL hint in the SDK. Verify whether SDL exposes the other two |
| `registerApp`, `relaunch` event raises the window to fullscreen | Done |
| Minimised (state bits empty) is intercepted and playback stopped, the state is not passed on because it shows an odd animation | Stops playback. Verify the window is not resized by the same event |
| `set_cursor` on the seat is a no-op because it breaks pointer input | SDL cursor is never set. Verify we never call `SDL_ShowCursor` or a custom cursor |
| Seat key-repeat info ignored (webOS 7 sends 40 cps after 400 ms) | Not handled (see key repeat row in section 2) |
| Reports a 1080p GUI on 4K and 720p on HD, video limit set separately to 4K or 1080p | Same rule |
| HDR display flag from `tv.model.supportHDR` | Config value read for DV only. Verify HDR flag source |
| Screen saver subscribed only while playback inhibits it | Done |

Not read: the generic Wayland classes (registry, connection, xkb keymap, window decorator, output, presentation sync). They are Kodi's own windowing stack and Fluxa uses SDL, so they have no counterpart. The audio engine hand-over (`RequestAudioDevice`) was read in the pipeline file only.

## 4. Combined comparison

Where the three references agree, or the dumps settle it, the decision is made. Where they disagree the decision waits for a TV.

| Topic | Kodi | Nuvio | plx | Decision |
| --- | --- | --- | --- | --- |
| Feed starts only after `LOADCOMPLETED` | yes | n/a | yes, and no verb while `Load` is in flight | Agreed. Check Fluxa for any verb that can run before `LOADCOMPLETED` |
| Seek | `flush()`, `setTimeToDecode`, `sendSegmentEvent` | `flush({"ptsToDecode"})`, replay Play or Pause | `Seek(ms)` or `sendSegmentEvent` | Disagree. Try `flush(const char*)` first, fall back to Kodi's order. TV needed |
| Throttle | 1600 ms ahead for audio and video, retry after 10 ms | buffer kept until reply | two lanes, byte caps | Adopt Kodi's time cap, per lane. The byte caps from plx as a second guard |
| Unload | joins feed threads on `UNLOADCOMPLETED` | waits for unload | waits for `UNLOADCOMPLETED` before destruct | Agreed. Fluxa must wait |
| ACB data events | forwards audio and video info | video, audio optional | both | Forward audio info too, only when `setMediaAudioData` exists |
| HDR | `setHdrInfo`, skips empty SEI | ACB payload patch | ACB payload on webOS 4 | Done |
| Error events | logged only | fallback paths | event log with firmware context | Keep logging, add the firmware context line from plx |
| Lifecycle | relaunch raises, minimise stops | grace 120 s | matrix of cases | Needs product decision on the grace period |
| Back / Guide | surface properties | hint | hint, root Back goes Home | Done for Back. Root Back behaviour is plx's |
| Codec gating | `Supports()` falls back to software decode | URI mode lets TV decide | probe at start | Fluxa has no software decoder. Keep the up-front rejection, report the reason |

## 5. Order from here

Done and committed: items 1 to 7 of section 1.

Seek correction after reading plx `docs/webos5-port.md` and `src/starfish.c`: plx decompiled the 4.5 libraries and found that `flush()` is itself a seek to 0, so `flush()` plus `setTimeToDecode` without a segment event stalls the sink after about 48 s. The public `Seek(ms)` flushes and re-anchors the segment itself, exists on every firmware 1.2 to 11.2 and is what LG's own Chromium MSE player uses in this mode. Fluxa now calls `Seek` on the first video packet after the demuxer seek (millisecond floor of that packet's PTS, audio before it dropped) and falls back to `flush()` plus `setTimeToDecode` only if `Seek` refuses. `flush(const char*)` is no longer used: nothing shows it re-anchors the segment. Kodi's `sendSegmentEvent` and plx's webOS 4 private offsets are not used because the layout changes from webOS 6.

Also from plx: webOS 3.x `Feed` (copy-on-write string) is now bound, with the reply released through `_ZNSsD1Ev`, and the Starfish object buffer is 64 KB like plx (the real size is unknown).

Implemented since (all unverified on a TV): seek as above; unload waits up to 1 s for `UNLOADCOMPLETED`; a packet the TV rejects is dropped; three refused demuxer seeks before playback fails; `Load` refused on k5lp/k3lp without `/dev/rtkmem`; 15 s load timeout; audio info to ACB; pause on minimise and stop after 120 s; colour keys 486 to 489 mapped (no action bound); held arrows throttled to 10 per second and other held keys ignored; the packaging script checks the executable bit and the appinfo `main`.

Original list, kept for reference:

1. Seek: `flush(const char*)` with `ptsToDecode`, keep `setTimeToDecode` as the second step, behind one function so the TV result decides which stays.
2. Pipeline life cycle: no feed before `LOADCOMPLETED`, wait for `UNLOADCOMPLETED` before free, per-lane time throttle, forward audio info to ACB.
3. Failure policy: three refused seeks then keep playing, `Load` timeout with a clear error, the Realtek probe.
4. TV run: build the ipk, run `probe`, play H.264/AC3, HEVC HDR10, DV profile 8 and 5, DTS, seek, audio switch, pause, Back. Nothing past step 3 is tuned until this has happened.
5. After the TV run: lifecycle grace period, key repeat, colour keys, crash marker, packaging checks, subtitles, old-ABI `Feed` for webOS 3.

A C++ shim stays an option, not a need: Nuvio ships one for DTS because it needs the old string ABI, and Kodi's `sendSegmentEvent` is a private C++ call. Neither is needed until a TV shows the public calls are not enough.

## 6. Not reviewed

- plx-native: the 345 KB `parity-gaps.md`, `adaptive-playback.md`, distribution and Store submission notes, the Rust UI.
- Nuvio: the non-webOS ports, the server code, the Android and Tizen sources.
- Kodi: the generic Wayland windowing classes and the audio engine, see section 3.
