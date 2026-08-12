# Fluxa TV performance tests

Use a real Android TV/Google TV device when judging frame pacing. Open the TV flavor once, complete profile selection, and make sure Home contains several populated rows before running either test.

## Frame timing benchmark

```bash
./gradlew :tvBenchmark:connectedTvBenchmarkAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=com.fluxa.app.tvbenchmark.TvHomeMacrobenchmark
```

The benchmark deliberately stays on Home for more than 18 seconds so `HomeBillboardRuntime` rotation is included in `FrameTimingMetric` and its Perfetto trace. It can run on the TV flavor's API 24 minimum. `frameOverrunMs` is available on API 31+, while older devices still report CPU frame duration.

The cold-start benchmark includes torrent bootstrap contention. For an explicit system trace around torrent startup, run `tools/performance/torrent-startup-perfetto.sh com.fluxa.app.tv`.

## Baseline Profile capture

`BaselineProfileRule` requires Android 13/API 33+ on an unrooted device, or a rooted API 28+ device.

```bash
./gradlew :tvBenchmark:connectedTvBenchmarkAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=com.fluxa.app.tvbenchmark.TvBaselineProfileGenerator
```

The generated human-readable file is named `tv-home-baseline-prof.txt`. For a connected device, look under `tvBenchmark/build/outputs/connected_android_test_additional_output/`; the exact subdirectory includes the variant and device. Review the generated rules and merge them into `app/src/main/baseline-prof.txt` before producing the release APK.

## Fast ADB smoke test

For a quick check without Macrobenchmark:

```bash
tools/performance/tv-frame-test.sh com.fluxa.app.tv
```

This resets `gfxinfo`, performs the same D-pad-heavy Home journey, crosses the 18-second billboard boundary, and writes frame statistics to `build/performance/`.
