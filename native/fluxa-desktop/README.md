# Fluxa Native Desktop

This is the native desktop shell for Fluxa. It owns the Winit window lifecycle and rewrites the structure of `apps/desktop/src/components/AppShell.tsx`, `NavSidebar.tsx`, `HeroSection.tsx`, `HomeScreen.tsx`, `ContinueCard.tsx`, and `MovieCard.tsx` as an interactive egui surface on WGPU. The current slice follows the shared Fluxa dark theme, floating navigation rail, global search/profile controls, hero treatment, Continue Watching cards, poster shelves, page navigation, and keyboard focus movement.

The UI reads its home model from the native `fluxa_core` snapshot through `FluxaRuntime`; the desktop crate has no Tauri dependency. The startup `readHomeBootstrap` effect is executed by the Rust effect executor, which reads the existing encrypted Fluxa KV store, serves cached home data, builds catalog requests through Fluxa Core, performs the HTTP requests natively, and completes the effect back into Fluxa Core. Artwork URLs from the returned metadata are loaded asynchronously into WGPU textures. No catalog content is fabricated by the renderer.

The artwork loader accepts PNG/JPEG/WebP and SVG from HTTP(S), `file://`, or
local relative/absolute paths. SVG is rasterized off the UI thread with
`resvg`, then uploaded only after the decoded image is ready. The poster grid
renders only rows intersecting the scroll viewport and keeps a bounded LRU
texture cache.

User fonts can be loaded without changing the source:

```bash
FLUXA_NATIVE_FONT_PATH="$PWD/fonts/Inter-Regular.ttf:$PWD/fonts/Inter-Bold.ttf" \
FLUXA_NATIVE_FONT_FAMILY=inter-regular \
cargo run --manifest-path native/Cargo.toml -p fluxa-desktop
```

`FLUXA_NATIVE_FONT_PATH` accepts a platform path list or a single font
directory, and `FLUXA_NATIVE_FONT_DIR` explicitly names a directory. TTF and
OTF files are validated before being added to egui's fallback chain.

Run it with:

```bash
cargo run --manifest-path native/Cargo.toml -p fluxa-desktop
```

Debug builds watch the shared token contract on every frame. Editing
`shared/contracts/ui-tokens.json` therefore refreshes colors, spacing and
responsive geometry in the already-open window. A different token file can
be used with:

```bash
FLUXA_NATIVE_TOKENS_PATH=/path/to/ui-tokens.json \
cargo run --manifest-path native/Cargo.toml -p fluxa-desktop
```

Rust drawing-code changes still need compilation because the renderer is a
native binary. For an automatic rebuild/relaunch loop, install
`cargo-watch` once and run:

```bash
cargo watch -w native -w shared -x 'run --manifest-path native/Cargo.toml -p fluxa-desktop'
```

This separates the two development loops: token/layout tuning is in-process
hot reload, while changing Rust component logic is an automatic native
rebuild rather than a manual close-and-reopen cycle.

On Linux the desktop shell uses the Vulkan WGPU backend. Override it for
diagnostics with `FLUXA_NATIVE_WGPU_BACKEND=gl` (or use `vulkan`, `metal`,
`dx12`, or `all`). Set `FLUXA_NATIVE_DISABLE_ARTWORK=1` to isolate
texture-upload problems while debugging.

The desktop frame uses `fluxa_renderer::SceneRenderer` for the native WGPU
scene pass on the same Vulkan device/surface, then composites the Rust egui
text/input layer above it. The renderer itself receives state only from Fluxa
Core; remaining product work is to add the other effect families (detail,
streams, search, auth, and native player controls) to the same executor
boundary.
