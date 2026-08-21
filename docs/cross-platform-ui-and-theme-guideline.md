# Fluxa Cross-Platform UI and Theme Guideline

## Purpose

This document defines how Fluxa should share application logic, UI contracts, themes, and platform behavior across:

- Android mobile
- Android TV
- iOS
- tvOS
- Windows, macOS, and Linux desktop
- browser
- LG webOS

The goal is Kodi-like platform coverage without making Compose, React, SwiftUI, or webOS-specific code the source of truth for the whole product.

The rule is:

> Business decisions are shared. Presentation is platform-adapted. Theme data is framework-independent.

## Current repository map

| Area | Repository | Current role | Integration |
|---|---|---|---|
| Domain and application engine | `fluxa-core` | State machines, addon protocol, stream policy, playback policy, profiles, library, sync, typed effects | Native Rust, JNI, UniFFI, WASM |
| Android mobile and TV | `fluxa` | Kotlin shell, Compose UI, Room, HTTP, Media3/MPV, Android lifecycle | JNI plus UniFFI |
| iOS | `fluxa` / `appleApp` | Compose Multiplatform host and Swift platform integration | UniFFI and generated Swift |
| tvOS | `fluxa` / `appleApp` | Native SwiftUI visual shell with shared KMP/core contracts | UniFFI and KMP data modules |
| Desktop, browser, webOS | `fluxa-desktop` | React UI, Tauri shell, native desktop player, browser/webOS adapters | Direct Rust, WASM, Tauri |
| Documentation website | `fluxa-docs-web` | Product and developer documentation | Independent Astro application |
| Sync service | `fluxa-sync` | Server-side sync and persistence | HTTP/database service |

The existing split is valid. Do not merge the Android and desktop UIs merely to reduce file count.

## Ownership rules

### `fluxa-core` owns

- content and addon decisions
- stream selection and fallback policy
- playback policy
- library and watch-state decisions
- profile contracts and preference migration
- sync planning and external service mapping
- effect definitions and wire contracts
- platform-neutral data models

`fluxa-core` must not know about Compose, React, SwiftUI, CSS, Android Views, Tauri, or webOS UI APIs.

### Platform shells own

- HTTP, filesystem, database, and OS APIs
- player handles and native surfaces
- notifications and background work
- permissions and lifecycle
- platform-specific input and focus
- screen rendering
- image/font loading
- theme adaptation

### UI layers own

- transient UI state
- navigation presentation
- dialogs, focus, and local gestures
- rendering and layout
- converting a core snapshot into visible controls

UI code may dispatch an intent or complete an effect. It must not decide watched state, select the next episode, resolve streams, or duplicate addon policy.

## Core contract rules

The effect loop is the cross-platform application contract:

```text
host dispatches action
        ↓
fluxa-core returns state and effects
        ↓
platform executes effects
        ↓
platform completes effects
        ↓
fluxa-core returns the next state and effects
```

When adding a feature:

1. Define the state, action, effect, and result in `fluxa-core`.
2. Keep JSON field names and nesting stable.
3. Add or update the core contract tests and wire fixtures.
4. Add the desktop route in `fluxa-core::ffi` only when desktop needs it.
5. Add Android JNI only when Android needs a direct binding.
6. Keep Swift on the shared `core_invoke`/UniFFI contract where possible.
7. Keep WASM on the same `core_invoke` contract.
8. Implement the platform effect handler in each shell.
9. Render the resulting state in each UI.

Do not create a desktop-only or Android-only business rule because a platform adapter is inconvenient.

The two existing Rust state engines are intentional. `headless_engine` is the primary engine; `app_state` remains a separate Android-facing engine until a deliberate migration is planned. Do not merge them as an incidental refactor.

## UI architecture

Fluxa should use two primary UI families:

```text
React UI
  desktop + browser + webOS

Compose Multiplatform UI
  Android + Android TV + iOS where supported

SwiftUI adapter
  tvOS-specific visual shell and Apple platform integration where required
```

The same screen does not need identical source code. It must consume the same concepts:

- destination and route identifiers
- screen snapshot models
- actions and intents
- loading, empty, and error states
- card metadata
- player state
- theme tokens
- translation keys

A screen-specific model should be serializable or representable in both TypeScript and Kotlin. Do not pass framework objects through the core boundary.

### Screen contract example

```text
HomeSnapshot
  hero
  shelves
  continueWatching
  loadingState
  error

HomeIntent
  refresh
  openItem(id)
  playItem(id)
  selectShelf(id)
```

React and Compose can render this differently for a mouse, touch screen, remote, or browser while preserving the same core behavior.

## Portable theme system

### Source of truth

Theme definitions must be framework-independent data. A Compose file, React component, CSS file, or SwiftUI view must never be the canonical theme format.

The long-term source of truth should be a small versioned theme contract package, either:

- a dedicated `fluxa-theme` package/repository, or
- a `contracts/theme` directory maintained with the cross-platform contract files.

The first implementation may live in `fluxa-desktop` while the schema is stabilized. `contracts/built-in-themes.json` contains the built-in packs, and `npm run theme:sync` generates the native consumers from it; generated files must not be hand-edited.

### Theme pack format

A theme pack contains data and assets, not executable UI code:

```text
theme-pack/
  theme.json
  assets/
    logo.png
    background.webp
  fonts/
    display.ttf
```

Example `theme.json`:

```json
{
  "schemaVersion": 1,
  "id": "midnight-blue",
  "nameKey": "theme.midnight_blue",
  "colors": {
    "background": "#080A10",
    "backgroundElevated": "#0D1220",
    "surface": "#121A2A",
    "surfaceRaised": "#1B263D",
    "navigation": "#090E1A",
    "textPrimary": "#FFFFFF",
    "textSecondary": "#A9B4C8",
    "textMuted": "#71809A",
    "border": "#24A9B4C8",
    "borderStrong": "#47A9B4C8",
    "accent": "#5C8DFF",
    "accentForeground": "#FFFFFF",
    "success": "#45D483",
    "warning": "#F0C674",
    "error": "#FF6B6B",
    "info": "#65B5FF",
    "focus": "#FFFFFF",
    "scrim": "#C7080A10"
  },
  "typography": {
    "displayFont": "display",
    "bodyFont": "system",
    "titleWeight": 700,
    "bodyWeight": 400
  },
  "shape": {
    "cardRadius": 12,
    "controlRadius": 8,
    "dialogRadius": 16
  },
  "spacing": {
    "screenPadding": 24,
    "sectionGap": 20,
    "controlGap": 12
  },
  "motion": {
    "enabled": true,
    "fastMs": 120,
    "normalMs": 180,
    "slowMs": 300
  },
  "layouts": {
    "home": "shelves",
    "detail": "hero-with-rail",
    "library": "poster-grid",
    "navigation": "sidebar"
  }
}
```

The actual schema must define valid color formats, ranges, required fields, and fallback behavior. Invalid or unknown values must fall back to the default Fluxa theme.

### Token categories

Use semantic tokens instead of raw color names:

- `background`, `backgroundElevated`
- `surface`, `surfaceRaised`, `navigation`
- `textPrimary`, `textSecondary`, `textMuted`
- `border`, `borderStrong`, `focus`
- `accent`, `accentForeground`
- `success`, `warning`, `error`, `info`
- typography and font roles
- spacing and shape roles
- animation durations and reduced-motion behavior

Content-specific colors are separate from the application theme. IMDb yellow, rating colors, media badges, and stream-quality colors must not be replaced accidentally by a global text or accent token.

### Frontend adapters

React maps the theme to CSS variables:

```text
theme.json → --fluxa-background
           → --fluxa-surface
           → --fluxa-text-primary
           → --fluxa-accent
```

Compose maps the same data to:

```text
FluxaTheme
  MaterialTheme.colorScheme
  FluxaTypography
  FluxaDimensions
  FluxaShapes
  FluxaMotion
```

SwiftUI maps it to an environment value or an equivalent `FluxaTheme` value. webOS uses the React/CSS adapter and must not receive a Compose/WASM theme implementation.

The adapter is allowed to translate a token for platform needs. It is not allowed to invent a different meaning for the token. The currently supported cross-platform home layout presets are `shelves` and `compact`; unknown presets fall back to `shelves` on native shells.

### Theme scope and sync

Use an explicit scope:

- selected built-in theme ID: profile preference if profiles are expected to look different
- custom theme metadata: local or profile-scoped preference
- theme assets: local device storage
- synced preference: theme ID and compatible settings only

If a synced theme is missing locally, use the default theme and show no broken asset references. Do not sync large theme assets through the core state or watch-progress sync.

### Theme limitations

Portable themes may customize:

- colors
- fonts
- spacing
- radii
- shadows and blur preferences
- animation timing
- named layout presets
- approved artwork slots

Portable themes must not contain:

- Compose code
- React components
- SwiftUI views
- JavaScript or Kotlin to execute
- arbitrary navigation logic
- arbitrary core actions
- platform-specific player code

If a theme needs a completely new component tree, it is a product feature and must be implemented in each UI family behind the same named layout contract.

## Current migration targets

### Desktop, browser, and webOS

The current desktop UI has overlapping token sources:

- `src/theme.ts`
- `src/design/tokens.ts`
- `src/index.css`
- hardcoded colors in screen/component style objects

Consolidate these into one React theme adapter. `useAppLayoutPrefs` should apply the selected theme rather than only applying the accent color. Migrate hardcoded application colors to semantic CSS variables. Keep media/provider brand colors explicit.

The same React build is the correct UI path for browser and webOS. Platform modules such as `src/platform/webos.ts` should continue to provide only webOS capabilities and services.

The first desktop foundation is now present: `contracts/theme.schema.json` defines the portable contract, `contracts/built-in-themes.json` is the built-in data source, and `src/theme/adapter.ts` maps it to CSS variables. Built-in theme selection and a first skin capability (showing or hiding Calendar in navigation) are wired through the existing profile preferences. The adapter validates theme data and safely falls back when skin JSON is invalid.

The desktop shell now accepts and exports validated JSON theme packs up to 256 KiB, stores up to 24 custom packs in profile preferences, and applies skin visibility and section-order settings to the Home and Detail screens. Detail skin section IDs are `hero`, `actions`, `meta`, `tabs`, `episodes`, `details`, `related`, and `rail`. Packs contain data only; they cannot execute code. Artwork/font installation and drag-and-drop route ordering remain future extensions. The Compose adapter and Android custom JSON import are implemented in `fluxa`; the tvOS SwiftUI adapter loads a persisted JSON pack from `AppStorage` and is implemented in `appleApp/tvOS`.

### Android and Android TV

The shared Compose UI already contains a common application shell and a separate TV home path. `FluxaThemePack` deserializes the portable contract, maps semantic colors into `ColorScheme`, and persists the selected built-in theme in shared settings. TV layout differences remain valid and are applied independently from the theme data.

TV layout differences remain valid. A TV-specific layout preset may change focus spacing, navigation rail width, card sizes, and animation behavior without changing core decisions.

Do not make TV behavior depend on a desktop or web layout assumption.

### iOS and tvOS

The existing Apple host already exposes the shared core and Compose/KMP data sources. Keep platform lifecycle, playback, and Apple framework integration in the Apple shell. tvOS uses `FluxaTvosTheme.swift` to decode the same theme fields and expose them through SwiftUI environment values.

The current tvOS separation is intentional because the existing Compose and dependency stack does not provide the same tvOS surface as Android/iOS. tvOS should consume the same theme contract through a SwiftUI adapter instead of trying to consume Android Material theme classes.

### `fluxa-core`

Keep theme rendering out of the domain engine. `fluxa-core` may own:

- theme preference migration if it is part of the shared settings contract
- theme ID validation
- a schema/version contract
- serialization needed by hosts

It must not import UI toolkit types or render theme assets.

## Repository rules

1. A new feature starts with the question: does it belong in `fluxa-core` or in a platform shell?
2. If it changes a decision, state transition, policy, or effect, start in `fluxa-core`.
3. If it changes layout, rendering, focus, native APIs, or platform lifecycle, implement it in the relevant shell.
4. Keep wire fields stable. Coordinate changes across all consumers before renaming them.
5. Do not add a platform-specific business rule to make a UI implementation easier.
6. Do not make a theme framework-specific.
7. Do not add a new raw color token when an existing semantic token applies.
8. Do not use a theme pack to execute code or dispatch arbitrary actions.
9. Every user-facing string must use the platform’s shared translation key contract.
10. Every platform-specific fallback must have an explicit capability check and a documented default.

## Consumer adapter contract

Every UI family should implement one pure conversion boundary:

```text
ThemePack JSON + SkinLayout JSON
        ↓ validate and merge defaults
FluxaTheme
        ↓ map semantic roles to toolkit values
Compose MaterialTheme / SwiftUI Environment / CSS variables
```

The adapter must preserve these roles without exposing toolkit types outside the UI shell:

| Portable role | React/CSS | Compose | SwiftUI |
| --- | --- | --- | --- |
| `colors.background` | `--fluxa-background` | `ColorScheme.background` | `Color` environment role |
| `colors.surface` | `--fluxa-surface` | `ColorScheme.surface` | surface environment role |
| `colors.textPrimary` | `--fluxa-text-primary` | `onBackground`/content color | primary foreground |
| `colors.accent` | `--fluxa-accent` | `primary` | tint/accent |
| `typography.displayFont` | font-family variable | `FontFamily` fallback | `Font.custom` fallback |
| `shape.cardRadius` | `--fluxa-card-radius` | card shape | card `ShapeStyle` |
| `spacing.sectionGap` | `--fluxa-section-gap` | section spacing | section spacing |
| `motion.normalMs` | CSS transition duration | animation duration | animation duration |

Android should deserialize the same JSON into a Kotlin data model in the shared module, then build `darkColorScheme` or `lightColorScheme` from it. tvOS should deserialize the same fields into a Swift value and expose them through the SwiftUI environment. Neither adapter should import React/CSS or copy desktop component code.

## Recommended implementation order

### Phase 1: Freeze and document contracts

- document the core action/effect/state contract
- identify duplicated desktop/Android field names
- define the initial `ThemePack` schema
- define the required token list and defaults
- add schema validation tests

Status: desktop contract, built-in data, documentation, validation tests, and generated native consumers are implemented. A dedicated cross-platform contracts package remains an optional repository-organization improvement.

### Phase 2: Build the theme adapters

- add the default Fluxa theme as portable JSON
- add the React adapter and CSS variable map
- add the Compose adapter
- add the SwiftUI adapter for tvOS
- add a snapshot/golden test that the same theme produces equivalent semantic tokens

Status: React/CSS, Compose, and tvOS SwiftUI adapters are implemented. Native built-in consumers are generated from the desktop contract and checked by `npm run theme:verify`. A dedicated generated contract package remains an optional repository-organization improvement.

### Phase 3: Migrate existing UI

- consolidate `theme.ts`, `design/tokens.ts`, and global CSS tokens
- replace desktop hardcoded application colors
- replace Compose fixed color scheme and direct color usage
- replace tvOS hardcoded application colors
- preserve content-brand colors as separate tokens

Status: global desktop tokens, root styling, navigation, loading surfaces, profile selection, Home/Detail, Search, Library, Category, poster grid, and hero surfaces use semantic variables. Provider/IMDb/rating colors and player-specific overlays remain explicit by design. Android consumes the same semantic data through Compose, and tvOS applies the shared roles through SwiftUI. A final desktop hardcoded-color audit is still required before calling this phase complete.

### Phase 4: Add built-in theme selection

- add `themeId` to the shared settings contract
- add built-in themes
- add live preview and reset
- persist the theme at the selected profile scope
- verify that missing/invalid themes fall back safely

Status: desktop and Android built-in themes, custom JSON import, persistence fields, live application, and safe fallback are implemented. tvOS loads its persisted theme pack through `AppStorage`. Live preview/reset, shared profile-scope verification, and native tvOS build verification remain.

### Phase 5: Add custom theme packs

- import a zip containing `theme.json`, assets, and fonts
- validate schema and file sizes before installation
- reject executable files and unsupported asset types
- store packs outside the core state machine
- expose only installed theme metadata to every platform
- add export and delete actions

Status: desktop JSON import/export, validation, profile persistence, reserved-ID protection, and bounded pack storage are implemented. Zip assets/fonts and pack deletion UI remain.

### Phase 6: Add layout presets

- define a small set of named layouts
- implement each preset in React and Compose
- implement only supported presets in tvOS/webOS
- expose platform capability fallback instead of pretending every layout is identical

Status: desktop Home section visibility and order are implemented. Android Compose and tvOS consume the `compact` home preset and fall back to the default shelves behavior for unknown values. Broader named layout presets remain.

## Acceptance criteria

A theme feature is complete only when:

- the same theme JSON can be loaded by React and Compose
- webOS uses the React adapter without a separate theme fork
- Android TV can use the same theme data with TV-specific layout adjustments
- tvOS can use the same colors, typography roles, and shapes through SwiftUI
- no business decision is duplicated in a theme or UI layer
- invalid packs cannot crash the app or execute code
- a missing custom font or asset has a deterministic fallback
- theme selection survives restart and profile changes correctly
- typecheck/build tests cover every affected consumer

The Apple native build must be verified in Xcode CI or on macOS. Linux can verify the generated Swift source shape and the shared Android consumer, but cannot prove the tvOS target compiles.

## Final architectural decision

Fluxa should not attempt to share the exact same UI source code across every platform. It should share:

```text
Rust core logic
JSON action/effect/state contracts
translation keys
theme packs and semantic tokens
screen model concepts
test fixtures
```

It should adapt presentation where the platform requires it:

```text
React/CSS for web, webOS, and desktop
Compose for Android and Android TV
Compose Multiplatform where it is stable and useful on Apple platforms
SwiftUI for tvOS-specific integration when necessary
```

This keeps the product portable without making a Compose theme or a React component the hidden source of truth for another platform.
