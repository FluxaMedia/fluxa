#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sdk="${WEBOS_SDK:-$HOME/.cache/webos-toolchain/sdk/arm-webos-linux-gnueabi_sdk-buildroot}"
version="${1:-$(node -p "require('$root/package.json').version")}"
version="${version#v}"
id=com.fluxa.native
src="$root/native/fluxa-webos/package"
bin="$root/native/target/armv7-unknown-linux-gnueabi/release/fluxa-webos"
out="$root/dist-web"
work="${TMPDIR:-$HOME/.cache/claude-tmp}/fluxa-ipk"

[ -f "$bin" ] || "$root/scripts/build-webos-native.sh" --features app --bin fluxa-webos

rm -rf "$work"
app="$work/data/usr/palm/applications/$id"
mkdir -p "$app" "$work/data/usr/palm/packages/$id" "$work/control" "$out"
install -m 755 "$bin" "$app/fluxa-webos"
"$sdk/bin/arm-linux-strip" "$app/fluxa-webos"
cp "$src/icon.png" "$src/largeIcon.png" "$app/"
node -e "
const fs = require('fs');
const info = JSON.parse(fs.readFileSync('$src/appinfo.json', 'utf8'));
info.version = process.argv[1];
fs.writeFileSync('$app/appinfo.json', JSON.stringify(info, null, 2) + '\n');
fs.writeFileSync('$work/data/usr/palm/packages/$id/packageinfo.json', JSON.stringify({ app: '$id', id: '$id', version: process.argv[1] }) + '\n');
" "$version"

size=$(du -sk "$work/data" | cut -f1)
cat > "$work/control/control" <<CONTROL
Package: $id
Version: $version
Section: misc
Priority: optional
Architecture: arm
Installed-Size: $size
Maintainer: Fluxa <noreply@fluxa.app>
Description: Fluxa
webOS-Package-Format-Version: 2
webOS-Packager-Version: 1.0.0
CONTROL

ipk="$out/fluxa-$version-webos-native.ipk"
rm -f "$ipk"
echo 2.0 > "$work/debian-binary"
tar --owner=0 --group=0 -C "$work/control" -czf "$work/control.tar.gz" .
tar --owner=0 --group=0 -C "$work/data" -czf "$work/data.tar.gz" .
main=$(node -p "require('$app/appinfo.json').main")
[ "$main" = fluxa-webos ] || { echo "appinfo main is '$main', expected fluxa-webos" >&2; exit 1; }
tar -tvzf "$work/data.tar.gz" | grep -E '^-rwx.* \./usr/palm/applications/'"$id"'/fluxa-webos$' >/dev/null ||
  { echo "fluxa-webos is not executable inside the ipk" >&2; exit 1; }
(cd "$work" && ar r "$ipk" debian-binary control.tar.gz data.tar.gz) 2>/dev/null
rm -rf "$work"
echo "packaged -> $ipk"
