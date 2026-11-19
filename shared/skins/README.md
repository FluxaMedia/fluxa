# Making and installing skins

A skin changes how Fluxa looks and how screens are arranged. It is data only: JSON layouts, a stylesheet, strings and constants. A skin never contains code and cannot reach the network or the filesystem. What the app can do (addons, trackers, sources) is not part of a skin.

Fluxa's own interface is the default skin in `shared/skins/default`. It is the best reference: every file below exists there and is written in exactly the format third-party skins use.

## Restyle the app without a skin

Settings, then Theme, has a Custom CSS box that opens with the complete stylesheet of the app (the default skin plus the active skin's theme). Find the rule you want, change it, and the app restyles as you type; nothing needs to be saved, built or restarted.

The edited text is stored on the device in `custom.css` in the app data directory and replaces the built-in stylesheet while it exists, so later app updates do not change a stylesheet you edited. Reset CSS deletes it and brings the original back. Invalid rules are skipped, the rest still apply.

## Quick start for full skins

1. Copy `shared/skins/default` to a new folder, for example `midnight/`.
2. Edit `manifest.json`: give it a new `id` and `name`.
3. Delete every screen you do not want to change from `screens` in the manifest (and the file). Screens a skin does not list fall back to the default skin, so a partial skin always works.
4. Change `style.css`, a screen, or both.
5. Start the installed Fluxa app with `FLUXA_SKIN` pointing at your folder. No build or clone is needed:

```bash
FLUXA_SKIN=/path/to/midnight fluxa
```

The folder is loaded directly and watched: save any file in it and the running app reloads the skin within about half a second. Load errors are printed to the terminal. When you are done, install the skin normally (see below).

## Folder layout

```
midnight/
  manifest.json
  style.css             theme stylesheet, layered over the default one
  constants.json        named numbers used from layouts
  includes.json         reusable layout fragments
  tokens.json           optional theme tokens (colors, sizes)
  screens/home.json     one file per replaced screen
  languages/en.json     optional string overrides, one file per language
```

Every file is limited to 1 MiB. Paths in the manifest must be relative and stay inside the skin folder.

## manifest.json

```json
{
  "schemaVersion": 1,
  "id": "midnight",
  "name": "Midnight",
  "version": "1.0.0",
  "author": "You",
  "forms": ["desktop", "mobile", "tv"],
  "canvas": { "tv": 960 },
  "constants": "constants.json",
  "theme": "style.css",
  "tokens": "tokens.json",
  "languages": { "en": "languages/en.json" },
  "screens": { "home": "screens/home.json" },
  "includes": ["includes.json"]
}
```

| Field | Meaning |
| --- | --- |
| `schemaVersion` | always `1` |
| `id` | letters, digits, `-` and `_`; must be unique among installed skins |
| `name`, `version`, `author` | shown in Settings; `author` is optional |
| `forms` | form factors the skin supports (`desktop`, `mobile`, `tv`); empty means all |
| `canvas` | logical width the skin is designed for per form factor, for example the TV layout at 960 |
| `constants` | JSON file of named numbers |
| `theme` | stylesheet file |
| `tokens` | JSON file overriding theme tokens, validated against `shared/contracts/theme-override.schema.json` |
| `languages` | map from language code to a JSON file of strings |
| `screens` | map from screen name to its layout file |
| `includes` | files of reusable fragments |

Unknown manifest fields are an error. All problems are collected and reported together with the file and node path.

## Screens

A screen file is one layout node. The screens a skin can replace are listed in `shared/contracts/skin-contract.json` under `screens`: `home`, `detail`, `library`, `discover`, `calendar`, `search`, `player`, `settings`, `profiles`, `folder`, `live`, `welcome`, `menu`, `mini`, `detail_modal`, `update`, `window`, `toast`, `tip`, `watch`, `dialog`, `sources`, `recommendations`.

`detail_modal` is shown on desktop only. Mobile and TV open the normal `detail` screen.

### Layout nodes

A node is exactly one of a primitive or component (`type`), an `include`, or an `each` list.

| Field | Meaning |
| --- | --- |
| `type` | `row`, `column`, `stack`, `scroll`, `spacer`, `text`, `page`, or a component: `image`, `icon`, `input`, `grid`, `pager`, `video`, `qr`, `fixed` |
| `when` | expression; the node is dropped when it is false |
| `form` | `["mobile", "desktop", "tv"]`; the node exists only on those form factors |
| `include`, `with` | insert a fragment from `includes.json`; `with` binds its parameters |
| `each`, `as`, `limit`, `template` | repeat `template` for every item of an array; `as` names the item (default `item`), `index` is always bound |
| `text`, `src`, `i18n` | strings with `{expression}` bindings, or a language key |
| `style`, `class` | named text styles and stylesheet classes |
| `props` | component properties, checked against the contract |
| `action` | `{ "name": "open_detail", "args": { "id": "{card.id}" } }` |
| `width`, `height`, `grow`, `gap`, `padding`, `align`, `justify`, `fill` | flex layout |
| `children` | only on `row`, `column`, `stack`, `scroll`, `fixed` |

Notes that save time:

- `padding` is `"top right bottom left"` or a single number.
- `width` and `height` accept numbers, `"100vw"`, `"100vh"`, and constants such as `"{const.modalWidth}"`.
- `scroll` is a horizontal strip; vertical scrolling uses `"type": "scroll", "props": { "axis": "y" }`.
- `row` is a flex row. Content that does not fit is clipped to the row and can be panned horizontally; it does not overflow its container. Add `"props": { "wrap": true }` to wrap onto new lines instead.
- A full-screen overlay (menu, modal) is a `fixed` root containing a `stack`; the first child is a clickable scrim, which is how tapping outside closes it.
- Text that must stay on one line takes `"props": { "lines": 1 }` and is cut with an ellipsis.
- Components and their props, the action list and the screen list are all in `shared/contracts/skin-contract.json`. Anything not in that file is a load error.

### Expressions

`a.b.c`, `rows[0]`, `'text'`, `1.5`, `true`, `false`, `null`, `!`, `&&`, `||`, `== != < <= > >=`, `len(x)`, parentheses. A missing path is `null`, never an error. A string that is exactly one binding keeps its type (`"{card.progress}"` stays a number); mixed text is a string.

### Data

A screen sees the JSON the core already produces for it, for example `home.title`, `home.rows`, `detail.episodes`, `calendar.cells`. The quickest way to learn the shape of a screen's data is to read the default skin's file for that screen and copy the bindings it uses.

### Actions

Actions are the only way a skin makes the app do something. The full list with argument names is `actions` in `skin-contract.json` (`play`, `open_detail`, `navigate`, `back`, `search`, `watchlist_toggle`, `set_setting`, `load_more`, `choose`, and so on).

### Includes

`includes.json` is one object whose keys are fragment names and whose values are nodes:

```json
{
  "title_logo": { "type": "image", "src": "{logo}", "height": 64 }
}
```

Use one with `{ "include": "title_logo", "with": { "logo": "player.logo" } }`. Include cycles hit the depth limit and fail.

### Limits

Depth 24, 12000 resolved nodes per screen, 200 items per list unless `limit` says otherwise, 1 MiB per file, 32 installed skins.

## Form factors

Desktop is the baseline. Differences for mobile and TV are stated with `form` on a node or with media rules in the stylesheet. The renderer still owns focus handling, overscan margins and minimum touch targets, so a skin cannot produce an unreachable control.

Follow the platform rules in `AGENTS.md` and `docs/design-language.md` (not tracked): mobile screens open as screens, not modals, TV is read from about 3 m with a D-pad, and desktop is dense with hover states.

## style.css

The stylesheet named by `theme` in the manifest is appended to the default one, so you only write what you change. Selectors are classes from your layouts, with optional state suffixes:

```css
.btn-primary {
  background: #ffffff;
  color: #000000;
  border-radius: 999px;
  transition: 0.15s;
}

.btn-primary:hover {
  background: ink/225;
}

.card.selected .card-title {
  font-weight: 700;
}
```

States: `:hover`, `:pressed`, `:focused` (or `:focus`), `:selected`, `:disabled`. A descendant selector (`.card.selected .card-title`) styles a node by its ancestor's state.

Properties in use: `background`, `color`, `font-size`, `font-weight`, `border`, `border-width`, `border-radius`, `padding`, `width`, `height`, `max-width`, `aspect-ratio`, `opacity`, `scale`, `box-shadow`, `backdrop-filter`, `transition`, `animation`, with `@keyframes` for animations.

Colors: `#rrggbb`, `rgba(r, g, b, a)`, or a theme name such as `background`, `theme.surface`, `accent`, `ink/225`. A theme name can be scaled with `*`: `background*0.5` is the background at half alpha.

Gradients: `linear-gradient(to bottom, background*0 0%, background*0.9 100%)`, with 2 to 16 color stops.

Media rules switch values per form factor: `@media (window: compact) { ... }`.

A rule that fails to parse is skipped and reported in the log (`style rule skipped: ...`); the rest of the stylesheet still applies. Start the app from a terminal with `FLUXA_SKIN` and watch the output while you work.

## constants.json and strings

`constants.json` is a flat object of numbers, read in layouts as `{const.name}`.

Language files are flat `"key": "text"` objects. A skin only needs the keys it overrides or adds; use them with `"i18n": "key"`. Fluxa's own translations live in `shared/skins/default/languages`.

## Installing a skin

In the app: Settings, then Theme, has a Skins section with a source field that takes any of:

- a GitHub repository URL, for example `https://github.com/owner/skin` (the default branch is downloaded; `/tree/<branch>` picks a branch),
- a direct `https://` link to a `.zip`,
- a local path to a skin folder or a `.zip` file.

Installing validates the skin and stores it. The same section lists installed skins with a switch for each; switching one on makes it the active skin, switching the active one off returns to the default skin.

On disk, installed skins live in the app data directory under `skins/`: a folder per skin, or `<id>.zip` for zip installs. Putting a skin there by hand works too, and the app finds it on the next start. The active skin id is stored in the file `active-skin` next to that folder.

| Platform | Data directory |
| --- | --- |
| Linux | `$XDG_DATA_HOME/com.fluxa.desktop/fluxa` (default `~/.local/share/com.fluxa.desktop/fluxa`) |
| macOS | `~/Library/Application Support/com.fluxa.desktop/fluxa` |
| Windows | `%APPDATA%\com.fluxa.desktop\fluxa` |

`FLUXA_DATA_DIR` overrides the location.

A zip needs `manifest.json` at its root or inside a single top-level folder, which is what a GitHub archive looks like.

## Publishing

Put the skin folder in its own repository. Users install it by pasting the repository URL into Settings. Fluxa does not host, index or recommend skins and runs no skin directory; share your repository link yourself.

## Checking your skin

- Run the app from a terminal with `FLUXA_SKIN` and read it for `skins/...` and `skin ...` messages. Load errors name the file and the node path.
- Check every form factor you list in `forms`: phone, desktop and TV sizes.
- The repository's `check:screens` renders reference screens at phone, desktop and TV sizes; form factors can be previewed in a repository checkout with the `FLUXA_PREVIEW_FORM` variable of `scripts/preview-desktop.sh`.
