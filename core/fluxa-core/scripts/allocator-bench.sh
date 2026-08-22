#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
iterations="${FLUXA_PERF_ITERATIONS:-100000}"
binary="$root_dir/target/release/fluxa-perf-hotpaths"

if [[ ! -x "$binary" ]]; then
    cargo build --release --manifest-path "$root_dir/Cargo.toml" --bin fluxa-perf-hotpaths
fi

echo "default allocator"
if [[ -x /usr/bin/time ]]; then
    FLUXA_PERF_ITERATIONS="$iterations" /usr/bin/time -f 'elapsed=%e rss_kb=%M' "$binary"
else
    FLUXA_PERF_ITERATIONS="$iterations" "$binary"
fi

echo "single glibc arena"
if [[ -x /usr/bin/time ]]; then
    MALLOC_ARENA_MAX=1 FLUXA_PERF_ITERATIONS="$iterations" \
        /usr/bin/time -f 'elapsed=%e rss_kb=%M' "$binary"
else
    MALLOC_ARENA_MAX=1 FLUXA_PERF_ITERATIONS="$iterations" "$binary"
fi
