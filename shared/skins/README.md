# Making and installing skins

## Write your first skin in one file

A skin can be one `.css` file with one small script beside it. This one adds a Notes page to the navigation bar; ticking a note is remembered between launches. The finished files are in `shared/skins/examples/one-file`.

`one-file.css`:

```css
@skin {
  id: one-file;
  name: "One File";
  version: 1.0.0;
  script: "notes.js";
  script-id: notes;
  permissions: storage;
  refresh: 30;
}

.notes-title { padding: 24px; }

.notes-row {
  padding: 12px 20px;
  background: #1f3a5f;
  border-radius: 12px;
}

@page notes { nav-label: "Notes"; nav-icon: NavLibrary; }

@element notes-title {
  parent: .page-notes;
  type: text;
  style: title;
  text: "{ext.notes.title}";
}

@element notes-row {
  parent: .page-notes;
  each: ext.notes.items;
  text: "{item.label}";
  action: ext.notes.toggle(index: item.index);
}
```

`notes.js`:

```js
class Provider {
  async data() {
    const done = storage.get("done", []);
    const items = ["Write a skin", "Share the link", "Watch it update"];
    return {
      title: "Notes (" + done.length + "/" + items.length + ")",
      items: items.map((text, index) => ({
        label: (done.includes(index) ? "[x] " : "[ ] ") + text,
        index,
      })),
    };
  }

  async toggle(args) {
    const done = new Set(storage.get("done", []));
    if (done.has(args.index)) done.delete(args.index);
    else done.add(args.index);
    storage.set("done", [...done]);
  }
}
```

1. Put both files in one folder and serve it from anywhere: a static host, a GitHub raw link, or `python3 -m http.server` while you try it out.
2. In Fluxa open Settings, Theme, Skins, paste the link of the `.css` file and press Install. The script is fetched from the same place (`script` is relative to the CSS link). The skin turns on at once and Notes appears in the navigation bar.
3. The Notes page stays empty until you press Allow scripts under the skin in the same list. Scripts never run before that.
4. Open Notes. `{ext.notes.title}` and `ext.notes.items` are what `data()` returned; pressing a row calls `toggle`, and `storage` keeps the ticks.
5. Edit the files on your server. The app checks the link again every few hours (set `FLUXA_SKIN_REFRESH_SECONDS` to shorten that while you test) and applies new CSS by itself. If the script changed, Allow scripts is asked again before the new script runs.

Things to know about `@element`:

- `type` can be left out: an element with `text` is a text node, one with `src` an image, anything else a column. An element with `text` and an `action` becomes a row holding the text, so it can be pressed and styled with padding and a background.
- With `each`, the element's own fields describe one entry of the list; `item` is the entry and `item.index` its position (`as` renames `item`).
- A page gets the navigation bar and the usual top spacing; its elements go inside it. A page opens over whichever screen you are on, and Back returns to it.
- A one-file skin can declare `id`, `name`, `version`, `script`, `script-id`, `hosts`, `permissions`, `events` and `refresh`. Settings pages and tracker providers need an `extension.json`, so they belong to a skin folder (see Scripts).

## What a skin is

A skin changes how Fluxa looks and how screens are arranged. It is mostly data: JSON layouts, a stylesheet, strings and constants. A skin can also bring script extensions (see Scripts below); they run in a sandbox and only after you allow them. What the app can do (addons, trackers, sources) is not part of a skin.

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
| `canvas` | logical width the skin is designed for per form factor, for example a skin authored for 960 on TV (the default skin instead uses `--tv-zoom` in `fluxa.css`, which multiplies the 1920-based TV scale) |
| `constants` | JSON file of named numbers |
| `theme` | stylesheet file |
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
| `include`, `with`, `class` | insert a fragment from `includes.json`; `with` binds its parameters; `class` is added to the fragment's root |
| `each`, `as`, `limit`, `template` | repeat `template` for every item of an array; `as` names the item (default `item`), `index` is always bound |
| `text`, `src`, `i18n` | strings with `{expression}` bindings, or a language key |
| `style`, `class` | named text styles and stylesheet classes |
| `tag` | semantic element name (`h1`, `h2`, `h3`, `p`, `small`, `label`, `span`, `li`, `ul`, `a`, `button`, `nav`, `main`, `form`, `article`, ...). It acts as the node's first, lowest-priority class, so CSS can style it as `h1 { ... }` or `.rows li small { ... }` and a `class` still overrides it. The settings screen is written this way |
| `props` | component properties, checked against the contract |
| `action` | `{ "name": "open_detail", "args": { "id": "{card.id}" } }` |
| `width`, `height`, `grow`, `gap`, `padding`, `align`, `justify`, `fill` | flex layout |
| `children` | only on `row`, `column`, `stack`, `scroll`, `fixed` |

Notes that save time:

- `padding` is `"top right bottom left"` or a single number.
- Put one-off spacing and sizes (`gap`, `padding`, `width`, `height`, `align`, `justify`, `grow`, `fill`) on the node itself rather than inventing a class for them; the default skin keeps classes for looks (colours, radii, type) and for values that change per form factor. A node field wins over the same property from a class.
- One-off classes of the default skin that were folded into a shared one (`slide`, `sources-list`, `search-fill`, `modal-scroll` into `grow`; `about-name` into `page-title`; `empty-hint` into `page-hint`; and a few more, listed in `removed` of `shared/contracts/skin-classes.json`) stay accepted in a third-party stylesheet and restyle the shared class.
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

Desktop is the baseline. Differences for narrow windows and TV are stated with media rules in the stylesheet (`.compact-only`, `.wide-only`, `.desktop-only` and `.tv-only` show a node on those sizes only). The renderer still owns focus handling, overscan margins and minimum touch targets, so a skin cannot produce an unreachable control.

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
  background: color-mix(in srgb, var(--color-text-primary) 88.235%, transparent);
}

.card.selected .card-title {
  font-weight: 700;
}
```

States: `:hover`, `:pressed`, `:focused` (or `:focus`), `:selected`, `:disabled`. A descendant selector (`.card.selected .card-title`) styles a node by its ancestor's state.

Properties in use: `background`, `color`, `font-size`, `font-weight`, `border`, `border-width`, `border-radius`, `padding`, `width`, `height`, `max-width`, `aspect-ratio`, `opacity`, `scale`, `box-shadow`, `backdrop-filter`, `transition`, `animation`, with `@keyframes` for animations.

`aspect-ratio` works with a width, a height or neither: give one side and the other follows (`.thumb { width: 160px; aspect-ratio: 16 / 9; }`), clamped by `max-width` first; an explicit width and height together win over the ratio. Across the main axis rows default to `align-items: center` and columns to `stretch`, so a rule that sets neither behaves differently on a row and on a column; write `align-items` when it matters.

Colors: `#rrggbb`, `rgba(r, g, b, a)`, `var(--color-accent)` (any color token, including the `--ink-*` text alphas) and `color-mix(in srgb, var(--color-background) 50%, transparent)` for a token at partial alpha. The default skin writes only these forms. The older spellings still parse for existing skins: theme names (`background`, `theme.surface`, `accent`, `text.primary`), `ink/225` and `ink/soft`, `background*0.5`, and a bare `pill` radius.

Gradients: `linear-gradient(to bottom, color-mix(in srgb, var(--color-background) 0%, transparent) 0%, var(--color-background) 100%)`, with 2 to 16 color stops.

Media rules switch values by size: `@media (size: compact)`, `(min-width: 900px)`, `(max-height: 479px)`, the platform `(platform: tv | mobile | desktop)` and the last input used `(input: touch | pointer | focus)`. Separate queries with commas for "or": `@media (size: compact), (max-height: 479px) { ... }`. The shorthands `@mobile`, `@tv`, `@medium`, `@expanded`, `(tv)` and `(window: ...)` keep working but the default skin does not use them (`npm run check:structure` enforces that).

Component classes: buttons are `.btn-primary`, `.btn-secondary`, `.btn-selected` and `.btn-danger` for layout nodes and for the buttons the engine draws itself; `.chip` (with `.chip.selected`) likewise. A button sets `color` and `font-size` once and its `.btn-label` inherits them (keep an empty `.btn-label {}` rule: a class needs some rule to take part in inheritance). `action-primary`, `action-secondary`, `button.primary`, `button.secondary`, `button.selected`, `button.danger` and `chip.selected` are accepted as aliases of the names above, so older skins keep working.

Colors, spacing, radii, icon sizes and motion durations are `:root` custom properties (`--color-surface`, `--color-accent`, `--space-4`, `--radius-md`, and so on; the full list is in `shared/skins/default/style.css`). Your skin overrides them in its own `:root` block, and `@media (theme: id)` switches rules for one built-in theme. Layout and component metrics (row heights, hero ratios, poster size clamps, player sizes and similar) are not in the stylesheet: the engine carries typed defaults for them in `native/fluxa-ui/src/stylesheet/defaults.rs`. A skin that wants a different value declares the same `--name` in `:root` (or inside `@media (size: compact|medium|tv)`), and that wins over the engine default. A user theme file can override numeric values under `vars`, colors under `colors` and durations under `motion`.

Scoped custom properties: a `--name` declared in any rule (not only `:root`) applies to the elements that rule matches and to their descendants; the nearest declaration wins and `:root` answers when none does. Use them to build variants that only swap colors:

```css
.btn-primary, .btn-danger { background: var(--btn-bg); color: var(--btn-fg); states: auto; }
.btn-primary { --btn-bg: var(--ink-full); --btn-fg: var(--color-background); }
.btn-danger { --btn-bg: var(--color-error); --btn-fg: var(--color-text-primary); }
```

Scoped properties are read by the color properties (`background`, `color`, `border-color`, gradients and `color-mix()` stops); layout lengths still take `:root` values only. Declarations inside `:hover` and the other state blocks are ignored.

Derived states: add `states: auto;` to a rule that declares a `background` (a solid color) and the engine fills in the states you did not write. `:hover`: an opaque background dims to 85 % opacity, a translucent one (alpha below about 90 %) gains 8 % of its remaining alpha, so a `color-mix(... 10.98%, transparent)` fill hovers at 18.04 %. `:disabled`: the shared disabled look (`--ink-soft` fill, `--ink-dim` text). Any `:hover` or `:disabled` you declare replaces the derived one, and `:focus`, `:pressed` and `:selected` are never derived. Without `states: auto` nothing is derived, so existing skins look the same (`states: none` turns it off again for a rule that inherits it).

HTML-style CSS: a node's `tag` is a selector, so text roles and rows are styled once by element rather than by a class per node, and containers style their children with descendant selectors (a node is matched against every ancestor class and tag; `>` is accepted and behaves like a descendant combinator):

```css
:root { font-size: 16px; }              /* what 1rem means */
@media (size: compact) { :root { font-size: 15px; } }

h1 { font-size: 1.625rem; font-weight: 700; }
small { font-size: .8125rem; color: var(--ink-subtle); }
.rows li { height: 3.5rem; padding: 0 1rem; gap: .75rem; align-items: center; }
.rows li label { flex-grow: 1; }
.rows li:hover, a:hover { background: var(--ink-soft); }
button:focus { scale: 1.04; }
```

`rem` is a multiple of the root font size (`font-size` on `:root` or `html`, 16px when unset, and a media block may change it per form factor so a whole screen scales by one number). `em` is a multiple of the element's own font size (of the inherited one when used for `font-size` itself). Both work in `font-size`, `padding`, `margin`, `gap`, `width`, `height`, `max-width` and `column-width`; `px` keeps working everywhere, and `border-radius` stays in `px`/tokens. Layout declared in CSS (`gap`, `padding`, `height`, `justify-content`, ...) applies to nodes through descendant selectors too, not only through the node's own class. A text node needs a tag or class with a rule to inherit color and size from its parent (`span {}` is the empty rule for plain labels).

Every screen of the default skin is written this way. The `Base elements` section of `style.css` holds the generic rules (`h1`..`h4`, `p`, `small`, `label`, `span`, `i`, `a`, `button`, `ul`); each screen then has one container class per section (`.feed`, `.login`, `.panel-box`, `.player-panel`...) whose tag descendants carry the gap, padding and sizes (`.panel-box footer { gap: .5rem; }`, `.login header { align-items: center; }`), so nodes carry a `tag` and a container class instead of per-node `gap`/`padding`/`width` fields. Fields stay on a node only when they are bound to data (`"width": "{const.panelWidth}"`).

Buttons are `button` nodes (`"tag": "button"`, optional `"class": "primary"`, label children tagged `span` and `i`). Rules are resolved tag first, then classes in order, so a class always overrides a tag rule even when the tag rule has a descendant selector; give distinct roles distinct tags or classes rather than relying on the order of two descendant rules for the same tag. Descendant selectors also work inside `@media` blocks (`@media (size: compact) { .navbar a { padding: 1rem; } }`) and rank after the plain rules of the same weight. `btn-label` is an alias for `span`; `btn-primary`, `btn-secondary`, `btn-selected` and `btn-danger` stay as the names the Rust button widgets look up.

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

## Pages and navigation entries

A skin can add screens of its own. List them under `pages` (name to layout file) and add entries to the navigation bar under `nav`:

```json
"pages": { "agenda": "pages/agenda.json" },
"nav": [{ "page": "agenda", "label": "Agenda", "icon": "NavLibrary" }]
```

A page name uses lowercase letters, digits, `-` and `_` and cannot reuse a built-in screen name. The `page` action (`{"name": "page", "args": {"name": "agenda"}}`) opens one; Back or any navigation returns to the previous screen. Every `nav` entry must point at a page of the same skin.

## Scripts

A skin lists extension manifests under `extensions`. Each `extension.json` has `kind: "script"`, an `entry` file, `permissions` and the `networkHosts` it may call. The script defines `class Provider` with a `data()` method; its result appears in layouts as `ext.<extension id>`. `refreshSeconds` (at least 30) re-runs `data()` on an interval.

Layouts call other methods with an action named `ext.<id>.<method>`. Extensions that declare `contributes.events` receive events through `onEvent(name, payload)` and need the matching `events.*` and `background` permissions. Events: `playback.started`, `playback.paused`, `playback.progress`, `playback.ended`, `playback.watched` (ids the core newly marks watched), `profile.changed`, `navigation.changed`.

Event payloads: `playback.started`, `playback.paused` and `playback.progress` carry the player state (`title`, `position`, `duration`, `paused`, `volume`); `playback.ended` carries `{title}`; `playback.watched` carries `{ids}`; `profile.changed` carries the new profile (`id`, `name`); `navigation.changed` carries `{route}`. A script only receives the events it lists in `contributes.events` and has the `events.playback`, `events.profile`, `events.navigation`, `events.library` or `events.tracker` permission for.

```js
class Provider {
  async onEvent(name, payload) {
    if (name === "playback.watched") storage.set("last", payload.ids);
  }
}
```

Permissions an extension can ask for: `storage`, `network` (with `networkHosts`), `background`, `ui.settings`, `provider.tracker`, `state.player`, `state.profile`, `state.library`, `state.settings`, `state.navigation`, `actions.player`, `actions.navigate` and the `events.*` ones above. A call that needs a permission the extension did not declare fails.

Globals available to every script call:

- `storage.get(key, fallback)`, `storage.set(key, value)`, `storage.remove(key)`, `storage.keys()`: persistent per extension, 64 KiB, needs `storage`.
- `state(path)` or `state()`: read-only app state (`player.title|position|duration|paused|volume`, `profile.id|name`, `library.watched`, `settings`, `navigation.route`). Each slice needs its own permission: `state.player`, `state.profile`, `state.library`, `state.settings`, `state.navigation`. Layouts see the same data as `app.*`.
- `settings`: the values of the settings the extension declares.
- `app.call(name, args)`: asks the app to act. `player.volume {value}`, `player.seek {seconds}` or `{to}`, `player.pause`, `player.play`, `player.toggle`, `play {id,type}` need `actions.player`; `navigate {screen}`, `page {name}`, `back`, `search {query}`, `open_detail {id,type}` need `actions.navigate`. The app decides whether and how to act.

An extension declares `settings` (`toggle`, `choice` with `choices`, `text`, `number` with `min`/`max`, optional `default`) and needs `ui.settings`. Layouts read them from `app.extensions.<id>.settings` and change one with the action `ext.<id>.setting(key, value)`; the script is told through `onSetting(key, value)`.

```json
"permissions": ["storage", "ui.settings"],
"settings": [
  { "key": "days", "title": "Days shown", "kind": "choice", "default": "5",
    "choices": [{ "value": "5", "label": "Weekdays" }, { "value": "7", "label": "Full week" }] },
  { "key": "quiet", "title": "Quiet mode", "kind": "toggle", "default": false }
]
```

The values are listed under the skin in Settings, Theme, Skins, and a script reads them from `settings.days` and `settings.quiet`.

An extension with `provider.tracker` and `contributes.trackers` implements a `tracker` object: `auth`, `scrobbleStart`, `scrobblePause`, `scrobbleStop` (called by the app on playback events), `watched` and `library`. Optional `status()` returning `{connected}` and `disconnect()` let the app show the tracker as a row on the Account page with a Connect or Disconnect button.

```js
class Provider {
  constructor() {
    this.tracker = {
      status: async () => ({ connected: storage.get("token", "") !== "" }),
      disconnect: async () => storage.remove("token"),
      auth: async (credentials) => {
        storage.set("token", await login(credentials));
        return { connected: true };
      },
      scrobbleStart: (item) => send("/start", item),
      scrobblePause: (item) => send("/pause", item),
      scrobbleStop: (item) => send("/stop", item),
      watched: async () => [],
      library: async () => [],
    };
  }
}
```

The extension declares `provider.tracker` and `contributes.trackers: [{ "id": "example", "title": "Example Tracker" }]`. The app calls the scrobble methods on its own when playback starts, pauses and ends; `watched` and `library` results are shown on the tracker's row; `login` and `send` stand for your own calls (see `shared/skins/examples/agenda/ext/tracker.js`).

Scripts never run until you allow them for that skin in Settings. The consent covers the exact scripts and hosts; any change to them asks again. `shared/skins/examples/agenda` is a complete example with a page, a nav entry, state, storage, settings and a tracker.

## A skin in one CSS file

A skin can be a single `.css` file. Its first block says who it is, and the rest is ordinary CSS:

```css
@skin {
  id: my-skin;
  name: "My Skin";
  version: 1.0.0;
  script: "agenda.js";
  hosts: example.com;
  permissions: network;
}

.poster { gap: 8px; }

@page agenda { nav-label: "Agenda"; }

@element agenda-row {
  parent: .page-agenda;
  each: ext.agenda.items;
  text: "{item.label}";
  action: ext.agenda.toggle(index: item.index);
}

@element agenda-hint {
  parent: .navbar;
  position: start;
  type: text;
  text: "Hi";
}
```

- `@element name { … }` adds a node inside every node that has the `parent` class. Fields: `parent`, `position` (`start` or `end`), `type`, `text`, `src`, `style`, `when`, `each`, `as`, `class`, `action`. The node gets the element's name as a class, so you style it with normal CSS.
- `@page name { nav-label; nav-icon }` adds a screen. Put content in it with elements whose `parent` is `.page-<name>`.
- `action: name(arg: value, …)`: a quoted value is text, `item.index` reads data. `page(agenda)` opens a page.
- `script`, `hosts`, `permissions`, `events` and `refresh` describe the optional script that comes next to the CSS; `script-id` names it for `ext.<id>` (default `ext`).

Paste the link of the `.css` file into the skin installer (Settings, Skins). The script path is read relative to the CSS link. The app checks the links again every few hours, so the author edits the files on their own server and nobody reinstalls. A script that changed asks for permission again.


## Class contract and stability

The default skin applies a fixed vocabulary of class names, and themes target them from CSS. `shared/contracts/skin-classes.json` lists every class the default skin uses, with the element it sits on, a one-line meaning and a stability:

- `stable`: public. Third-party themes may rely on it (poster, navbar, hero, buttons, chips, shelves, settings rows, sliders, player panels, `only-*` form helpers). A stable class keeps its name, element and meaning inside one contract version.
- `internal`: used by the default skin only. It can be renamed or removed at any time, so do not select it from a theme. All layout helper classes such as `gap-8` or `items-stretch` are internal, and the check refuses to mark one stable.

The file also carries a `version` (a whole number) and a `removed` list. Adding a stable class, or changing an internal one, needs no bump. Removing or renaming a stable class is a breaking change: move its entry to `removed` with `removedIn` set to the new contract version (it must be greater than the entry's `since`), optionally add `renamedTo`, and raise `version` to match. `npm run check:structure` fails when a stable class is no longer used by any screen, include or `style.css` selector without that bookkeeping.

The file is generated and then edited by hand:

```bash
node scripts/skin-classes.mjs
```

The script reads `screens/*.json`, `includes.json` and the selectors of `style.css`, adds new classes as `internal` with an empty meaning, and keeps the meaning, stability and `since` you wrote for existing ones. Run it after adding classes, then fill in the meaning and promote the ones themes should rely on. Classes assembled from data at runtime (for example `{card.shape}` or `{item.class}`) are found only where `style.css` selects them.
