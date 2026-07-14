# Apple hosts

`FluxaIos` is a SwiftUI host for the Compose Multiplatform `FluxaShared` framework. `FluxaTvos` is a native SwiftUI host backed by the Kotlin Multiplatform `FluxaCore` framework and loads its catalog through the same Rust headless action/effect flow as iOS. The current Compose and Coil dependency stack does not publish tvOS artifacts, so the tvOS visual layer remains native SwiftUI while application logic is shared through KMP core and the Rust headless runtime.

On macOS, install XcodeGen and generate the project:

```bash
cd appleApp
xcodegen generate
open FluxaApple.xcodeproj
```

The Xcode build phase invokes Gradle with the active Xcode SDK and architecture, then embeds the matching framework. Both targets bundle the shared English and Turkish i18n files from `core/src/commonMain/resources/i18n`.
