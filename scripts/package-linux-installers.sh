#!/usr/bin/env bash
set -euo pipefail

app="${1:?app directory is required}"
version="${2:?version is required}"
dist="${3:?output directory is required}"
root="$(cd "$(dirname "$0")/.." && pwd)"
tools="$root/dist/tools"
mkdir -p "$tools"

nfpm_version=2.41.0
appimagetool_version=1.9.0

curl -fsSL "https://github.com/goreleaser/nfpm/releases/download/v${nfpm_version}/nfpm_${nfpm_version}_Linux_x86_64.tar.gz" | tar -xz -C "$tools" nfpm
curl -fsSL -o "$tools/appimagetool" "https://github.com/AppImage/appimagetool/releases/download/${appimagetool_version}/appimagetool-x86_64.AppImage"
chmod +x "$tools/appimagetool"

sed -e "s|\${FLUXA_VERSION}|$version|g" -e "s|\${FLUXA_APP_DIR}|$app|g" "$root/packaging/linux/nfpm.yaml" > "$tools/nfpm.yaml"
(cd "$root" && "$tools/nfpm" package --config "$tools/nfpm.yaml" --packager deb --target "$dist/fluxa-$version-linux-amd64.deb")
(cd "$root" && "$tools/nfpm" package --config "$tools/nfpm.yaml" --packager rpm --target "$dist/fluxa-$version-linux-x86_64.rpm")

appdir="$dist/Fluxa.AppDir"
rm -rf "$appdir"
mkdir -p "$appdir"
cp -a "$app" "$appdir/fluxa"
cp "$root/packaging/linux/fluxa.desktop" "$appdir/fluxa.desktop"
cp "$root/packaging/icons/fluxa.png" "$appdir/fluxa.png"
cat > "$appdir/AppRun" <<'RUN'
#!/bin/sh
here="$(dirname "$(readlink -f "$0")")"
exec "$here/fluxa/fluxa-desktop" "$@"
RUN
chmod +x "$appdir/AppRun"
ARCH=x86_64 "$tools/appimagetool" --appimage-extract-and-run "$appdir" "$dist/fluxa-$version-linux-x86_64.AppImage"
rm -rf "$appdir"
