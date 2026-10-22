# Releasing

Push a tag like `v0.2.0`. `.github/workflows/release.yml` builds everything and publishes one GitHub Release. `workflow_dispatch` with an existing tag re-runs it.

## What a release contains

| Platform | Files |
| --- | --- |
| Windows | `fluxa-<v>-windows-x86_64-setup.exe` (per-user install), `-portable.zip` |
| macOS (arm64) | `.dmg`, `-portable.zip` |
| Linux (x86_64) | `.AppImage`, `.deb`, `.rpm`, `-portable.tar.gz` |
| webOS | `fluxa-<v>-webos-native.ipk` |
| Android / Android TV | per-ABI `fluxa-<v>-android-{mobile,tv}-<abi>.apk` |
| iOS / tvOS | `fluxa-<v>-{ios,tvos}-unsigned.ipa` (sideload only) |
| Updates | `SHA256SUMS`, `latest.json`, `latest.json.sig` |

Each packaging step writes a `*.updatekeys` file listing `manifest-key=filename` for what it built. The release job feeds those to `scripts/update-manifest.mjs`, which refuses to publish if any key in `shared/update-required-assets.txt` is missing, and `scripts/verify-release.mjs` re-checks the published manifest, signature and every asset hash after the release is created.

The `.zip` / `.tar.gz` / `.AppImage` / `.apk` / `-setup.exe` files are also the payloads the in-app updater downloads.

The web build is not attached to the release. `.github/workflows/pages.yml` runs on the same `v*` tag and deploys it to GitHub Pages.

## Secrets

- `FLUXA_UPDATE_PRIVATE_KEY`: ed25519 key that signs `latest.json`. The public half is `shared/update-public-key.txt` and is compiled into the app. Generate a new pair with `node scripts/update-manifest.mjs keygen`; `shared/update-public-key.txt` may list several keys separated by whitespace and any of them verifies a manifest. To rotate, ship a release that lists both the current and the next key, then switch `FLUXA_UPDATE_PRIVATE_KEY` to the next one in a later release.
- `FLUXA_SENTRY_DSN`: Sentry DSN compiled into the desktop, Android, iOS/tvOS, webOS and web builds. Empty means crash reporting stays off. Users can turn it off in Settings → About → Crash reports.
- Android keystore and provider keys are the existing `FLUXA_RELEASE_*`, `FLUXA_TRAKT_*`, `FLUXA_SIMKL_*`, `FLUXA_ANILIST_*`, `FLUXA_NUVIO_*` secrets.

## In-app updates

- Desktop, AppImage and Android check `releases/latest/download/latest.json` (signature verified), download the matching asset, verify its sha256 and stage it.
- Settings → Updates shows the state; "Restart to update" swaps the files and relaunches (Android hands the APK to the system installer).
- Windows installs from `-setup.exe` into `%LocalAppData%\Programs\Fluxa` without admin rights; the update runs the new setup silently and it relaunches the app. The portable zip updates in place when its folder is writable.
- `.deb` and `.rpm` installs update through `pkexec dpkg -i` / `rpm -U` (a polkit prompt). Installs in a read-only location say so.
- Portable, AppImage and macOS updates keep the previous files until the new version has run for 30 seconds. If it starts three times without getting that far, the previous files are restored and that version is skipped.
- Download resumes from the partial file when the server supports ranges.
- Add `--min-supported vX.Y.Z` to the manifest build to make older versions show the update prompt even after "Later".
- A failed update check, download or install is reported through the crash reporter (when enabled) with the platform and current version.
- `automaticUpdates` in General controls the background check.

## SideStore

After each release the workflow commits `sidestore.json` to the default branch, an AltStore-style source pointing at the unsigned iOS ipa. Add `https://raw.githubusercontent.com/FluxaMedia/fluxa/HEAD/sidestore.json` as a source in SideStore. The source lists only the newest version. tvOS is not supported by SideStore.
