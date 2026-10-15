#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if ! command -v ares-package >/dev/null 2>&1; then
  echo "ares-package not found: npm install -g @webos-tools/cli" >&2
  exit 1
fi

version="${1:-$(node -p "require('./package.json').version")}"
version="${version#v}"
webos=native/fluxa-web/webos
app=dist-web/webos-app
service=dist-web/webos-service
out=dist-web

./scripts/build-web.sh "$app"
rm -f "$app/404.html" "$app/.nojekyll"
cp "$webos/icon.png" "$webos/largeIcon.png" "$app"/
node -e "
const fs = require('fs');
const info = JSON.parse(fs.readFileSync('$webos/appinfo.json', 'utf8'));
info.version = process.argv[1];
fs.writeFileSync('$app/appinfo.json', JSON.stringify(info, null, 2) + '\n');
" "$version"

rm -rf "$service"
mkdir -p "$service"
cp -r "$webos/services/com.fluxa.app.proxy" "$service/com.fluxa.app.proxy"

rm -f "$out"/*.ipk
ares-package --no-minify "$app" "$service/com.fluxa.app.proxy" -o "$out"
mv "$out"/com.fluxa.app_*.ipk "$out/fluxa-$version-webos.ipk"
echo "packaged -> $out/fluxa-$version-webos.ipk"
