#!/usr/bin/env bash
# Fluxa P2P Benchmark
# Kullanım: ./bench.sh "<magnet-url>"
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
RUST_BIN="$REPO_ROOT/../fluxa-core/target/debug"

MAGNET="${1:-magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp://tracker.opentrackr.org:1337/announce&tr=udp://open.stealth.si:80/announce&tr=udp://tracker.openbittorrent.com:80/announce}"

TARGET_FIRST_FRAME=2500
TARGET_SEEK_BYTE=500
TARGET_SEEK_FRAME=2000

pass_fail() { (( $1 <= $2 )) && echo "✅" || echo "❌ (hedef <${2}ms)"; }

# ────────────────────────────────────────────────────────────────────────────
# measure <url> <file_size>
# Her ölçüm kendi timeout'una sahip; hata olursa "N/A" yazar.
# Sonuç stdout'a: "FB FF SB SF"
# ────────────────────────────────────────────────────────────────────────────
measure() {
  local url="$1" file_size="$2"
  local t0 first_byte first_frame seek_byte seek_frame

  # ── First byte ──────────────────────────────────────────────────
  t0=$(date +%s%3N)
  if curl -s --max-time 30 -r "0-0" "$url" -o /dev/null 2>/dev/null; then
    first_byte=$(( $(date +%s%3N) - t0 ))
  else
    first_byte="N/A"
  fi

  # ── First frame (30s hard limit) ────────────────────────────────
  t0=$(date +%s%3N)
  if timeout 30 ffmpeg -hide_banner -loglevel error \
      -i "$url" -frames:v 1 -an -f null /dev/null 2>/dev/null; then
    first_frame=$(( $(date +%s%3N) - t0 ))
  else
    first_frame="N/A"
  fi

  # ── Seek first byte (50% offset Range request) ──────────────────
  sleep 1
  local seek_pos=$(( file_size / 2 ))
  t0=$(date +%s%3N)
  if curl -s --max-time 30 -r "${seek_pos}-${seek_pos}" "$url" -o /dev/null 2>/dev/null; then
    seek_byte=$(( $(date +%s%3N) - t0 ))
  else
    seek_byte="N/A"
  fi

  # ── Seek first frame (30s hard limit) ───────────────────────────
  t0=$(date +%s%3N)
  if timeout 30 ffmpeg -hide_banner -loglevel error \
      -ss 600 -i "$url" -frames:v 1 -an -f null /dev/null 2>/dev/null; then
    seek_frame=$(( $(date +%s%3N) - t0 ))
  else
    seek_frame="N/A"
  fi

  echo "$first_byte $first_frame $seek_byte $seek_frame"
}

# ────────────────────────────────────────────────────────────────────────────
# run_engine <name> <result_file> <cmd> [args...]
#
# - Tüm display çıktısı doğrudan terminale (stdout) gider — $() içinde çağrılmaz.
# - Engine sonucu (5 sayı) $result_file'a yazılır.
# - READY satırını tüm stdout dosyasında arar (grep -m1), sadece ilk satıra bakmaz.
# - tail | sed pipeline'ı düzgün kapatılır.
# ────────────────────────────────────────────────────────────────────────────
run_engine() {
  local name="$1" result_file="$2"; shift 2
  local t0 url log_file url_file pid tail_pid

  url_file=$(mktemp)
  log_file=$(mktemp)
  t0=$(date +%s%3N)

  echo "━━━ $name ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  # engine stdout → url_file (READY satırı burada), stderr → log_file
  "$@" >"$url_file" 2>"$log_file" &
  pid=$!

  # log_file'ı canlı olarak terminale akıt
  { tail -f "$log_file" 2>/dev/null | sed "s/^/  /"; } &
  tail_pid=$!

  # READY satırını bütün dosyada ara (ilk satıra kilitlenme)
  local url=""
  while true; do
    local line
    line=$(grep -m1 "^READY " "$url_file" 2>/dev/null || true)
    if [[ "$line" == READY\ * ]]; then
      url="${line#READY }"
      break
    fi
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done

  local meta_ms=$(( $(date +%s%3N) - t0 ))

  # log tail'i durdur
  kill "$tail_pid" 2>/dev/null || true
  wait "$tail_pid" 2>/dev/null || true

  if [[ -z "$url" ]]; then
    echo "  ✗ READY satırı gelmedi / engine çöktü"
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    rm -f "$url_file" "$log_file"
    echo "N/A N/A N/A N/A N/A" >"$result_file"
    return
  fi

  # Content-Length ile dosya boyutunu al
  local file_size
  file_size=$(curl -sI "$url" 2>/dev/null \
    | grep -i "^content-length" | awk '{print $2}' | tr -d '\r' || true)
  [[ -z "$file_size" || "$file_size" == "0" ]] && file_size=1000000000

  echo ""
  echo "  Metadata: ${meta_ms}ms  →  $url  (dosya: $((file_size/1024/1024)) MB)"
  echo "  Ölçümler başlıyor…"
  echo ""

  # measure çıktısı capture edilir (FB FF SB SF — display değil, veri)
  local measures
  measures=$(measure "$url" "$file_size")

  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -f "$url_file" "$log_file"

  echo "$meta_ms $measures" >"$result_file"
}

# ────────────────────────────────────────────────────────────────────────────

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Fluxa P2P Benchmark                                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "  ${MAGNET:0:72}…"
echo ""

RQ_RESULT=$(mktemp)
WT_RESULT=$(mktemp)
trap 'rm -f "$RQ_RESULT" "$WT_RESULT"' EXIT

run_engine "rqbit (Fluxa)" "$RQ_RESULT" "$RUST_BIN/torrent_serve" "$MAGNET"
echo ""

run_engine "WebTorrent" "$WT_RESULT" node "$SCRIPT_DIR/webtorrent_serve.mjs" "$MAGNET"
echo ""

read -r RQ_META RQ_FB RQ_FF RQ_SB RQ_SF <"$RQ_RESULT" || true
read -r WT_META WT_FB WT_FF WT_SB WT_SF <"$WT_RESULT" || true

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SONUÇLAR                                                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
printf "  %-24s %-14s %-14s %s\n" "Metrik" "rqbit" "WebTorrent" "Hedef"
echo "  ──────────────────────────────────────────────────────────────"

row() {
  local label="$1" rq="$2" wt_raw="$3" target="${4:-}"
  local pf="" wt
  [[ "$wt_raw" =~ ^[0-9]+$ ]] && wt="${wt_raw}ms" || wt="$wt_raw"
  [[ -n "$target" && "$rq" =~ ^[0-9]+$ ]] && pf=$(pass_fail "$rq" "$target")
  printf "  %-24s %-14s %-14s %s\n" "$label" "${rq}ms" "${wt}" "$pf"
}

row "Metadata"       "$RQ_META" "$WT_META"
row "İlk byte"       "$RQ_FB"   "$WT_FB"   "1500"
row "İlk frame"      "$RQ_FF"   "$WT_FF"   "$TARGET_FIRST_FRAME"
row "Seek ilk byte"  "$RQ_SB"   "$WT_SB"   "$TARGET_SEEK_BYTE"
row "Seek ilk frame" "$RQ_SF"   "$WT_SF"   "$TARGET_SEEK_FRAME"
echo ""
