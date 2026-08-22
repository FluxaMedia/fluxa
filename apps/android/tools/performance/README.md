# Android frame smoke tests

Install a release-like benchmark build on real hardware before using these scripts.

## Mobile

```bash
./tools/performance/mobile-frame-test.sh com.fluxa.app.mobile
```

The touch journey swipes the hero, scrolls horizontal catalog rows, traverses Home vertically and captures the 18-second billboard update.

## Android TV

```bash
./tools/performance/tv-frame-test.sh com.fluxa.app.tv
```

The TV journey uses rapid D-pad navigation, keeps folder focus GIF behavior active, and crosses the 18-second billboard update.

Reports are written under `build/performance/`. Use Android Studio System Trace for remaining spikes and inspect `main`, `RenderThread`, GPU completion, image decode and garbage collection.
