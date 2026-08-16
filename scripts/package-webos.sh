#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if ! command -v ares-package >/dev/null 2>&1; then
  echo "ares-package not found. Install the webOS TV CLI:" >&2
  echo "  npm install -g @webos-tools/cli" >&2
  exit 1
fi

version="$(node -p "require('./package.json').version")"
staging="build/webos-service"

npm run build:webos

rm -rf dist-webos/services
cp webos/icon.png webos/largeIcon.png dist-webos/
node -e "
const fs = require('fs');
const info = JSON.parse(fs.readFileSync('webos/appinfo.json', 'utf8'));
info.version = process.argv[1];
fs.writeFileSync('dist-webos/appinfo.json', JSON.stringify(info, null, 2) + '\n');
" "$version"

rm -rf "$staging"
mkdir -p "$staging"
cp -r webos/services/com.fluxa.app.proxy "$staging/com.fluxa.app.proxy"

ares-package --no-minify dist-webos "$staging/com.fluxa.app.proxy" -o dist-webos

echo "packaged com.fluxa.app ${version} -> $(ls dist-webos/*.ipk)"
