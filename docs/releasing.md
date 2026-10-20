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
| Web | `fluxa-<v>-web.zip` |
| Updates | `SHA256SUMS`, `latest.json`, `latest.json.sig` |

The `.zip` / `.tar.gz` / `.AppImage` / `.apk` files are also the payloads the in-app updater downloads.

## Secrets

- `FLUXA_UPDATE_PRIVATE_KEY`: ed25519 key that signs `latest.json`. The public half is `shared/update-public-key.txt` and is compiled into the app. Generate a new pair with `node scripts/update-manifest.mjs keygen`; rotating it strands installed apps, which then need a manual reinstall.
- Android keystore and provider keys are the existing `FLUXA_RELEASE_*`, `TRAKT_*`, `SIMKL_*`, `ANILIST_*`, `FLUXA_NUVIO_*` secrets.

## In-app updates

- Desktop, AppImage and Android check `releases/latest/download/latest.json` (signature verified), download the matching asset, verify its sha256 and stage it.
- Settings → Updates shows the state; "Restart to update" swaps the files and relaunches (Android hands the APK to the system installer).
- `.deb` and `.rpm` installs update through `pkexec dpkg -i` / `rpm -U` (a polkit prompt). Installs in a read-only location such as Program Files cannot self-update and say so.
- `automaticUpdates` in General controls the background check.
