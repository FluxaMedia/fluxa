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

`FluxaPlayerKit` is a local Swift package holding the shared playback stack for iOS, tvOS and later the macOS Tauri shell. `FluxaPlayer` is the only type callers touch, and `FluxaAVFoundationEngine` backed by AVPlayer is the sole playback engine.

FFmpeg is not a second renderer or a software video-player fallback. The Rust streaming layer uses FFmpeg/libavformat only as an AVPlayer compatibility filter: it probes sources, remuxes containers when necessary, repairs stream signaling, and performs selective elementary-stream conversion only when AVPlayer cannot consume that stream. Streams that AVPlayer already supports are copied without re-encoding, preserving the system HDR, Dolby Vision, VideoToolbox, Atmos, AirPlay and PiP pipeline.
