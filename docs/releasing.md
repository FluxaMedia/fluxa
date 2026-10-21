# Releasing

Push a tag like `v0.2.0`. `.github/workflows/release.yml` builds everything and publishes one GitHub Release. `workflow_dispatch` with an existing tag re-runs it.

## What a release contains

| Platform | Files |
| --- | --- |
| Windows | `fluxa-<v>-windows-x86_64-setup.exe`, `.msi`, `.zip` |
| macOS (arm64) | `.dmg`, `.zip` |
| Linux (x86_64) | `.AppImage`, `.deb`, `.rpm`, `.tar.gz` |
| webOS | `fluxa-<v>-webos-native.ipk` |
| Android / Android TV | per-ABI `fluxa-<v>-android-{mobile,tv}-<abi>.apk` |
| iOS / tvOS | `fluxa-<v>-{ios,tvos}-unsigned.ipa` (sideload only) |
| Updates | `SHA256SUMS`, `latest.json`, `latest.json.sig` |

The `.zip` / `.tar.gz` / `.AppImage` / `.apk` files are also the payloads the in-app updater downloads.

The web build is not attached to the release. `.github/workflows/pages.yml` runs on the same `v*` tag and deploys it to GitHub Pages.

## Secrets

- `FLUXA_UPDATE_PRIVATE_KEY`: ed25519 key that signs `latest.json`. The public half is `shared/update-public-key.txt` and is compiled into the app. Generate a new pair with `node scripts/update-manifest.mjs keygen`; rotating it strands installed apps, which then need a manual reinstall.
- `FLUXA_SENTRY_DSN`: Sentry DSN compiled into the desktop, Android, iOS/tvOS, webOS and web builds. Empty means crash reporting stays off. Users can turn it off in Settings → About → Crash reports.
- Android keystore and provider keys are the existing `FLUXA_RELEASE_*`, `FLUXA_TRAKT_*`, `FLUXA_SIMKL_*`, `FLUXA_ANILIST_*`, `FLUXA_NUVIO_*` secrets.

## In-app updates

- Desktop, AppImage and Android check `releases/latest/download/latest.json` (signature verified), download the matching asset, verify its sha256 and stage it.
- Settings → Updates shows the state; "Restart to update" swaps the files and relaunches (Android hands the APK to the system installer).
- `.deb` and `.rpm` installs update through `pkexec dpkg -i` / `rpm -U` (a polkit prompt). Installs in a read-only location such as Program Files cannot self-update and say so.
- `automaticUpdates` in General controls the background check.

## SideStore

After each release the workflow commits `sidestore.json` to the default branch, an AltStore-style source pointing at the unsigned iOS ipa. Add `https://raw.githubusercontent.com/FluxaMedia/fluxa/HEAD/sidestore.json` as a source in SideStore. The source lists only the newest version. tvOS is not supported by SideStore.
