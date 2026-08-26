# Apple playback migration — remaining work

Last updated: 2026-08-27

## Current state

The Apple playback path now has a single `FluxaPlayer` surface with an
AVFoundation-first backend. MKV/Matroska sources can be adapted to rolling
fragmented MP4 in the Rust streaming layer without re-encoding supported video.
The Player Kit also contains a compile-gated FFmpeg/VideoToolbox fallback for
sources that AVPlayer cannot demux. Unsupported software video decoding is not
used as a normal path.

The latest Apple CI run was commit `8021ba7`. Its iOS host build failed because
the playback-item call placed `fallbackURL` before `startPosition`; that call is
now corrected locally. The local machine has no Swift/Xcode toolchain, so Apple
builds are validated in GitHub Actions.

## Already verified

- Rust core tests: `566 passed`.
- Streaming-engine tests: `108 passed`.
- FFmpeg fallback artifact CI job: passed.
- Rust core XCFramework and Apple streaming bridge CI job: passed.
- The previous CI run passed the tvOS shared-code, XcodeGen, and tvOS host
  build steps.
- CI run `33000291475` completed with an iOS host compile failure at
  `FluxaApplePlaybackPresenter.swift:151`; the failure was the named-argument
  ordering described above. The tvOS host result was not affected by that
  compiler diagnostic, but the workflow was red overall.

## Remaining work

### 1. Finish the next Apple CI run

- Confirm the iOS host build passes after the playback-item argument ordering
  fix.
- Confirm the current tvOS host build remains green.
- If another compiler error appears, collect related fixes and commit them as
  one grouped change; do not commit every small correction separately.

### 2. Exercise the real playback fallback chain

Validate on a real Apple device or simulator with representative media:

1. Native AVPlayer URL (MP4/HLS, H.264/HEVC, supported audio).
2. MKV/Matroska → rolling fMP4 remux → AVPlayer.
3. Remux rejection for unsupported audio (for example DTS/TrueHD) →
   recoverable AVPlayer failure → FFmpeg fallback.
4. AVPlayer demux failure for a hardware-decodable video format (for example
   VP9) → compressed sample buffers through VideoToolbox.
5. Unsupported video/device combination → clear non-recoverable failure; do
   not silently attempt expensive software video decoding.

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

### 4. Complete audio compatibility policy

- Current policy: fMP4 adaptation is copy-only and accepts AAC audio. DTS,
  TrueHD, and other unsupported audio tracks reject adaptation as a recoverable
  failure so the FFmpeg fallback can handle the original local HTTP source;
  they are never silently dropped.
- Decide whether a future device/output policy should transcode those tracks
  to AAC or AC-3/E-AC-3.
- Ensure audio-only transcode keeps supported video bit-for-bit untouched.
- Verify language/default/forced track metadata and user track selection after
  adaptation.

### 5. Cover torrent playback explicitly

- The Apple adapter now routes the torrent server's local HTTP stream through
  the same local proxy/remux path used by direct Matroska sources.
- The FFmpeg fallback receives the proxy's direct local HTTP URL rather than a
  magnet URI, and the existing playback item preserves resume position.
- Still validate on-device with MKV and non-MKV torrent files, including seek
  and remux rejection behavior.

### 6. Finish Apple integration cleanup

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

### 7. Desktop/Tauri integration (later milestone)

- Integrate the Player Kit surface into macOS/Tauri only after the Apple kit
  and fallback behavior are stable.
- Keep existing desktop mpv/libvlc paths intact until the new path is proven.
- Do not modify `src-tauri` as part of the current Apple-only milestone.

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
