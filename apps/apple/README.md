# Apple hosts

`FluxaIos` is a SwiftUI host for the Compose Multiplatform `FluxaShared` framework. `FluxaTvos` is a native SwiftUI host backed by the Kotlin Multiplatform `FluxaCore` and `FluxaData` frameworks and loads its catalog through the same Rust headless action/effect flow as iOS. The current Compose and Coil dependency stack does not publish tvOS artifacts, so the tvOS visual layer remains native SwiftUI while application logic, Stremio protocol handling, and player state are shared through KMP and the Rust headless runtime.

On macOS, install XcodeGen and generate the project:

```bash
cd apps/apple
xcodegen generate
open FluxaApple.xcodeproj
```

The Xcode build phase invokes Gradle with the active Xcode SDK and architecture, then embeds the matching framework. Both targets bundle the shared English and Turkish i18n files from `shared/i18n`.

## FluxaPlayerKit

`FluxaPlayerKit` is a local Swift package holding the shared playback stack for iOS, tvOS and later the macOS Tauri shell. `FluxaPlayer` is the only type callers touch; behind it sit two interchangeable engines — `FluxaAVFoundationEngine` (AVPlayer, used whenever AVFoundation can play the stream natively) and `FluxaFFmpegEngine` (FFmpeg demux feeding `AVSampleBufferDisplayLayer` and `AVSampleBufferAudioRenderer` through a render synchronizer).

The FFmpeg engine is compiled only when `Vendor/CFFmpeg.xcframework` exists; without it the package builds AVFoundation-only and `FLUXA_FFMPEG` stays undefined. Build the framework once before enabling it:

```bash
cd apps/apple
./scripts/build-ffmpeg-xcframework.sh
```

The script fetches an FFmpeg release, configures an LGPL-only static build with a trimmed demuxer/decoder set, and merges `avformat`/`avcodec`/`avutil`/`swresample` into a single `CFFmpeg` module covering iOS, tvOS, their simulators and macOS. It skips itself when the framework is already present; pass `FLUXA_FORCE_FFMPEG_BUILD=1` to rebuild. Re-run `xcodegen generate` afterwards so the package picks up the new binary target.
