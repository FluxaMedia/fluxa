# Fluxa Repository

Fluxa is a monorepo:

- `core/fluxa-core` — shared domain logic, policies, state, effects, and contracts
- `apps/android` — Android, Android TV, and shared Kotlin Multiplatform modules
- `apps/apple` — iOS and tvOS host code
- `apps/desktop` — desktop, browser, and webOS shell
- `shared/i18n` — single source for application translations; Android XML is generated from it

Put decisions, state transitions, policies, and cross-platform contracts in `core/fluxa-core`. Put UI, storage, networking, lifecycle, players, and OS integration in the relevant platform shell. Do not duplicate business rules in a shell because the core boundary is inconvenient.

## Desktop rules

The Desktop shell owns Tauri commands, native windows, OS integration, and React presentation. It must provide web fallbacks for native commands in `apps/desktop/src/platform/web/invoke.ts` where browser/webOS builds need them.

## Localization

Every user-facing string must be localized. Update both language files in `shared/i18n`; platform-native system labels may remain in the platform resource files.

## Commit messages

Use Conventional Commit prefixes for every commit:

- `feat:` for a user-visible feature
- `fix:` for a bug fix
- `perf:` for a performance improvement
- `docs:`, `test:`, `build:`, `ci:`, `refactor:`, or `chore:` for maintenance work

Add a scope when useful, for example `fix(desktop): avoid duplicate stream requests`. Keep the subject short, imperative, and focused on one change. Do not create vague subjects or mix unrelated changes in one commit. Release notes include only user-facing `feat`, `fix`, and `perf` work plus a small compatibility filter for older unprefixed history.

## Validation

Use the smallest relevant offline checks first:

```bash
npm run check:structure
npm run check:desktop
npm run test:desktop
npm run check:core
```

Do not run dependency downloads or large builds on mobile data. Build packaged targets only when the required SDKs and local libraries are available.
