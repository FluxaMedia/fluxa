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

## Status after the second pass
- Ratchet: 467 calls in 87 files at the start of the pass, 194 at the end (baseline file re-recorded at 194). Remaining calls are listed per area under the "Leftovers" headings above. `library.rs` (19), `layout.rs`, `kit/grid.rs` and `components/input.rs` were not migrated on purpose (other sessions have uncommitted edits there).
- Defaults audit: 96 pre-existing `flex::row`/`column` chains that did not set `.align(...)` were pinned to `Align::Start` to keep their old behaviour. min-width auto and shrink 1 were not audited per site; items that used to squeeze below their content now overflow unless given `.min_width(0.0)` or `.shrink(0.0)`.
- Tests: 169 pass. Fixed along the way: probe runs of `kit::widget` and `kit::lazy` are detected with `kit::probing()` (egui's `is_sizing_pass` is also true inside an Area's first frame); hugged `lay` height is no longer clamped to the viewport.
- Not verified visually. Things most likely to look wrong: settings addon dialog, watch-together panels (items are built before their data; closures run twice), calendar and discover headers, player sources/panel flows.

## Step 5: gfx owns its surface
- `gfx::Ui`, `Painter`, `Context`, `Response`, `InnerResponse`, `Frame`, `Area`, `CentralPanel`, `ScrollArea` (output `ScrollOutput`), `Popup`, `TextEdit` (output `TextEditOutput`) are repr(transparent) or owning newtypes with forwarding methods taken from call sites. Closures given to `Area`, `Frame`, `CentralPanel`, `ScrollArea`, `Popup`, `scope`, `scope_builder` receive `&mut gfx::Ui`.
- Input state, pointer, data, fonts and run output are wrapped too (`InputState`, `Pointer`, `Data`, `Fonts`, `RunOutput`); field reads became methods (`i.time()`, `i.stable_dt()`, `i.pointer()`). Layer shape surgery that used `graphics_mut` is now `Context::layer_cursor`, `transform_layer_since`, `fade_and_shift_layers`.
- Escape hatches (`raw`, `raw_mut`, `wrap`, `wrap_mut`, `from_egui`) are `pub(in crate::gfx)`, so the compiler rejects them outside gfx. `Context::from_backend` is the one public entry for hosts; `scripts/check-egui-isolation.mjs` fails on it (and the other hatch names) anywhere in fluxa-ui outside gfx.
- Still plain egui re-exports in gfx (pure values or widget builders): Rect, Pos2, Vec2, Color32, Id, FontId, Sense, Stroke, Shape, Mesh, Galley, Layout, UiBuilder, Key, Event, RawInput, Modifiers, Margin, CornerRadius, Shadow, RichText, Label, Button, Image, TextFormat, font definitions and a few enums. `Ui::add`/`put` still take `egui::Widget`, so Button/Label/Image stay egui widgets until they get gfx equivalents.
- `components/input.rs` was edited minimally to use the new `TextEdit`, `set_cursor`, `ui.has_focus`; it still needs its own cleanup pass (other sessions have uncommitted edits there). `library.rs`, `layout.rs` (one `stable_dt()` change only), `kit/grid.rs` untouched otherwise.
- Hosts are NOT migrated: `fluxa-host` (gpu.rs, frame.rs, detail_modal.rs, actions/remember.rs and others) passes `&egui::Context` where `&gfx::Context` is now expected and fails to compile. Each call needs `fluxa_ui::gfx::Context::from_backend(&ctx)`; `fluxa-renderer`, `fluxa-desktop` and `fluxa-android-renderer` depend on it and were not touched. `fluxa-trickplay-gen` already fails to build on its own (unrelated).
- Tests: `cargo test -p fluxa-ui --lib` passes (169). Nothing was run visually; the `gfx::Ui` pointer casts rely on `#[repr(transparent)]` over the egui type.

## Rect site classification

173 sites, line numbers are from before this pass. Classes: hit/clip 10, paint 26, placement 80, solve bound 57. Placement converted: 70, left: 10.

| site | class | converted | reason |
|---|---|---|---|
| ambient.rs:117 | paint | yes | UV crop rect; rect_around |
| ambient.rs:126 | solve bound | yes | viewport screen rect; rect_at |
| ambient.rs:166 | placement | yes | logo bottom-left in title lane, then translated by rise; flex::anchored |
| ambient.rs:179 | placement | yes | failed-logo placeholder same placement as the logo; shared logo_rect closure |
| calendar.rs:547 | placement | yes | today badge centered in the cell; flex::centered |
| calendar.rs:549 | placement | yes | today badge offset from cell top-left; anchored + translate |
| calendar.rs:585 | placement | yes | "+N" pill at cell top-right; anchored RIGHT_TOP + translate |
| calendar.rs:62 | solve bound | yes | viewport screen rect; rect_at |
| cards.rs:1128 | placement | yes | progress fill via flex::fraction |
| cards.rs:196 | hit/clip | yes | clip panel of animated height, via rect_at |
| cards.rs:210 | solve bound | yes | solve bound, via rect_at |
| cards.rs:300 | placement | yes | tooltip centered on anchor and kept inside screen via rect_around + new flex::kept_within |
| cards.rs:369 | paint | yes | artwork rect extended past card by radius (overflow); rect_at |
| cards.rs:686 | placement | yes | progress bar bottom-aligned in lane via flex::anchored |
| cards.rs:695 | placement | yes | progress fill via flex::fraction |
| cards.rs:831 | placement | yes | labels solve bound below card via flex::below |
| cards.rs:939 | paint | yes | unwrap_or fallback rect; rect_at |
| components/avatar.rs:48 | paint | yes | artwork square around circle center point; gfx::square_around |
| components/avatar.rs:81 | paint | yes | artwork square around circle center point; gfx::square_around |
| components/brand.rs:46 | paint | yes | QR module cell in dot-matrix; per-cell flex solve would be wasteful; gfx::rect_at |
| components/buttons.rs:506 | placement | yes | progress fill as fraction of track; flex::fraction |
| components/buttons.rs:548 | placement | yes | clip as fraction of pill rect; flex::fraction |
| components/controls.rs:391 | solve bound | yes | test screen rect; gfx::rect_at |
| components/controls.rs:85 | placement | yes | indicator is first slot translated by slot*position |
| components/filmstrip.rs:169 | placement | yes | selected frame is base scaled about center then shifted to keep bottom edge via flex::scaled + translate |
| components/filmstrip.rs:96 | placement | yes | thumbnails at computed x offsets now a flex row with gap |
| components/glass.rs:120 | paint | yes | dom image square around center point; gfx::square_around |
| components/icon_button.rs:126 | placement | yes | icon centered in button rect; flex::centered |
| components/input.rs:356 | placement | no | text row centered vertically in the field rect, full width |
| components/keycap.rs:199 | paint | yes | d-pad arm bars around a point offset; now gfx::rect_around |
| components/keycap.rs:81 | placement | yes | cap at running x, vertically centered in rect with depth offset; anchored plus translate |
| components/media.rs:72 | placement | yes | level fill as fraction of track; flex::fraction |
| components/mini_player.rs:136 | hit/clip | yes | stored mini-player rect for hit testing; gfx::rect_at |
| components/mini_player.rs:146 | solve bound | yes | tile rect handed to mini_player at area pos; gfx::rect_at |
| components/mini_player.rs:70 | placement | yes | buttons centered in tile at x/y offsets via flex::centered + translate |
| components/mini_player.rs:81 | placement | yes | progress fill is a width fraction of bar via flex::fraction |
| components/profile_tile.rs:257 | paint | yes | square around lock badge circle point, no parent box; gfx::square_around |
| components/profile_tile.rs:311 | placement | yes | action buttons offset from avatar slot center; flex::centered + translate |
| components/profile_tile.rs:66 | placement | yes | avatar disc centered in solved avatar slot; flex::centered |
| components/profile_tile.rs:67 | placement | yes | hit strip below avatar slot; flex::below |
| components/recent_searches.rs:38 | placement | yes | panel below anchor with gap; flex::below |
| components/seats.rs:25 | solve bound | yes | bounds the seat row solve; now gfx::rect_at |
| components/seats.rs:68 | placement | yes | plus icon at half the solved seat slot; flex::scaled(slot, 0.5) |
| components/slider.rs:42 | placement | yes | bar centered in track via flex::centered |
| components/slider.rs:44 | placement | yes | filled part is a width fraction of bar via flex::fraction |
| components/slider.rs:46 | placement | yes | base fill is a width fraction of bar via flex::fraction |
| components/slider.rs:86 | placement | yes | bubble centered over knob then translated up via flex::centered + translate |
| components/subtitle_preview.rs:80 | placement | yes | line box centered horizontally in area strip; now flex::centered |
| components/ticket.rs:238 | solve bound | yes | zero-height strip at tear used as anchored bound; gfx::rect_at |
| components/toggle.rs:15 | paint | yes | wordmark image around a bare point, no parent box; now gfx::rect_around |
| components/toggle.rs:62 | placement | yes | knob at one end of track; flex::centered(track) translated along x |
| content.rs:23 | paint | yes | icon around a circle center point, no box; square_around |
| content.rs:94 | placement | yes | icon horizontally centered in rect at running y via flex::centered |
| detail.rs:89 | solve bound | yes | viewport screen rect; rect_at |
| detail_modal.rs:182 | solve bound | yes | viewport rect, via rect_at |
| detail_modal.rs:208 | placement | yes | panel centered in screen via flex::centered |
| detail_modal.rs:255 | paint | yes | artwork extended past image by radius (overflow); rect_at |
| detail_modal.rs:270 | paint | yes | trailer image extended past image by radius (overflow); rect_at |
| detail_modal.rs:289 | placement | yes | strip below image via flex::below |
| detail_modal.rs:433 | placement | yes | icon centered in button via flex::centered |
| detail_modal.rs:844 | placement | yes | poster anchored left-top in allocated rect via flex::anchored |
| discover.rs:32 | solve bound | yes | viewport screen rect; rect_at |
| discover.rs:726 | solve bound | yes | skeleton header band solved as flex container; rect_at |
| discover.rs:753 | solve bound | yes | skeleton strip solve bound; rect_at |
| downloads.rs:412 | placement | yes | progress fill via flex::fraction |
| downloads.rs:581 | placement | yes | progress fill via flex::fraction |
| downloads.rs:59 | solve bound | yes | strip solve bound, via rect_at |
| downloads.rs:729 | placement | yes | progress fill via flex::fraction |
| downloads.rs:904 | solve bound | yes | heading row solve bound, via rect_at |
| downloads.rs:973 | placement | yes | poster anchored left-top in grid cell via flex::anchored |
| effects.rs:348 | paint | yes | test fixture geometry; gfx::rect_at |
| emoji.rs:125 | paint | yes | glyph cluster box from font metrics; gfx::rect_at |
| emoji.rs:141 | placement | yes | emoji scaled about slot center; flex::scaled |
| emoji.rs:148 | placement | yes | texture fitted and centered in rect; flex::centered |
| folder.rs:210 | placement | yes | poster at top-left of the card cell; anchored LEFT_TOP |
| folder.rs:90 | solve bound | yes | viewport screen rect; rect_at |
| home.rs:199 | solve bound | yes | viewport screen rect; rect_at |
| home.rs:703 | solve bound | yes | hero column solve bound; rect_at |
| home/category_bar.rs:53 | solve bound | yes | lane the bar is anchored in; rect_at |
| home/hero.rs:117 | solve bound | yes | lane for the play button stack; rect_at |
| home/rows.rs:357 | solve bound | yes | max_rect of child Ui; rect_at |
| interact.rs:152 | hit/clip | yes | test screen rect; gfx::rect_at |
| interact.rs:168 | hit/clip | yes | test screen rect; gfx::rect_at |
| interact.rs:230 | hit/clip | yes | test screen rect; gfx::rect_at |
| kit/flex.rs:117 | hit/clip | yes | child ui max_rect from solved slot; gfx::rect_at |
| kit/flex.rs:161 | hit/clip | yes | test screen rect; gfx::rect_at |
| kit/grid.rs:194 | placement | no | grid cell at computed row/column offset (with centered last row lead) in area |
| kit/grid.rs:219 | solve bound | no | test screen_rect at ZERO with fixed viewport size |
| kit/grid.rs:41 | placement | no | Lattice cell at row/column offset from origin |
| layout.rs:154 | placement | no | card cell at index offset from origin (grid position) |
| layout.rs:411 | solve bound | no | viewport rect at ZERO for screen layout |
| layout.rs:59 | placement | no | compact search box below title at margin, full width |
| layout.rs:70 | placement | no | search box right-aligned and vertically centered against title row |
| library.rs:14 | solve bound | no | screen rect at ZERO from viewport |
| library.rs:185 | solve bound | no | empty state region from ui cursor origin, width and min height |
| library.rs:230 | solve bound | no | region origin plus width passed as child max_rect for download status |
| library.rs:457 | placement | no | poster rect at card min with poster height inside card rect |
| menu.rs:130 | solve bound | yes | viewport rect, via rect_at |
| menu.rs:242 | placement | no | popover flip plus lower-edge clamp only; kept_within needs both bounds, rect_at kept |
| menu.rs:415 | solve bound | yes | viewport rect, via rect_at |
| menu.rs:593 | placement | no | icon may be larger than its slot; centered would clamp it, square_around kept |
| menu.rs:621 | placement | yes | scrollbar thumb anchored at rail top plus travel via anchored+translate |
| menu.rs:96 | placement | no | scrolled content taller than viewport; flex stack clamps oversize child, so rect_at kept |
| navbar.rs:281 | paint | yes | animated jelly pill; width can exceed inner and stack would shrink it, so left geometric; rect_around |
| navbar.rs:408 | paint | yes | icon image at a center point; square_around |
| navbar.rs:66 | solve bound | yes | lane the bar is anchored in; rect_at |
| navigation.rs:245 | placement | yes | mobile nav bar centered at bottom of viewport; anchored CENTER_BOTTOM in a bounds rect ending at the bottom edge |
| overlay.rs:114 | solve bound | yes | viewport rect for scrim, via rect_at |
| overlay.rs:213 | solve bound | yes | test input screen rect, via rect_at |
| overlay.rs:42 | solve bound | yes | viewport rect for scrim, via rect_at |
| paint.rs:329 | solve bound | yes | column solve bound; gfx::rect_at |
| paint.rs:494 | paint | yes | circle texture quad around a point; gfx::square_around |
| player.rs:301 | solve bound | yes | full viewport rect; gfx::rect_at |
| player.rs:551 | hit/clip | yes | button/icon rect around a bare center point param (no parent box available); gfx::square_around |
| player/controls.rs:111 | placement | yes | drawn-width box is a width fraction of home; flex::fraction(home, amount) |
| player/controls.rs:252 | solve bound | yes | container for tree.solve; gfx::rect_at |
| player/controls.rs:341 | solve bound | yes | container for tree.solve; gfx::rect_at |
| player/controls.rs:890 | paint | yes | pause glyph bars geometry; gfx::rect_around |
| player/controls.rs:95 | solve bound | yes | volume home box at cursor origin; now gfx::rect_at |
| player/loading.rs:193 | placement | yes | pulsing logo is logo_rect scaled about center via flex::scaled |
| player/loading.rs:281 | solve bound | yes | rows.solve container; gfx::rect_at |
| player/loading.rs:56 | solve bound | yes | panel.solve container; gfx::rect_at |
| player/loading.rs:64 | placement | yes | progress fill is a height fraction of the bar; added flex::fraction_height |
| player/overlay.rs:155 | placement | yes | bar segment vertically centered in track at x offset via flex::anchored + translate |
| player/overlay.rs:414 | solve bound | yes | strip.solve container (origin derived from track, no above helper, overflow risk in anchored); gfx::rect_at |
| player/panel.rs:1098 | solve bound | yes | list.solve container; gfx::rect_at |
| player/panel.rs:1244 | solve bound | yes | measuring solve container; gfx::rect_at |
| player/panels.rs:150 | paint | yes | logo image painted at (left,y) in a deferred block closure; gfx::rect_at |
| player/sources.rs:176 | solve bound | yes | max_rect for sizing probe; gfx::rect_at |
| player/sources.rs:293 | placement | yes | fade rect directly below strip via flex::below |
| player/sources.rs:436 | solve bound | yes | row.solve container; gfx::rect_at |
| player/stats.rs:208 | paint | yes | arrow icon image at line-relative coordinates, no parent box; gfx::rect_at |
| poster_overlay.rs:351 | paint | yes | logo image from a bare left-center point (draw_logo signature shared with meta.rs, not my file); rect_at |
| poster_overlay.rs:478 | placement | yes | watched mark in a flex row at poster top-left corner |
| poster_overlay.rs:482 | placement | yes | saved mark in the same flex row |
| poster_overlay.rs:501 | placement | yes | age badge in the same row, centered on the mark height (row fixed to mark height) |
| poster_overlay/scores.rs:114 | placement | yes | fill pill extended past the score fill; now from_min_max off fill |
| settings.rs:1218 | solve bound | yes | bound handed to page::at at an absolute origin; now gfx::rect_at |
| settings.rs:1268 | solve bound | yes | container the card stack is solved in; now gfx::rect_at |
| settings.rs:356 | paint | yes | card background geometry from an origin and computed size; now gfx::rect_at |
| settings.rs:369 | solve bound | yes | container the row column is solved in; now gfx::rect_at |
| settings.rs:502 | solve bound | yes | max_rect of the flow child ui; now gfx::rect_at |
| settings/account.rs:485 | placement | yes | avatar left-centered in header; anchored(LEFT_CENTER) plus S20 translate |
| settings/addons.rs:1003 | placement | yes | logo image centered in solved logo slot; flex::centered |
| settings/addons.rs:1010 | placement | yes | icon centered in solved logo slot; flex::centered |
| settings/addons.rs:1106 | placement | yes | chip at running x offset in row; anchored(LEFT_TOP) translated |
| settings/addons.rs:1128 | placement | yes | toggle track left-centered vertically in row; anchored(LEFT_CENTER) |
| settings/addons.rs:1138 | hit/clip | yes | focus hit rect only; now gfx::rect_at |
| settings/addons.rs:1195 | placement | yes | chevron right-centered in row with inset; anchored(RIGHT_CENTER) translated |
| settings/addons.rs:1227 | placement | yes | action buttons at computed x offsets in row; anchored(LEFT_TOP) translated |
| settings/addons.rs:1591 | placement | yes | buttons rect bottom-left aligned in buttons_rect; anchored(LEFT_BOTTOM) |
| settings/badges.rs:152 | placement | yes | pill button centered in solved slot; flex::centered |
| settings/compact.rs:111 | solve bound | yes | container the nav column is solved in; now gfx::rect_at |
| settings/page.rs:373 | solve bound | yes | list solve bound from outer_size; now gfx::rect_at |
| settings/page.rs:61 | solve bound | yes | screen/viewport rect; now gfx::rect_at |
| settings/row.rs:152 | solve bound | yes | container the single-row column is solved in; now gfx::rect_at |
| settings/watch_party.rs:60 | solve bound | yes | bounds the padded() flex solve for hint text; now gfx::rect_at |
| shorts.rs:248 | solve bound | yes | step buttons column solve bound; rect_at |
| shorts.rs:36 | solve bound | yes | viewport screen rect anchored into; rect_at |
| shorts.rs:92 | solve bound | yes | viewport screen rect; rect_at |
| sidebar.rs:129 | solve bound | yes | panel bounds from viewport; rect_at |
| sidebar.rs:248 | paint | yes | icon image at a bare center point; square_around |
| sidebar.rs:347 | placement | yes | indicator is slot rect shifted to animated y; translate |
| sidebar.rs:359 | solve bound | yes | container solved for the indicator bar; rect_at |
| sidebar.rs:633 | placement | yes | progress fill as width fraction of track; flex::fraction |
| sticky_bar.rs:133 | placement | yes | logo left-aligned and vertically centered in bar band via flex::anchored |
| sticky_bar.rs:53 | hit/clip | yes | full-width block strip for interact, via rect_at |
| streams.rs:194 | placement | yes | wrapping badge flow replaced by flex row wrap with gaps |
| streams.rs:89 | placement | yes | icon left-center in chip with offset via anchored+translate |
| update.rs:104 | paint | yes | icon image at badge center; square_around |
| update.rs:85 | placement | yes | badge square offset from avatar center; square_around (point-relative, no box) |
| watch_together.rs:100 | paint | yes | icon image at a center point; square_around |
| watch_together.rs:1042 | placement | yes | toggle track at right of the row; anchored RIGHT_TOP + translate |

## Rect pass summary
- Ratchet: `check-layout-calls.mjs` no longer counts bare `Rect::from_*`; it counts layout calls plus rect constructions (`Rect::from_min_size/from_center_size`, `gfx::rect_at/rect_around/square_around`) whose arguments read another box's geometry. Baseline re-recorded at 63 in 27 files (was 194 under the old counting).
- New helpers: `flex::centered`, `fraction`, `fraction_height`, `scaled`, `below`, `kept_within` (next to the existing `anchored`), and `gfx::rect_at`, `rect_around`, `square_around` for constructions that stay.
- Not converted placement sites: `components/input.rs`, `kit/grid.rs`, `layout.rs`, `library.rs` (other sessions' files, listed in the table), plus three in `components/menu.rs` where the content is taller than the viewport, the popover flips at the edge, or the icon exceeds its slot.
- Hosts: touched `fluxa-host/src/{gpu,frame,trailer,episode_details,input,layout,scroll,poster_data,hero_badges,detail_modal}.rs` and `actions/remember.rs` (each `&egui::Context` argument wrapped in `fluxa_ui::gfx::Context::from_backend`), plus `fluxa-ui/src/lib.rs` (`gfx` is now `pub mod`). `fluxa-renderer` and `fluxa-desktop` needed no edits and build.
- Risk: converted sites go through `anchored`/`centered`, which solve a flex stack and clamp a child that is larger than its bounds; they match the old rects only while the child fits. Nothing was run visually.
