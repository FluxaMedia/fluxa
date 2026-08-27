# Apple playback migration — remaining work

Last updated: 2026-08-27

## Current state

The Apple playback path now has a single `FluxaPlayer` surface backed only by
AVFoundation/AVPlayer. The compatibility policy is copy-first and delegates
container/stream adaptation to FFmpeg's native demuxer/muxer without
re-encoding supported video. The native/desktop route uses the FFmpeg process
adapter. The Apple route now has the linked static-library adapter and
incremental callback runtime-tested against local FFmpeg media on Linux, but
it is not device-complete until the C bridge is linked and exercised in Apple
CI. FFmpeg is not a
second player: there is no software video renderer or playback fallback in
Player Kit.

The latest completed Player Kit CI run is commit `70639d8` and passed after the
AVPlayer-only cleanup. The current Apple playback validation runs are tracking
the linked bridge and tvOS detail route on `e0d1e4f`. The local machine has no
Swift/Xcode toolchain, so Apple builds are validated in GitHub Actions.

## Already verified

- Rust core tests: `570 passed`, including the shared media policy and
  unsupported-audio coverage.
- Streaming-engine tests: `109 passed`, including the FFmpeg copy-first audio
  policy.
- FFmpeg bridge runtime fixture: strict `clang` syntax checks passed; real
  H.264/AAC MKV remux, keyframe seek, multi-audio language metadata, and
  unsupported Opus-to-ALAC adaptation all produced valid fMP4 inspected by
  `ffprobe`.
- Rust core XCFramework and Apple streaming bridge CI job: passed.
- The previous CI run passed the tvOS shared-code, XcodeGen, and tvOS host
  build steps.
- CI run `33000291475` completed with an iOS host compile failure at
  `FluxaApplePlaybackPresenter.swift:151`; the failure was the named-argument
  ordering described above. The tvOS host result was not affected by that
  compiler diagnostic, but the workflow was red overall.
- CI run `33024568322` passed for commit `b64c3dc`: macOS package build and
  tests, iOS host build, tvOS host build, and FFmpeg package build all passed.
- CI run `33025330485` passed for commit `71f1d99`: macOS package build and
  tests (including the WebVTT parser test), iOS host build, and tvOS host build
  all passed. The FFmpeg job was skipped because the workflow change filter did
  not select it.
- Player Kit CI run `33026060425` passed for commit `b5fc4b7`: macOS package
  build, Swift package tests, iOS host build, and tvOS host build all passed.

## Remaining work

### 1. Finish the Apple FFmpeg bridge

- Link and runtime-test the callback on iOS, tvOS, and macOS with the actual
  Apple-built FFmpeg slices; the Linux FFmpeg fixture is already passing.
- Fix remaining FFmpeg encoder/layout edge cases, then verify the bridge
  produces valid fragmented MP4 with stream-copied video and ALAC audio.
- Run the actual iOS, tvOS, and macOS static-link workflow. iOS/tvOS must not
  depend on an FFmpeg executable or process spawning.
- Keep demuxing, muxing, timestamps, fragmentation, and codec signaling inside
  FFmpeg; Swift/Rust should only pass requests, headers, and output chunks.

### 2. Exercise the real playback ladder

Validate on a real Apple device or simulator with representative media:

1. Native AVPlayer URL (MP4/HLS, H.264/HEVC, supported audio).
2. MKV/Matroska → incremental FFmpeg fMP4 remux → AVPlayer.
3. Unsupported audio (for example DTS/TrueHD) → selective lossless-compatible
   audio conversion while the video stream remains untouched → AVPlayer.
4. AVPlayer demux failure for a video format it cannot consume → clear
   non-recoverable failure; do not silently start a software video player.

Check startup, first-frame latency, seeking, resume position, pause/rate,
track selection, HTTP headers, subtitles, audio session interruptions, and
teardown.

### 3. Harden remux correctness

- Test fragmented MP4 output with real H.264, HEVC, and AAC Matroska samples.
- Verify timestamps, keyframe boundaries, edit/start positions, duration, and
  seeking after reopening with `start=`.
- Verify HDR/Dolby Vision signaling and relevant codec extradata survive the
  stream-copy path.
- Add/keep an explicit response path for sources that cannot be adapted to
  fMP4, so AVPlayer does not receive a misleading silent or partial stream.

### 4. Complete and verify audio compatibility

- Current policy: fMP4 adaptation copies streams that AVPlayer can consume and
  never silently drops unsupported audio. DTS, TrueHD, and other unsupported
  audio tracks are selectively decoded by FFmpeg and stored as lossless ALAC;
  object metadata such as TrueHD Atmos cannot survive that representation.
- Do not default to AAC: preserve the original audio bitstream whenever
  possible, and use a lossless-compatible representation when conversion is
  unavoidable.
- Ensure audio-only transcode keeps supported video bit-for-bit untouched.
- Verify language/default/forced track metadata and user track selection after
  adaptation.
- Verify supported E-AC-3/Atmos, AAC, AC-3, MP3, ALAC, and FLAC paths are not
  unnecessarily converted.

### 5. Cover torrent playback explicitly

- The Apple adapter now routes the torrent server's local HTTP stream through
  the same local proxy/remux path used by direct Matroska sources.
- AVPlayer receives the proxy's direct local HTTP URL rather than a magnet URI,
  and the existing playback item preserves resume position.
- Still validate on-device with MKV and non-MKV torrent files, including seek
  and remux rejection behavior.
- tvOS catalog cards now open a detail route that loads metadata, lists series
  episodes, resolves direct streams with request headers and external
  subtitles, offers all returned stream options, and enters the shared
  `FluxaPlayer` surface. Magnet and `.torrent` streams now use the tvOS torrent
  service plus the same local remux path. On-device validation remains.

### 6. Finish Apple integration verification

- Now Playing metadata (title, duration, position, rate, and playback state)
  is owned by `FluxaPlayerKit` and follows the shared player state.
- Common audio-session behavior is owned by `FluxaPlayerKit`.
- External WebVTT/SRT subtitles are parsed and rendered by the shared player
  overlay, and exposed as selectable player tracks; embedded AVPlayer subtitle
  track selection remains available.
- External subtitle requests reuse the playback HTTP headers.
- WebVTT/SRT parser coverage is included in the `FluxaPlayerKit` Swift package
  tests.
- Interruption handling now reactivates the audio session and resumes only when
  the system supplies the `shouldResume` option; on-device verification
  remains.
- The obsolete `FluxaAppleAudioSessionCoordinator.swift` was removed after
  all references were eliminated.
- Keep iOS and tvOS UI differences behind platform conditionals.
- Complete the tvOS detail route around `FluxaTvosPlaybackPresenter`, then
  verify headers, resume position, remux lifecycle, and dismissal cleanup.
- Verify interruption, route changes, PiP, external playback, HDR/Dolby Vision,
  subtitles, teardown, and resume on real hardware where simulator behavior is
  insufficient.

### 7. Desktop/Tauri macOS AVPlayer integration

- The macOS desktop shell now has a native `FluxaPlayerKit` bridge selected by
  the `AVPlayer` player-engine setting. It hosts `AVPlayerLayer` in a native
  `NSView` behind the existing React overlay.
- Existing mpv/libVLC surfaces remain separate and unchanged; selecting either
  engine does not load the AVPlayer bridge.
- The bridge forwards load, play/pause, seek, rate, volume, track selection,
  external subtitles, status polling, and teardown through a C ABI handle.
- Remaining validation is on a macOS build: universal-arch Swift linking,
  native surface placement, HDR/Dolby Vision, track/subtitle behavior, and
  error/teardown behavior with representative media.

## Validation commands

Run the smallest relevant offline checks first:

```bash
npm run check:structure
npm run check:core
cargo test --manifest-path core/fluxa-core/fluxa-streaming-engine/Cargo.toml --offline
```

Apple package and host builds must continue to run in GitHub Actions on the
available macOS runners. The Player Kit workflow also runs `swift test` for
the subtitle parser. A green compile does not replace real playback tests with
representative media.

## Scope guardrails

- No PR workflow is required; changes go directly to `master` when explicitly
  ready.
- Accumulate related fixes and make one meaningful commit/push per milestone.
- Preserve unrelated pre-existing dirty files.
- Do not remove the existing desktop mpv/libvlc paths.
