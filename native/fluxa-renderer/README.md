# Fluxa Native Renderer

This crate is the first native UI rendering slice for the Rust renderer migration.

It owns a renderer-independent `RenderScene`, a small animation primitive,
and WGPU renderers. The pipeline draws instanced rounded rectangles with
gradients, borders, opacity, and focus state. `SceneRenderer` is the composable
form: a host supplies its existing `Device`, `Queue`, target format, and
`RenderPass`, so it can share one surface and command encoder with egui or a
video overlay. `WgpuRenderer` remains available as a standalone smoke surface.
The scene model is deliberately independent from Fluxa domain data so screens
can be migrated without moving network, storage, or lifecycle code into the
renderer.

The renderer-independent `fluxa-theme` crate consumes
`shared/contracts/ui-tokens.json`. Both `fluxa-ui` and `fluxa-renderer` depend
on that token crate; shared screen code does not depend on this renderer crate
or on WGPU. The `fluxa-ui` crate derives responsive `UiMetrics` from the
selected viewport/form-factor; screen code should not introduce another set of
fixed card or navigation dimensions.
The `ui` module
contains platform-neutral nodes, events, and focus navigation. The `platform`
module defines the host boundary: Android tries Vulkan first and GLES second,
while surface creation, resize, and destruction are represented as lifecycle
events that Android, desktop, and future webOS adapters can share.

`fluxa-app` owns the host-independent `FluxaRuntime`: it is the single
application-facing boundary to `fluxa-core`, including snapshots, commands,
and effect results. Discover uses one Core action (`discoverRequested`) on all
hosts. A request can carry content type, catalog/genre/search filters, and an
optional `loadCatalogFilters` flag. When set, Core loads the filter options,
resolves the requested/default catalog, then runs the results request through
the same effect pipeline; hosts do not need separate catalog-load and discover
Core methods.

`fluxa-renderer::egui_wgpu_backend` owns the egui-wgpu painter and its pipeline
interaction. The native host still owns its OS window/surface lifecycle and
platform GPU/video interop, while shared components only produce egui UI and
never name WGPU resource types.

The Android host now has a Rust-owned home surface rendered by egui-wgpu over
the Android WGPU surface. It accepts a small JSON projection through
`setHomeStateNative` (`title`, `eyebrow`, `description`, hero artwork, catalog
rows, and cards) so the existing Android data source can feed the native scene without
passing Compose nodes into the renderer. Remote JPEG/PNG/WebP artwork is
decoded off the render thread, capped, and uploaded incrementally. The host
view polls a bounded native action queue through `pollActionsNative`, allowing
Rust-owned Home buttons, cards, and D-pad activation to navigate or open a
detail/play request in the existing Android core flow. The default model is
only a boot fallback. The Android host also exposes the visible Home controls
as virtual `AccessibilityNodeProvider` children and forwards accessibility
focus/click actions into the same Rust UI tree.

The Android Core cutover is behind `fluxaNativeCoreRuntime=true`. When
combined with `fluxaNativeHome=true`, the view dispatches `homeLoadRequested`
to `fluxa-app`, polls its effect queue, executes those effects through
`FluxaAndroidHeadlessEnvironment`, and projects the returned Core snapshot
directly in Rust. Home, Library, Discover, and Calendar now share the same Rust
route, top bar, focus tree, card model, and artwork loader; Library, Discover,
and Calendar navigation/filter/month actions also dispatch through the same effect bridge. In this mode the
legacy Home ViewModel bootstrap and cloud-stream binder are disabled, so they
do not create a second Home/Core owner. Home, Library, Discover, Calendar,
Detail, and Settings are now all represented by shared Rust UI models and
renderer routes; Compose remains only as the Android host/platform shell while
this feature flag is being hardened.

Run the desktop smoke surface with:

```bash
cargo run --manifest-path native/fluxa-renderer/Cargo.toml --example native-home
```

The desktop host now feeds the root scene into `SceneRenderer` before egui
compositing. Android can keep a minimal Activity and platform service bridge
while the visible catalog/player surfaces move to this crate one route at a
time.

For deterministic visual QA, the desktop host accepts a Core snapshot fixture:

```bash
FLUXA_NATIVE_FIXTURE=$PWD/native/fixtures/native-ui-fixture.json \
FLUXA_NATIVE_DISABLE_ARTWORK=1 \
FLUXA_NATIVE_INITIAL_PAGE=Home \
cargo run --manifest-path native/Cargo.toml -p fluxa-desktop
```

`FLUXA_NATIVE_INITIAL_PAGE` can be `Home`, `Library`, `Discover`, `Calendar`,
`Detail`, or `Settings`.
