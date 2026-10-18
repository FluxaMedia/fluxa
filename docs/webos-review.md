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

## 3. Not reviewed

- Kodi: the base Wayland window classes, renderer synchronisation, audio engine. Only the media pipeline, platform config, window class and README were read.
- plx-native: the 345 KB `parity-gaps.md`, `adaptive-playback.md`, distribution and Store submission notes, the Rust UI.
- Nuvio: the non-webOS ports, the server code, the Android and Tizen sources.

## 4. Proposed order

1. Fix items 1 to 4 and 7 in `ffi.rs` (symbol names, optional `setHdrInfo`, library lifetime). Add a test that checks every bound name against a stored list of the dump symbols.
2. Set the Back hint (item 6).
3. HDR on webOS 4 through the ACB payload (item 5).
4. Seek through `flush(const char*)` with `ptsToDecode`, still unverified on TV. This answers the `sendSegmentEvent` question without a C++ shim.
5. Take it to a TV: build the ipk, run `probe`, play H.264/AC3, HEVC HDR10, DV profile 8 and 5, DTS, seek, audio switch, pause, Back.
6. Only after a TV run: feed lanes, lifecycle grace period, the Realtek probe, subtitles.

A C++ shim stays an option, not a need: Nuvio ships one for DTS because it needs the old string ABI, and the same may be needed for webOS 3.
