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
cp "$out/index.html" "$out/404.html"
touch "$out/.nojekyll"
echo "built web site -> $out"
