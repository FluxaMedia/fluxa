<!-- Keep the title short and imperative. English only. -->

## What & why

<!-- What does this change do, and why is it needed? The "why" is the part a diff can't show. -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] UI / visual
- [ ] Refactor / cleanup
- [ ] Performance
- [ ] Docs / tooling

## How it was tested

- [ ] `./gradlew :app:assembleMobileDebug` builds clean
- [ ] Unit tests pass for the modules touched (`./gradlew :<module>:test`)
- [ ] If Rust changed: `cargo fmt` / `cargo clippy` / `cargo test --lib` clean in the crate

Platforms tested:

- [ ] Android (phone / tablet)
- [ ] Android TV
- [ ] iOS
- [ ] tvOS

## Checklist

- [ ] No hardcoded user-facing strings — everything via `AppStrings.t(lang, "key")`, keys added to **both** `core/src/commonMain/resources/i18n/english_us.json` and `tr_tr.json`
- [ ] Compose leaves receive small UI models / primitives, not `Meta` / `Stream` / repositories / ViewModels
- [ ] Rust does no direct I/O — network/storage work is modeled as an effect
- [ ] No comments added to code; everything developer-facing is English
- [ ] Logic that belongs in fluxa-core wasn't put in the Kotlin shell instead

## Related issues

<!-- "Closes #123", or remove this line. -->
