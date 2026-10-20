#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

profile="${FLUXA_WEB_PROFILE:-release}"
out="${1:-dist-web/site}"
www=native/fluxa-web/www
cargo_flags=()
[ "$profile" = release ] && cargo_flags+=(--release)

export RUSTFLAGS="${RUSTFLAGS:-} --cfg getrandom_backend=\"wasm_js\""
cargo build --manifest-path native/Cargo.toml -p fluxa-web --target wasm32-unknown-unknown "${cargo_flags[@]}"
wasm-bindgen --target web --out-dir "$www/pkg" "native/target/wasm32-unknown-unknown/$profile/fluxa_web.wasm"

rm -rf "$out"
mkdir -p "$out"
cp -r "$www"/. "$out"/
rm -f "$out/pkg/.gitignore"
version="$(cat "$out"/*.js "$out"/pkg/fluxa_web.js "$out"/pkg/fluxa_web_bg.wasm | sha1sum | cut -c1-10)"
sed -i -E "s#from \"\./([a-z0-9_/]+\.js)\"#from \"./\1?v=$version\"#; s#await init\(\)#await init({ module_or_path: \"./pkg/fluxa_web_bg.wasm?v=$version\" })#" "$out/main.js"
sed -i "s#src=\"main.js\"#src=\"main.js?v=$version\"#" "$out/index.html"
version_name="${FLUXA_VERSION:-$(node -p "require('./package.json').version")}"
sed -i "s#__FLUXA_SENTRY_DSN__#${FLUXA_SENTRY_DSN:-}#; s#__FLUXA_VERSION__#$version_name#" "$out/crash.js"
cp "$out/index.html" "$out/404.html"
touch "$out/.nojekyll"
echo "built web site -> $out"
