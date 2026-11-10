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
