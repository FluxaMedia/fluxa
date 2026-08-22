#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bytes="${FLUXA_BUFFER_BENCH_BYTES:-67108864}"
for size in 65536 131072 262144; do
    FLUXA_BUFFER_SIZE="$size" FLUXA_BUFFER_BYTES="$bytes" \
        cargo run --release --manifest-path "$root_dir/Cargo.toml" \
        --bin fluxa-buffer-bench
done
