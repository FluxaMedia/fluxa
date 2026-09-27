# Apple hosts

`FluxaIos` and `FluxaTvos` are the same thin UIKit host (`Host/`) over the Rust renderer in `native/fluxa-apple-renderer`. Rust draws the UI into a `CAMetalLayer`; video plays through libmpv from [KhooLy/mpv](https://github.com/KhooLy/mpv) into a second `CAMetalLayer` behind it.

On macOS, install XcodeGen and generate the project:

```bash
cd apps/apple
xcodegen generate
open FluxaApple.xcodeproj
```

The build phases fetch `Libmpv.xcframework` from the fork's `libmpv-apple.xcframework.zip` release asset into `Vendor/` and build the Rust renderer for the active SDK. Both targets bundle the shared i18n files from `shared/i18n`.
