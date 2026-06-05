#!/usr/bin/env bash
# Fluxa P2P Benchmark: rqbit vs WebTorrent
# Usage: ./run_bench.sh ["<magnet-url>"] [--bytes N]
# Defaults to Big Buck Bunny (public domain) if no magnet given.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$REPO_ROOT/../fluxa-core"

MAGNET="${1:-magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp://tracker.opentrackr.org:1337/announce&tr=udp://open.stealth.si:80/announce&tr=udp://tracker.openbittorrent.com:80/announce}"
BYTES="${3:-262144}"

echo "============================================================"
echo "  Fluxa P2P Engine Benchmark"
echo "  Magnet: ${MAGNET:0:80}…"
echo "  Target: $((BYTES / 1024)) KB"
echo "============================================================"
echo ""

# Build rqbit bench binary
echo ">>> Building rqbit bench binary…"
(cd "$RUST_DIR" && cargo build --bin torrent_bench --quiet)
RQBIT_BIN="$RUST_DIR/target/debug/torrent_bench"

# ---- Run WebTorrent ----
echo ""
echo "━━━ [1/2] WebTorrent (tiny-stremio-player engine) ━━━━━━━━"
WT_RESULT=$( node "$SCRIPT_DIR/webtorrent_bench.mjs" "$MAGNET" --bytes "$BYTES" 2>/dev/tty )

# ---- Run rqbit ----
echo ""
echo "━━━ [2/2] rqbit (Fluxa engine) ━━━━━━━━━━━━━━━━━━━━━━━━━━━"
RQ_RESULT=$( "$RQBIT_BIN" "$MAGNET" --bytes "$BYTES" 2>/dev/tty )

# ---- Parse & compare ----
echo ""
echo "============================================================"
echo "  RESULTS"
echo "============================================================"

node - "$WT_RESULT" "$RQ_RESULT" <<'EOF'
const wt = JSON.parse(process.argv[1]);
const rq = JSON.parse(process.argv[2]);

function row(label, wtVal, rqVal, unit = "ms") {
  const wtStr = wtVal != null ? `${wtVal}${unit}` : "N/A";
  const rqStr = rqVal != null ? `${rqVal}${unit}` : "N/A";
  let diff = "";
  if (wtVal != null && rqVal != null) {
    const delta = rqVal - wtVal;
    diff = delta > 0 ? `rqbit slower by ${delta}${unit}` : delta < 0 ? `rqbit faster by ${-delta}${unit}` : "tie";
  }
  const pad = (s, n) => String(s).padEnd(n);
  console.log(`  ${pad(label, 26)} ${pad(wtStr, 12)} ${pad(rqStr, 12)}  ${diff}`);
}

console.log(`\n  ${"Metric".padEnd(26)} ${"WebTorrent".padEnd(12)} ${"rqbit".padEnd(12)}  Delta`);
console.log("  " + "─".repeat(72));
row("Engine startup",        wt.engineStartupMs,  rq.engineStartupMs);
row("Metadata ready",        wt.metadataMs,       rq.metadataMs);
row("First byte (from t=0)", wt.firstByteMs,      rq.firstByteMs);
row("Read done (from req)",  wt.readDoneMs,       rq.readDoneMs);
row("Bytes received",        wt.bytesRead,        rq.bytesRead, "B");

console.log("\n  Files:");
console.log(`    WebTorrent → ${wt.fileName}`);
console.log(`    rqbit      → ${rq.fileName}`);

const wtTotal = wt.firstByteMs ?? wt.readDoneMs + (wt.metadataMs ?? 0);
const rqTotal = rq.firstByteMs ?? rq.readDoneMs + (rq.metadataMs ?? 0);

console.log("\n  ── Summary ──────────────────────────────────────────────");
if (wtTotal != null && rqTotal != null) {
  const winner = wtTotal < rqTotal ? "WebTorrent" : rqTotal < wtTotal ? "rqbit" : "Tie";
  const diff = Math.abs(wtTotal - rqTotal);
  console.log(`  First-byte winner: ${winner} (by ${diff}ms)`);
  if (diff < 1000) {
    console.log("  → Difference is < 1s: engines are roughly equivalent.");
    console.log("    Perceived gap in Fluxa is likely Android service overhead.");
  } else {
    console.log("  → Significant gap: engine-level difference.");
  }
}
console.log("");
EOF
