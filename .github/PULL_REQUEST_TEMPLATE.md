<!-- Keep the title short and imperative, e.g. "Resolve icons against the base path". -->

## What & why

<!-- What does this change do, and why is it needed? The "why" is the part a diff can't show. -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] UI / visual
- [ ] Refactor / cleanup
- [ ] Docs / tooling

## How it was tested

<!-- Say what you actually ran, not what should work. Screenshots or a short clip help for UI changes. -->

- [ ] `npm run check` passes (typecheck + cargo check)
- [ ] `npm test` passes
- [ ] Verified in a real build, not only `tauri dev`, if this is a production-only path

Platforms tested:

- [ ] Windows
- [ ] macOS
- [ ] Linux
- [ ] Web
- [ ] webOS

## Checklist

- [ ] No new hardcoded user-facing strings — everything goes through `t()`, keys added to **both** `english_us.json` and `tr_tr.json`
- [ ] Bundled assets referenced via `assetUrl()`, not a bare `/foo.svg`
- [ ] New native command has a `webInvoke` case (or a deliberate unsupported path)
- [ ] No comments added to code
- [ ] Logic that belongs in fluxa-core wasn't put here instead

## Related issues

<!-- "Closes #123", or remove this line. -->
