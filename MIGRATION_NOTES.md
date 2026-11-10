# egui decouple and flex migration notes

Branch: claude/egui-decouple-ba98f7

## Phase A
- `native/fluxa-ui/src/gfx.rs` is `pub use egui::*;`. Every `egui::` in fluxa-ui now reads `crate::gfx::`. This is a facade, not yet newtype wrappers around Ui and Painter: swapping the backend means rewriting gfx.rs and anything that depends on egui-only methods.
- `scripts/check-egui-isolation.mjs` is wired into `check:structure`.
- Comments that mentioned egui were reworded to "the backend".
- fluxa-host, fluxa-renderer, fluxa-desktop untouched.

## Phase B
- flex.rs: CSS defaults (align stretch, shrink 1, grow 0, min-width auto), plus basis, margin, max sizes, align_self, Align::End, Justify::SpaceEvenly, `measured` and `wrapped_text`. Shrinking is weighted by shrink * base size and clamped by min-content.
- Behaviour change for every existing flex caller: default align was Start and is now Stretch; items no longer shrink below content unless `.min_width(0.0)` or similar is set. Existing screens were not audited for this. Expect overflow lint noise and stretched leaves.
- Only tiny slices were moved; the agents stopped early:
  - poster_overlay: sash and release-disc placement via `anchored` (edge sash is now clamped to the poster height).
  - settings/addons.rs: card header row and edit-dialog Cancel/Save row.
  - player.rs: brightness/volume gauge.
  - detail_modal.rs: horizontal_row via kit view.
  - kit/view.rs: outer placement.
  - home.rs: hero text stack as a flex column.
  - calendar.rs header, watch_together.rs title bars (new `title_bar` helper).
  - components: brand wordmark, seats row, recent_searches panel.

## Not migrated
- detail.rs, detail/episodes.rs: add_space stacks, hero/poster rect math, `.columns()`.
- settings.rs and the rest of settings/.
- player/controls.rs view containers, panel, sources, loading, stats, overlay.
- home/rows.rs headings, hero.rs, discover.rs (virtualized grid math).
- most of components/ (content, cards, buttons, menu, downloads, ticket, slider, controls).
- profiles, welcome, gallery, folder, flow: widgets allocate from the egui cursor, so flex needs those widgets rewritten first.
- kit::view children: immediate-mode closures, no retained child tree, so flex cannot place them.
- Left for last, only the egui path codemod applied: components/input.rs, kit/grid.rs, library.rs, layout.rs, fluxa-host, fluxa-android-renderer.

## Verification
- `cargo check -p fluxa-ui` has no errors; `cargo test -p fluxa-ui --lib` passes (168). Nothing was run visually and fixtures were not touched.
- Risky visually: calendar title/month text widths, addon card toggle vertical alignment, home hero spacing when logo/eyebrow/ratings are absent, kit view row vertical alignment in detail modal.

## Retained layout API (target for all screens)
Defined in `native/fluxa-ui/src/kit/items.rs`, re-exported from `kit`:
- `kit::item(size, |ui, rect| ...)`: a leaf with a known size and a paint closure called with its solved rect.
- `kit::widget(ui, |ui| ...)`: a leaf for an existing cursor-allocating widget. It is measured once in a sizing pass, then runs in a child ui at the solved rect. Use this instead of rewriting widgets.
- `kit::layout_items::{row, column, gap, space, fill}` build containers, fixed gaps and growing spacers.
- Item builders mirror flex: `.width .height .min_width .max_width .grow .shrink .basis .margin .padding .gap .align .align_self .justify .wrap`.
- `kit::lay(ui, items)` solves the items in the ui's available rect, using the ui's layout direction and item_spacing for the root, hugs the root's height, allocates it. `kit::lay_in(ui, bounds, root)` solves into an explicit rect. `kit::at(ui, rect, |ui| ...)` runs any existing widget code inside a rect.
- `view().row().gap(..).padding(..).items(ui, vec![...])` is the retained form of `View::show`: same chrome, fill, focus and stylesheet class, children placed by flex.
- Defaults are CSS: align stretch, shrink 1, grow 0, min-width auto. Use `.align(Align::Start)` for hugging cross size, `.shrink(0.0)` for fixed slots, `.min_width(0.0)` for truncating text.

Replacement map: `ui.horizontal`/`vertical`/`with_layout`/`columns` become `lay` or `view().items`; `add_space(x)` becomes `gap(x)` inside a container; `allocate_ui` becomes `at` plus a solved rect; `ui.cursor()` placement becomes `lay_in`.

## Ratchet
`scripts/check-layout-calls.mjs` counts structural layout calls per file and fails if a file's count exceeds `scripts/layout-calls-baseline.json`. Baseline at the start of the second pass: 467 calls in 87 files. `Rect::from_min_size` used to build solve bounds or clip rects is counted too, so some files cannot reach zero; list those in this file with a reason.

## Leftovers: settings
Gap calls (`add_space`, `ui.gap`) now go through `settings::page::vgap`, a one-gap `kit::lay`. `ui.cursor()` reads became `next_widget_position()`; the TV header button row is a flex row of `kit::widget` items.
- settings.rs: 5, `Rect::from_min_size` as solve bounds, flow area max_rect and page card background rect.
- settings/account.rs: 1, avatar image draw rect.
- settings/addons.rs: 7, logo and chevron image draw rects (3), chip stroke rects, toggle track, row hit rect and action button rects painted at computed x offsets inside the card painter.
- settings/badges.rs: 1, pill button rect centered in its already-solved slot.
- settings/compact.rs: 1, solve bounds.
- settings/page.rs: 2, screen rect and solve bounds.
- settings/row.rs: 1, solve bounds.
- settings/watch_party.rs: 1, solve bounds for the hint slot.

## Leftovers: components
All remaining calls are `Rect::from_min_size`/`from_center_size` for drawing geometry, solve bounds or clip rects; no cursor, spacing or ui.horizontal/vertical calls remain. surfaces.rs, dropdown.rs and the menu row now use `kit::view().items` / `kit::item`; streams action buttons use a flex row.
- avatar.rs: 2, circle image draw rects. glass.rs: 1, same. toggle.rs: 2, knob and track draw rects. icon_button.rs: 1, glyph draw rect.
- brand.rs: 1, QR module cell. buttons.rs: 2, progress fill and its clip rect. media.rs: 1, meter fill. slider.rs: 4, track, fill, base and thumb drawing.
- cards.rs: 9, popup panel and clip rect, preview anchor placement result, image draw rect, progress bars, labels solve bounds, `pinned` fallback.
- content.rs: 2, icon draw rect and spinner geometry. controls.rs: 2, animated segmented indicator and a test screen rect.
- downloads.rs: 6, three solve bounds, two progress fills, poster rect handed to `download_poster`.
- filmstrip.rs: 2, thumbnail draw rects computed per frame for the seek animation.
- keycap.rs: 2, key cap draw rects advanced along a painter row.
- menu.rs: 6, scrolled body rect, screen rect, panel solve bounds, icon draw rect, scrollbar thumb.
- mini_player.rs: 4, drag-positioned tile, progress fill, avatar draw rect.
- overlay.rs: 3, screen rects as solve bounds and a test screen.
- profile_tile.rs: 4, avatar disc, hit rect, badge and action button hit rects with reveal animation offsets.
- recent_searches.rs: 1, popup panel rect handed to `placed_front`. seats.rs: 2, solve bounds and seat dot.
- sticky_bar.rs: 2, strip rect and logo image draw rect. streams.rs: 2, icon draw rect and wrapped badge chip layout positions.
- subtitle_preview.rs: 1, caption line background rect. ticket.rs: 1, tear line geometry.

## Leftovers: kit
- kit/grid.rs, 4: not part of this pass, untouched.
- kit/flex.rs, 0 (skipped by the ratchet by name): the legacy closure Flex still builds child bounds with Rect::from_min_size around solve_with; superseded by kit::lay.
- kit.rs, 0: UiKit::gap uses allocate_exact_size and kit::center uses a top_down(Center) scope because their callers pass FnOnce closures that cannot be measured by kit::widget.
- layout_items::gap is a width-only slot, so it has no height inside a column root; use item(Vec2::new(0.0, px), ..) for vertical spacers until it is fixed.

## Leftovers: home and discover
- home.rs, 2: viewport screen rect used for clipping and hit tests, and the solve bounds for the hero text column (height is deliberately unbounded).
- home/hero.rs, 1: lane rect that bounds the fallback play pill solve. home/category_bar.rs, 1: lane rect that bounds the category bar solve.
- home/rows.rs, 1: scrolled row content rect (scroll offset applied) used as the card strip child ui bounds.
- discover.rs, 3: viewport screen rect, plus the band and strip solve bounds of the skeleton section.
- Vertical spacing in these files is a bottom or top margin on the neighbouring kit item, since layout_items::gap is width-only inside a column.

## Leftovers: player
- player.rs, 2: full-viewport screen rect used as the root bound; `Rect::from_center_size` hit rect around an already solved button center.
- player/controls.rs, 5: volume lane region bound and its animated drawn sub-rect, two solve bounds (header band, bottom actions row), and the pause glyph bar geometry.
- player/loading.rs, 4: solve bounds for the warning panel and the status rows, partial progress-bar fill, and the pulsing logo image rect (drawing geometry).
- player/overlay.rs, 2: badge chip strip solve bounds and the painted seek-bar segment stroke rect.
- player/panel.rs, 2: solve bounds for the virtualized option list (offset by scroll) and the chip height measurement.
- player/panels.rs, 1: logo image drawing rect at a y already solved by the block stack.
- player/sources.rs, 3: probe bounds for measuring uncached rows, the gradient fade strip, and the pinned chip strip solve bounds.
- player/stats.rs, 1: arrow icon image rect inside a hand-drawn row.

## Leftovers: overlays, chrome and misc
- watch_together.rs, 2: glyph image rect around a computed center, and the toggle track rect (drawing geometry).
- calendar.rs, 4: viewport screen rect, today marker and pill painter rects inside a day cell.
- folder.rs, 2: viewport screen rect and the poster rect inside a grid cell (painter geometry).
- update.rs, 2: badge hit/draw rect around the avatar center and the 11px status icon rect.
- ambient.rs, 4: viewport screen rect, centered backdrop rect, and two title logo image rects at a solved baseline.
- emoji.rs, 3: glyph cluster rects from galley positions and centered image rects (text painting geometry).
- interact.rs, 3: test-only screen_rect fixtures.
- navbar.rs, 3: lane rect bounding the pill solve, animated jelly pill rect, icon image rect.
- navigation.rs, 1: mobile nav bar rect (anchor geometry returned to callers).
- paint.rs, 2: solve bounds of the skeleton column and a centered spinner rect.
- poster_overlay.rs, 4 and poster_overlay/scores.rs, 1: painter-space mark, badge, logo and score bar rects.
- shorts.rs, 3: viewport screen rects and the solve bounds of the step buttons column.
- sidebar.rs, 5: panel bounds, solve bounds of slots, jelly highlight rect, progress fill rect.
- effects.rs, 1: rect inside a unit test.

## Known risk: kit::widget runs its closure twice
`kit::widget` measures in a sizing pass and then runs the closure again at the solved rect, so content inside is built twice per frame. Closures that push focus targets, activations, haptics or mutate state must guard with `ui.is_sizing_pass()`; calendar and folder do. Audit other `kit::widget` users for side effects.
