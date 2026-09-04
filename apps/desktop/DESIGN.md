# Fluxa Desktop Design System

Every screen uses the same vocabulary. Reuse the existing tokens and primitives before adding a new visual value.

## Sources

- `src/design/tokens.ts` — colors, scale, spacing, typography, motion, and layers
- `src/design/primitives.tsx` — `Button`, `IconButton`, `Chip`, `SectionLabel`, `Divider`, and `MetaText`
- `src/index.css` — font faces and primitive interaction states

The naming mirrors Android's `FluxaColors` and `FluxaDimensions`. The platforms share the vocabulary, not the implementation.

## Rules

- Use semantic tokens instead of raw `#RRGGBB` or `rgba()` values. Exceptions are `src/design/` and `src/index.css`.
- Use `radius`, `fontSize`, and `space`; an isolated value such as `0.8438rem` needs a documented design reason.
- Use `heading()` for Archivo headings and `font.body` for Montserrat body text.
- Use semantic layer roles such as `z.overlay` and `z.dialog`; never invent large `zIndex` numbers.
- Extend a primitive instead of drawing a new styled `<button>`. Use `IconButton` for icon controls and `Chip` for selectable labels.

## Color roles

| Role | Use |
|---|---|
| `bg` | Application background |
| `bgElevated` | Detail/hero background and backdrop layer |
| `surface` / `surfaceRaised` | Cards and panels |
| `textPrimary` → `textFaint` | Text hierarchy |
| `line` / `lineStrong` | Dividers and borders |
| `fill` / `fillHover` / `fillActive` | Interactive surfaces |
| `accent` | Progress, selection, and state; not a background or button fill |

Primary actions use white buttons. Accent communicates state rather than decoration.

## Typography scale

`micro` 0.625 · `xs` 0.6875 · `sm` 0.75 · `base` 0.8125 · `md` 0.875 · `lg` 1 · `xl` 1.125 · `xxl` 1.375 · `h1` 2 · `hero` 3.125 rem

Use `base` for interface text. `lg` and above are headings.

## Check

```bash
npm run lint:design
```

The check detects raw colors, out-of-scale radius/font sizes, and arbitrary z-index values outside the design layer. Migrated files are listed in `scripts/check-design.mjs`.

## Migration targets

- [ ] `src/components/detail/**`
- [ ] `src/screens/{Home,Library,Search,Discover,CategoryGrid,Settings,Calendar}Screen.tsx`
- [ ] `src/screens/welcome/**`, `src/screens/Profile*`
- [ ] `src/components/player/**`
