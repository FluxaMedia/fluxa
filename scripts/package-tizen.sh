#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

version="${1:-$(node -p "require('./package.json').version")}"
version="${version#v}"
tizen=native/fluxa-web/tizen
www=native/fluxa-web/www
app=dist-web/tizen-app
out=dist-web/fluxa-$version-tizen.wgt
target="$root/native/target/tizen"

export RUSTC_BOOTSTRAP=1
export CARGO_TARGET_DIR="$target"
export RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\" -C target-cpu=mvp"
cargo build --manifest-path native/Cargo.toml -p fluxa-web --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort

rm -rf "$app"
mkdir -p "$app/pkg"
cp "$www"/*.js "$www/index.html" "$www/favicon.svg" "$app"/
wasm-bindgen --target web --out-dir "$app/pkg" "$target/wasm32-unknown-unknown/release/fluxa_web.wasm"

sed -i 's#await init();#await init({ module_or_path: await loadWasm("./pkg/fluxa_web_bg.wasm") });#' "$app/main.js"
sed -i "s#__FLUXA_SENTRY_DSN__#${FLUXA_SENTRY_DSN:-}#; s#__FLUXA_VERSION__#$version#" "$app/crash.js"
node_modules/.bin/esbuild "$app/main.js" --bundle --format=iife --target=chrome76 --outfile="$app/bundle.js"
cat "$tizen/polyfill.js" "$app/bundle.js" > "$app/app.js"
rm -f "$app/bundle.js" "$app"/{main,a11y,artwork,player,trailer,remote,crash}.js "$app/pkg/fluxa_web.js" "$app/pkg"/*.d.ts "$app/pkg/package.json"
sed -i 's#<script type="module" src="main.js"></script>#<script src="app.js"></script>#' "$app/index.html"

rsvg-convert -w 512 -h 512 shared/brand/fluxa-mark.svg -o "$app/icon.png"
sed "s#__VERSION__#$version#" "$tizen/config.xml" > "$app/config.xml"

rm -f "$out"
(cd "$app" && zip -qr "$root/$out" .)
echo "packaged -> $out (unsigned, install with Apps2Samsung)"
