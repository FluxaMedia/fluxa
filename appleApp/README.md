# Apple hosts

`FluxaIos` is a SwiftUI host for the Compose Multiplatform `FluxaShared` framework. `FluxaTvos` is a native SwiftUI host backed by the Kotlin Multiplatform `FluxaCore` framework.

On macOS, install XcodeGen and generate the project:

```bash
cd appleApp
xcodegen generate
open FluxaApple.xcodeproj
```

The Xcode build phase invokes Gradle with the active Xcode SDK and architecture, then embeds the matching framework. Both targets bundle the shared English and Turkish i18n files from `core/src/androidMain/assets/i18n`.
