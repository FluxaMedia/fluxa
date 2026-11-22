# fluxa-ui

Layout, skin and paint for every Fluxa shell. Screens are JSON, style is `shared/skins/default/style.css`, and a frame is: resolve screen to nodes, apply styles, solve layout (`flex.rs`), run paint closures.

## Engine design note

Status: baseline measured; phase B results below; phase C waits for approval.

### How to measure

Counters live in `src/stats.rs` and the host frame log; both are off unless `FLUXA_FRAME_STATS=1`.

- `FLUXA_FRAME_BENCH=1` forces a redraw every frame, starting 8 s after launch, so the numbers are steady-state cost and not idle.
- Build `fluxa-desktop` in release with `CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true` (the `FLUXA_PREVIEW*` routes need debug assertions) and `--features alloc-stats` for allocation counts.
- Run it under `xvfb-run` with `FLUXA_PREVIEW=<route> FLUXA_PREVIEW_FIXTURE=1 FLUXA_PREVIEW_SIZE=WxH`, wait about 24 s and read the 5 s windows: `Skin per frame`, `Draw per frame`, `Frame stats`.
- `cargo test -p fluxa-ui --lib solve_cost_by_depth -- --ignored --nocapture` with `FLUXA_FRAME_STATS=1` prints layout cost against tree depth.

Caveats: software GPU under xvfb (so `draw` is CPU tessellation and upload, not a real GPU), forced redraw (a real session is mostly idle), allocation counts include every thread, fixtures not live data.

### Baseline

Per frame, forced redraw, release, 1280x800 unless noted. Times are means over the last 5 s window; `total` is the frame p50 / p95.

| scene | nodes | style lookups | solves | span calls | outer_size calls | build ms | style ms | layout ms | jobs | vertices | allocs | total ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| settings | 179 | 557 | 9 | 370 | 4350 | 0.93 | 0.34 | 0.23 | 37 | 5480 | 10.2k | 2.2 / 4.0 |
| player panel | 133 | 478 | 8 | 242 | 2644 | 0.56 | 0.25 | 0.11 | 33 | 3438 | 7.6k | 1.3 / 1.6 |
| player | 101 | 370 | 2 | 170 | 1866 | 0.46 | 0.22 | 0.09 | 19 | 2662 | 6.5k | 1.1 / 2.1 |
| home | 148 | 437 | 24 | 254 | 1148 | 0.29 | 0.21 | 0.08 | 17 | 3123 | 6.2k | 1.5 / 3.3 |
| library | 53 | 175 | 9 | 120 | 800 | 0.48 | 0.15 | 0.09 | 18 | 2480 | 3.4k | 1.4 / 2.6 |
| tv home 1920x1080 | 120 | 352 | 22 | 226 | 1029 | 0.17 | 0.12 | 0.05 | 15 | 2522 | 5.1k | 1.0 / 1.2 |
| phone home 390x844 | 83 | 240 | 17 | 192 | 946 | 0.16 | 0.08 | 0.04 | 17 | 1996 | 4.0k | 0.8 / 0.9 |
| detail | 50 | 135 | 5 | 94 | 665 | 0.18 | 0.05 | 0.04 | 18 | 1342 | 2.8k | 0.6 / 1.1 |
| discover | 42 | 146 | 9 | 98 | 603 | 0.19 | 0.06 | 0.03 | 17 | 2348 | 2.5k | 0.5 / 0.7 |

Worst single frames seen: settings 10.3 ms, home 8.9 ms.

### Findings

1. Paint dominates. Of a 2.2 ms settings frame, build is 0.9 ms and layout 0.2 ms; the rest is paint closures, tessellation and upload (the `draw` column of the host log is 85-90 % of `work`). Layout is not the bottleneck on any screen today.
2. `outer_size` is called 5-12 times per `span` call and 100-500 times per solve. In the synthetic nested tree its calls per node grow linearly with depth (about 4 at depth 3, 24 at depth 9), so `span` re-measures subtrees: cost is O(nodes x depth), not O(nodes). Absolute cost is still small (0.65 ms for 1023 nodes at depth 9), but deep wrap/percent layouts will make it visible.
3. Nothing is cached across frames at the style level. Every frame does 135-557 `sheet_style` lookups and rebuilds 42-179 nodes even when nothing changed. The class-parts cache works (0 misses), so the repeated cost is the lookup and merge itself, 0.05-0.44 ms.
4. 2.5k-10k allocations (0.4-1.6 MB) per frame, roughly 55 per node. Candidates: per-node `Vec` of children, `String` class keys, closure boxes.
5. The card shape-replay cache (`skin/cache.rs`) reports 0 hits and tens of thousands of misses per 5 s window in every fixture scene that has cards (home 36k, phone home 36k, tv home 51k). Either fixtures keep a card permanently "dirty" (placeholder poster, no texture) or the signature changes every frame. Unknown until investigated; if it is the fixtures, real data would hit and the benchmark must load real textures.
6. Draw calls are already low (15-37 paint jobs per frame) and the paint-job fingerprint skips unchanged frames, so batching is not the first problem; vertex counts (up to 5.5k) come from text and rounded rects.
7. Layout and build run more than once per frame (`solves` 2-42; the gallery runs 42). Some are legitimate (panels, scroll regions), some are likely the same tree laid out at several candidate sizes.

### Proposed order

B. Fix what the numbers show, one commit and one before/after table each, behaviour identical:

1. Explain the card-cache 0 % hit rate (fixtures or real bug) and make the benchmark representative. Without this, B3 cannot be judged.
2. Memoise `outer_size` per node for the current available size inside one solve (invalidate on `solve` entry). Target: `outer_size` calls per solve fall to about one per node.
3. Resolved-style cache keyed by (class set, state, media epoch) so unchanged nodes skip lookup and merge.
4. Dirty tracking: skip build and layout when the screen, size and data generation are unchanged and no animation is running; keep the existing paint fingerprint.
5. Allocation pass on the hottest sites found by the counters (children vectors, class keys), only where a count drops measurably.
6. Off-screen skipping for scroll lists (virtualise rows and cards outside the clip plus a margin). Largest win for library and settings, riskiest for focus; last.

C. Layout features, ordered by call sites in the skin today:

1. `calc()`, `min()`, `max()`, `clamp()` in lengths. Removes the `100vw` and `100vh` stand-ins (about 9 rules) and the remaining fixed clamps.
2. Absolute positioning with `inset` and `place-self` / `place-items`. Replaces the 15 `fixed` nodes and the `.layer` wrappers.
3. `flex` shorthand and `align-self`.
4. A real grid: `grid-template-columns` with `repeat`, `auto-fill`, `minmax`, `fr`, and spans. Replaces `column-width` / `columns` for the poster grids and the 10 `grid` nodes.
5. Container queries and `cqw`, once the sizing model above is in; today `@media (size)` covers the screens.

Quality, alongside both: invariant tests (children stay inside the parent unless positioned, layout is idempotent, no NaN or negative sizes) run over the screen gallery; keep `FLUXA_LAYOUT_LINT`; a bench script in `scripts/` so every B commit reports the same table.

Already done (percent sizing, stack `align` and `place`, aspect ratio from one side, wrap-row height) is covered by tests in `skin/tests.rs` and `flex.rs`.

### Phase B results

Best of several interleaved runs (machine shared with other builds, so single runs vary 2-5x; compare within a row, not across commits). Work ms per frame, p50 / p95, art fixture unless noted:

| scene | before | after | outer_size calls | allocs per frame |
|---|---|---|---|---|
| home (no artwork, plain fixture) | 1.10 / 1.30 | 0.60 / 0.80 | 1148 -> 473 | 6170 -> 3028 |
| phone home (plain fixture) | 0.70 / 0.80 | 0.50 / 0.60 | 946 -> 413 | 4001 -> 2402 |
| home | 0.80 / 0.95 | 0.70 / 0.85 | 973 -> 479 | 4404 -> 3501 |
| library | 0.60 / 0.65 | 0.50 / 0.60 | 800 -> 282 | 3681 -> 2690 |
| detail | 0.50 / 0.55 | 0.40 / 0.50 | 665 -> 241 | 3091 -> 2338 |
| player | 0.90 / 1.00 | 0.70 / 0.85 | 1866 -> 479 | 6599 -> 4373 |
| settings | 1.40 / 1.70 | 1.20 / 1.40 | 4350 -> 910 | 10475 -> 5865 |

What each commit did:

- Cards without an artwork url were marked dirty on every frame, so the card and strip replay caches never stored them. They now replay: plain-fixture home 1.10 -> 0.70 ms, 6170 -> 3927 allocations.
- Node sizes are memoised for the length of one solve. Text measuring per nested tree: 24 calls -> 1. Settings layout 0.14 -> 0.04 ms.
- Resolved styles are memoised per node for one frame. Style time roughly halves (settings 0.22 -> 0.12 ms).
- The focus frame is updated in place instead of copied for every registered target (quadratic). 40-frame heap profile: 89.9 MB -> 70.6 MB.
- Token lookups no longer clone the value string; flex children are measured without collecting sizes. Blocks per 40 frames: 334k -> 255k in total across the three allocation commits.

Not done, with reasons:

- Dirty tracking. A pointer sweep over home produces about 78 % frames with unchanged output (51 presented of 227), each paying the full build and paint. Skipping them safely needs a hover-membership signature across the whole host frame (rendered layout, focus ring, cursor icon, hover tip, seek preview), so it is a design decision and not a drop-in cache.
- Off-screen skipping. Horizontal strips (far cards) and grids (windows) already virtualise: a library of 6 items, 600 items and 6000 items all build 139 nodes per frame at 1.4-1.5 ms.

Verified: 45 of 47 `check-screens` scenes are pixel-identical to baselines regenerated from the commit before phase B; the two player-sources scenes differ by 3-2055 px between runs of the same code. Layout lint counts are identical. Not verified: real GPU, real network artwork, live data, a real device.
