# Fluxa mobile performance tests

Use a real phone with a populated Home screen and a selected profile. Measure the `mobileBenchmark` build, never a debug build.

## Frame timing

```bash
./gradlew :app:assembleMobileBenchmark
./gradlew :mobileBenchmark:connectedMobileBenchmarkAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=com.fluxa.app.mobilebenchmark.MobileHomeMacrobenchmark
```

The journey swipes the hero, horizontally scrolls catalog rows, vertically traverses Home and remains active long enough to capture the 18-second billboard update.

## Baseline Profile

On an ordinary unrooted device use API 33 or newer:

```bash
./gradlew :mobileBenchmark:connectedMobileBenchmarkAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=com.fluxa.app.mobilebenchmark.MobileBaselineProfileGenerator
```

Review `mobile-home-baseline-prof.txt` under the benchmark additional-output directory before merging it into the app profile.
