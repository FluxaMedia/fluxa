# Apple hosts

`FluxaIos` hosts the Rust renderer. `FluxaTvos` is a native SwiftUI host that loads its catalog through the Rust headless action/effect flow over UniFFI (`Generated/FluxaRustCore.swift`); no Kotlin frameworks are involved.

On macOS, install XcodeGen and generate the project:

```bash
cd apps/apple
xcodegen generate
open FluxaApple.xcodeproj
```

The Xcode build phase builds the Rust core for the active SDK and architecture. Both targets bundle the shared English and Turkish i18n files from `shared/i18n`.

## FluxaPlayerKit

`FluxaPlayerKit` is a local Swift package holding the shared playback stack for iOS and tvOS. `FluxaPlayer` is the only type callers touch, and `FluxaAVFoundationEngine` backed by AVPlayer is the sole playback engine.

FFmpeg is not a second renderer or a software video-player fallback. The Rust streaming layer uses FFmpeg/libavformat only as an AVPlayer compatibility filter: it probes sources, remuxes containers when necessary, repairs stream signaling, and performs selective elementary-stream conversion only when AVPlayer cannot consume that stream. Streams that AVPlayer already supports are copied without re-encoding, preserving the system HDR, Dolby Vision, VideoToolbox, Atmos, AirPlay and PiP pipeline.
