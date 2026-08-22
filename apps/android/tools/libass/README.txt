Testing ExoPlayer Libass Support
================================

Fluxa renders ASS/SSA subtitles for ExoPlayer through a native libass overlay because Media3 does not preserve the full ASS styling model. The verification needs to run on Android, not only on the JVM, because NativeLibassRenderer loads fluxa_libass_renderer and resolves libass symbols from the packaged libmpv.so.

Fast device probe
-----------------

Run the always-available instrumentation tests:

./gradlew :app:connectedMobileDebugAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=com.fluxa.app.player.LibassExoPlayerInstrumentationTest

These tests verify:

- NativeLibassRenderer.create(...) succeeds on-device.
- ASS timing produces pixels only inside the dialogue window.
- ASS positioning draws around the expected frame region.
- LibassEventRelay converts raw MKV ASS packet bodies into renderable ASS dialogue lines.
- Fluxa-created ExoPlayer instances register a LibassEventRelay.

Full embedded MKV probe
-----------------------

Generate a small MKV with an embedded ASS subtitle track:

tools/libass/generate-exoplayer-libass-fixture.sh /tmp/fluxa-libass-fixture
adb push /tmp/fluxa-libass-fixture/libass-exoplayer-probe.mkv /data/local/tmp/libass-exoplayer-probe.mkv

Run the optional end-to-end ExoPlayer playback probe:

./gradlew :app:connectedMobileDebugAndroidTest \
  -Pandroid.testInstrumentationRunnerArguments.class=com.fluxa.app.player.LibassExoPlayerInstrumentationTest \
  -Pandroid.testInstrumentationRunnerArguments.libassMkvUrl=file:///data/local/tmp/libass-exoplayer-probe.mkv \
  -Pandroid.testInstrumentationRunnerArguments.libassProbeMs=1500

The optional probe starts a real Fluxa ExoPlayer, waits for an embedded SSA/ASS text track, enables it, waits for the relay renderer, then asks libass to render at the probe timestamp and checks that subtitle pixels exist.

Logcat
------

Keep this running while testing:

adb logcat -s FluxaLibassRenderer NativeLibassOverlay PlayerScreen

Failures to investigate:

- Could not open libmpv.so
- Missing libass symbol
- libass not available via libmpv.so
- External renderer failed
- Embedded renderer failed
