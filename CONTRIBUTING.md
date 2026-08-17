# Contributing to Fluxa Desktop

Thanks for wanting to help. Fluxa Desktop is the Tauri shell — one codebase that ships as
a native desktop app, a browser build ([Fluxa Pages](https://fluxamedia.github.io/fluxa-desktop/)),
and a webOS build. It handles windows, the UI, and platform I/O; the decisions live in
[fluxa-core](https://github.com/FluxaMedia/fluxa-core).

## The shell is thin on purpose

Before writing a feature, ask: **does this belong in fluxa-core or here?**

| fluxa-core | fluxa-desktop |
|---|---|
| Addon resolution, stream policy | Window / tray / OS integration |
| Library & playback rules, search | Tauri commands (thin wrappers) |
| Catalog handling, data models | React UI components |
| State machines | IPC glue, desktop config |

If a feature is about *what* the app does, it's core. If it's about *how* it presents or
integrates on a device, it's here. When in doubt, prefer core so the logic stays reusable
across Android and the other targets — but only move genuinely portable, decision-making
code, not something that needs a platform API to express.

## Getting set up

```bash
git clone https://github.com/FluxaMedia/fluxa-desktop.git
cd fluxa-desktop
npm install
npm run tauri dev
```

**Prerequisites**

- Node 22+ and Rust stable
- `libmpv` — install it system-wide, or run `./src-tauri/fetch-libmpv.sh` to pull the
  prebuilt gpu-next fork used for release builds

fluxa-core is consumed from `../fluxa-core` (see `package.json`), so clone it as a sibling
directory. The commands you'll use most:

```bash
npm run check            # typecheck + cargo check — run this before every PR
npm test                 # frontend unit tests (vitest)
npm run build            # production frontend build
npm run build:web        # browser build in dist-web
npm run build:web:pages  # browser build with the GitHub Pages base path
npm run build:webos      # webOS build in dist-webos
```

## Before you open a PR

```bash
npm run check
npm test
```

Both must pass. If you touched Rust, `cargo fmt` and `cargo clippy` in `src-tauri/` too.

## Two things that bite in this repo

**Web/webOS builds run the same code as the desktop app.** A `@tauri-apps/*` import or a
Tauri command with no web fallback will compile fine and then throw at runtime in the
browser build. If you add a native command, add its `webInvoke` case in
`src/platform/web/invoke.ts` (or a deliberate unsupported path). Reference bundled assets
with `assetUrl()` from `src/platform/assets`, never a bare `/foo.svg` — the Pages build is
served from a subdirectory and absolute paths 404 there.

**`npm run tauri dev` is not a packaged build.** Bugs involving the asset protocol, CSP,
bundled-library paths, or production error boundaries only show up in a real bundle. Build
one locally before claiming a production-only fix is done:

```bash
npm run tauri build -- --bundles appimage   # Linux
./src-tauri/target/release/bundle/appimage/*.AppImage --appimage-extract
squashfs-root/AppRun                        # run via AppRun, not the raw binary
```

Cutting a GitHub release is not part of testing a fix — build and verify locally first.

## i18n

Every user-facing string goes through `t()` from `src/i18n.ts`. No hardcoded text in JSX
or component logic — not even with a `|| 'fallback'` guard. When you add a string:

1. Add the key to `src/i18n/english_us.json`
2. Add the translation to `src/i18n/tr_tr.json`
3. Use `t('your.key')` — `%s` substitution works: `t('player.playing_in_seconds', n)`

## Style

- **No comments in code.** Not inline, not doc comments. Explanation belongs in the commit
  message or PR description; rename or restructure until the code reads on its own.
- **English only** for everything developer-facing: identifiers, files, commit messages,
  PR text, test names.
- Match the surrounding component's idiom — this codebase leans on inline styles and small
  hooks; follow what a file already does rather than importing a new pattern.
- For UI, keep it neutral: white/muted text on dark surfaces, simple borders, no random
  accent colours or decorative gradients.
- Keep changes focused — one logical change per commit, no drive-by refactors folded into a fix.

## Commits and pull requests

- Write a real commit message: what changed and *why*, imperative mood.
- Say in the PR which targets you checked. "Works in `tauri dev`" is not the same as "works
  in the AppImage" or "works on Pages" — name what you actually ran.
- New behaviour needs a test where it's testable; a bug fix should include the test that
  catches it.

## Reporting bugs

Open an issue with your OS, whether it's the desktop/web/webOS build, the version, and
steps to reproduce. For a crash, the terminal output or the in-app error boundary text is
gold. If it only happens in a packaged build, say so.

## Legal

Contributions are licensed under the repository's GPLv3. Fluxa is a client for
user-installed addons; it hosts and distributes no content. Don't add anything that changes that.

Questions are welcome on [Discord](https://discord.gg/wan9FeDEfe).
