#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pgo_dir="$root_dir/target/pgo"
rust_llvm="$(rustc -vV | awk '/LLVM version/ {print $3; exit}')"
llvm_major="${rust_llvm%%.*}"
profdata_tool="$(command -v "llvm-profdata-$llvm_major" || command -v llvm-profdata)"
profdata_llvm="$("$profdata_tool" --version | awk '/LLVM version/ {print $3; exit}')"
if [[ "${rust_llvm%%.*}" != "${profdata_llvm%%.*}" ]]; then
    echo "rustc uses LLVM $rust_llvm but llvm-profdata is $profdata_llvm" >&2
    exit 2
fi
rm -rf "$pgo_dir"
mkdir -p "$pgo_dir"

LLVM_PROFILE_FILE="$pgo_dir/%m-%p.profraw" \
RUSTFLAGS="-Cprofile-generate=$pgo_dir" \
cargo run --release --manifest-path "$root_dir/Cargo.toml" --bin fluxa-perf-hotpaths

"$profdata_tool" merge -o "$pgo_dir/fluxa.profdata" "$pgo_dir"/*.profraw
RUSTFLAGS="-Cprofile-use=$pgo_dir/fluxa.profdata" \
cargo build --release --manifest-path "$root_dir/Cargo.toml"
