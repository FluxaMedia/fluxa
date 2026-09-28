# Fluxa design language

One language, three layouts. Every platform shares the same tokens, components, content hierarchy and behavior, so a screen is recognizable everywhere. The arrangement is designed per form factor: desktop is not a stretched phone and TV is not a stretched desktop.

## Foundations

Tokens live in `shared/contracts/ui-tokens.json` and are read through `UiMetrics`. Screens use tokens, never literal sizes.

**Color.** Monochrome. Background `#060606`, raised surface `#141416`, card `#1C1C1F`. Text is white; secondary text is white at 70%, disabled at 38%. Borders are white at 8% and 16%. No accent colors and no decorative gradients.

**Type scale.** Six steps, used everywhere:

| Token | Phone | Desktop / TV | Weight | Use |
| --- | --- | --- | --- | --- |
| display | 28 | 44 | extra bold | fallback title when there is no logo |
| title | 20 | 26 | bold | page and section titles |
| subtitle | 16 | 18 | semibold | card titles, episode names, buttons |
| body | 14.5 | 16 | regular | synopsis, descriptions |
| meta | 13 | 14 | medium | facts line, dates, runtimes |
| label | 12 | 13 | medium | icon labels, badges, navigation |

**Spacing.** 4, 8, 12, 16, 24, 32, 48. Page gutter 16 on phone, 32 on tablet, 48 on desktop and TV.

**Radius.** 8 for posters and thumbnails, 12 for cards and inputs, full for buttons and chips.

**Motion.** 150 ms for press and hover, 250 ms for reveals, standard ease-out.

## Layout

The hierarchy is fixed on every platform: identity (logo, facts), primary action, secondary actions, description, then content. Only the arrangement changes.

**Phone.** A single column at full width minus the gutter. Artwork sits on top, and the column continues below it. It is built for the thumb: Play is full width, and the actions are labeled and evenly spread.

**Desktop.** Uses the width instead of scaling up.
- Artwork fills the window behind the page. The content column is left-aligned and capped at 560. A horizontal eased fade on the left keeps text readable.
- Secondary content sits beside the column rather than under it. Examples: episodes next to seasons, the cast and details panel next to the synopsis, a settings category list next to its rows.
- Hover reveals detail on cards (synopsis, progress, quick actions), so cards stay clean at rest.
- Denser grids and wider shelves, keyboard shortcuts, and a persistent side or top navigation instead of a bottom bar.

**TV.** Designed for the remote and for distance.
- Every screen has a single focus path, and the focused element is always obvious: scale plus a white ring.
- The type steps are larger and the gutters are wider. There is less text per screen, and no hover-only information.
- Shelves scroll horizontally, and the page scrolls vertically one shelf at a time.

Only the title logo and the facts line sit on artwork. Longer text sits on the background, or behind a fade that fully covers the artwork under it.

Artwork fades into the background with a multi-stop eased gradient. A linear fade leaves a visible edge.

## Components

Every element exists once in `fluxa-ui/src/components.rs`.

- **Primary button.** White pill, black label, centered icon and text, height 48 (52 on TV). One per screen.
- **Secondary button.** White 10% fill, white label, same shape and height as primary.
- **Labeled action.** Icon above label, no frame. When active, the icon gets a solid white circle.
- **Icon button.** 40 circle, white 10% fill. Used only for back, close and overflow.
- **Poster card.** 2:3, radius 8, title below the artwork, progress bar on the artwork edge.
- **Landscape card.** 16:9, radius 8, used for episodes and continue watching.
- **Chip.** Full radius, white 10% fill, white 100% fill when selected.
- **Section header.** Title token, optional "See all" in the meta token on the right.
- **Hero.** Artwork, eased fade, logo or display title, facts line, then the content column.

## Screens

Every screen follows hero or header, then the content column, then shelves.

- **Home.** Hero with logo, facts and primary and secondary buttons, then continue watching, then catalog shelves.
- **Detail.** Hero with logo and facts, then Play, the labeled actions and the expandable synopsis. Below that come episodes, cast and similar.
- **Library, Discover, Search.** Page title, chip row for filters, then a poster grid.
- **Calendar.** Page title, then a day list of landscape cards.
- **Settings and Profile.** Page title, then grouped rows on raised surfaces. On desktop the category list sits beside the rows.
- **Player.** Transport controls centered. The title and episode name use the title and meta tokens.
