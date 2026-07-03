package com.fluxa.app.player

import android.graphics.Bitmap
import android.graphics.SurfaceTexture
import android.net.Uri
import android.os.SystemClock
import android.view.Surface
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.atomic.AtomicReference
import org.junit.After
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LibassExoPlayerInstrumentationTest {
    private val players = mutableListOf<ExoPlayer>()
    private val surfaces = mutableListOf<Surface>()
    private val textures = mutableListOf<SurfaceTexture>()
    private val renderers = mutableListOf<NativeLibassRenderer>()

    @After
    fun tearDown() {
        renderers.forEach { it.close() }
        players.forEach { player ->
            runOnMainSync { player.release() }
        }
        surfaces.forEach { it.release() }
        textures.forEach { it.release() }
    }

    @Test
    fun nativeRendererRendersStyledAssPixelsAndHonorsTiming() {
        val renderer = requireNotNull(NativeLibassRenderer.create(assProbeDocument().toByteArray())) {
            "Native libass renderer could not be created. Check libmpv.so packaging and libass symbols."
        }
        renderers += renderer

        val beforeBitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT, Bitmap.Config.ARGB_8888)
        val beforeResult = renderer.render(500L, beforeBitmap)
        assertTrue("Expected no ASS image before the dialogue start.", beforeResult == 0)
        assertTrue("Expected transparent bitmap before the dialogue start.", beforeBitmap.nonTransparentPixelCount() == 0)

        val activeBitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT, Bitmap.Config.ARGB_8888)
        val activeResult = renderer.render(1_500L, activeBitmap)
        assertTrue("Expected libass to report a visible image during the dialogue.", activeResult > 0)
        assertTrue(
            "Expected rendered subtitle pixels during the dialogue.",
            activeBitmap.nonTransparentPixelCount() > MIN_EXPECTED_PIXELS
        )
        assertTrue(
            "Expected ASS positioning to place pixels near the center of the frame.",
            activeBitmap.nonTransparentPixelCountIn(160, 90, 480, 270) > MIN_EXPECTED_PIXELS / 2
        )

        val afterBitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT, Bitmap.Config.ARGB_8888)
        val afterResult = renderer.render(3_000L, afterBitmap)
        assertTrue("Expected no ASS image after the dialogue end.", afterResult == 0)
        assertTrue("Expected transparent bitmap after the dialogue end.", afterBitmap.nonTransparentPixelCount() == 0)
    }

    @Test
    fun relayConvertsMkvAssEventsToRenderableFramesAndCanClearThem() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val relay = LibassEventRelay()
        relay.setHeader(assProbeHeader().toByteArray(), emptyList(), context.filesDir.resolve("fonts").absolutePath)
        relay.renderThread.drainForTesting()

        val renderer = requireNotNull(relay.activeRenderer.value) {
            "Relay did not create a native libass renderer from the ASS header."
        }
        renderers += renderer

        relay.addEvent(
            startMs = 1_000L,
            durationMs = 1_000L,
            rawMkvBody = "0,0,Probe,,0,0,0,,{\\pos(320,180)}RELAY LIBASS".toByteArray()
        )
        relay.renderThread.drainForTesting()

        val activeBitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT, Bitmap.Config.ARGB_8888)
        val activeResult = renderer.render(1_500L, activeBitmap)
        assertTrue("Expected relayed MKV ASS event to render.", activeResult > 0)
        assertTrue(
            "Expected relayed MKV ASS event to produce visible pixels.",
            activeBitmap.nonTransparentPixelCount() > MIN_EXPECTED_PIXELS
        )

        relay.clearEvents()
        relay.renderThread.drainForTesting()
        val clearedBitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT + 1, Bitmap.Config.ARGB_8888)
        val clearedResult = renderer.render(1_500L, clearedBitmap)
        assertTrue("Expected no ASS image after clearing relayed events.", clearedResult == 0)
        assertTrue("Expected transparent bitmap after clearing relayed events.", clearedBitmap.nonTransparentPixelCount() == 0)

        relay.addEvent(
            startMs = 2_000L,
            durationMs = 1_000L,
            rawMkvBody = "Dialogue: 0:00:00:00,0:00:01:00,0,0,Probe,,0,0,0,,{\\pos(320,180)}MEDIA3 LIBASS".toByteArray()
        )
        relay.renderThread.drainForTesting()
        val media3Bitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT, Bitmap.Config.ARGB_8888)
        val media3Result = renderer.render(2_500L, media3Bitmap)
        assertTrue("Expected Media3-prefixed ASS event to render.", media3Result > 0)
        assertTrue(
            "Expected Media3-prefixed ASS event to produce visible pixels.",
            media3Bitmap.nonTransparentPixelCount() > MIN_EXPECTED_PIXELS
        )
    }

    @Test
    fun createdExoPlayerRegistersLibassRelay() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val player = runOnMainSync {
            MediaPlayerController.createExoPlayer(
                context = context,
                bufferCacheMb = 32,
                forwardBufferSeconds = 10,
                backBufferSeconds = 0,
                minBufferSeconds = 2,
                playbackBufferMs = 500,
                rebufferBufferMs = 1_000
            )
        }
        players += player

        assertNotNull(
            "Expected Fluxa ExoPlayer factory to register a LibassEventRelay.",
            MediaPlayerController.getLibassRelay(player)
        )
    }

    @Test
    fun optionalEmbeddedAssMkvPlaybackActivatesRelayAndRendersPixels() {
        val args = InstrumentationRegistry.getArguments()
        val mkvUrl = args.getString("libassMkvUrl").orEmpty()
        assumeTrue(
            "Pass -e libassMkvUrl file:///sdcard/Android/data/com.fluxa.app.mobile/files/libass-exoplayer-probe.mkv to run the full MKV playback probe.",
            mkvUrl.isNotBlank()
        )
        val probeMs = args.getString("libassProbeMs")?.toLongOrNull() ?: 1_500L

        val context = InstrumentationRegistry.getInstrumentation().targetContext
        assertReadableFileUrlIfLocal(mkvUrl)
        val player = runOnMainSync {
            MediaPlayerController.createExoPlayer(
                context = context,
                bufferCacheMb = 32,
                forwardBufferSeconds = 10,
                backBufferSeconds = 0,
                minBufferSeconds = 2,
                playbackBufferMs = 500,
                rebufferBufferMs = 1_000
            )
        }
        players += player

        val texture = SurfaceTexture(0)
        val surface = Surface(texture)
        textures += texture
        surfaces += surface
        runOnMainSync { player.setVideoSurface(surface) }

        val controller = runOnMainSync { MediaPlayerController(context, player) }
        val failure = AtomicReference<PlaybackException?>()
        runOnMainSync {
            player.addListener(object : Player.Listener {
                override fun onPlayerError(error: PlaybackException) {
                    failure.set(error)
                }
            })
        }

        runOnMainSync { controller.prepareAndPlay(mkvUrl) }
        waitUntil("Timed out waiting for ExoPlayer to discover an embedded SSA/ASS subtitle track. error=${failure.get()}") {
            failure.get()?.let { throw it }
            controller.availableSubtitles.value.any { it.sampleMimeType == MimeTypes.TEXT_SSA }
        }

        val assTrack = controller.availableSubtitles.value.first { it.sampleMimeType == MimeTypes.TEXT_SSA }
        runOnMainSync {
            controller.enableSubtitle(assTrack)
            player.seekTo(0L)
            player.playWhenReady = true
        }

        val relay = requireNotNull(MediaPlayerController.getLibassRelay(player)) {
            "Expected a LibassEventRelay for the test ExoPlayer."
        }
        waitUntil("Timed out waiting for embedded ASS playback to create a native renderer. error=${failure.get()}") {
            failure.get()?.let { throw it }
            relay.activeRenderer.value != null
        }

        val renderer = requireNotNull(relay.activeRenderer.value)
        relay.renderThread.drainForTesting()
        val bitmap = Bitmap.createBitmap(PROBE_WIDTH, PROBE_HEIGHT, Bitmap.Config.ARGB_8888)
        val result = renderer.render(probeMs, bitmap)
        assertTrue("Expected embedded ASS renderer to report visible pixels at $probeMs ms.", result > 0)
        assertTrue(
            "Expected embedded ASS renderer to draw visible pixels at $probeMs ms.",
            bitmap.nonTransparentPixelCount() > MIN_EXPECTED_PIXELS
        )
    }

    private fun assertReadableFileUrlIfLocal(url: String) {
        val uri = Uri.parse(url)
        if (uri.scheme != "file") return
        val path = requireNotNull(uri.path) { "File URL has no path: $url" }
        val file = java.io.File(path)
        assertTrue(
            "The test app cannot read $path. Push the fixture into the app-specific external files directory, " +
                "for example: adb push /tmp/fluxa-libass-fixture/libass-exoplayer-probe.mkv " +
                "/sdcard/Android/data/com.fluxa.app.mobile/files/libass-exoplayer-probe.mkv",
            file.canRead()
        )
    }

    private fun <T> runOnMainSync(block: () -> T): T {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val result = AtomicReference<T>()
        val failure = AtomicReference<Throwable>()
        instrumentation.runOnMainSync {
            runCatching(block)
                .onSuccess { result.set(it) }
                .onFailure { failure.set(it) }
        }
        failure.get()?.let { throw it }
        return result.get()
    }

    private fun waitUntil(message: String, timeoutMs: Long = 30_000L, predicate: () -> Boolean) {
        val deadline = SystemClock.elapsedRealtime() + timeoutMs
        while (SystemClock.elapsedRealtime() < deadline) {
            if (predicate()) return
            Thread.sleep(100L)
        }
        assertTrue(message, predicate())
    }

    private fun Bitmap.nonTransparentPixelCount(): Int {
        val pixels = IntArray(width * height)
        getPixels(pixels, 0, width, 0, 0, width, height)
        return pixels.count { (it ushr 24) != 0 }
    }

    private fun Bitmap.nonTransparentPixelCountIn(left: Int, top: Int, right: Int, bottom: Int): Int {
        val clippedLeft = left.coerceIn(0, width)
        val clippedTop = top.coerceIn(0, height)
        val clippedRight = right.coerceIn(clippedLeft, width)
        val clippedBottom = bottom.coerceIn(clippedTop, height)
        if (clippedRight == clippedLeft || clippedBottom == clippedTop) return 0
        val regionWidth = clippedRight - clippedLeft
        val regionHeight = clippedBottom - clippedTop
        val pixels = IntArray(regionWidth * regionHeight)
        getPixels(pixels, 0, regionWidth, clippedLeft, clippedTop, regionWidth, regionHeight)
        return pixels.count { (it ushr 24) != 0 }
    }

    private fun assProbeDocument(): String = """
        ${assProbeHeader()}
        Dialogue: 0,0:00:01.00,0:00:02.00,Probe,,0,0,0,,{\pos(320,180)}NATIVE LIBASS
    """.trimIndent()

    private fun assProbeHeader(): String = """
        [Script Info]
        ScriptType: v4.00+
        PlayResX: 640
        PlayResY: 360
        WrapStyle: 0

        [V4+ Styles]
        Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
        Style: Probe,sans-serif,48,&H0000FF00,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,0,5,20,20,20,1

        [Events]
        Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    """.trimIndent()

    private companion object {
        const val PROBE_WIDTH = 640
        const val PROBE_HEIGHT = 360
        const val MIN_EXPECTED_PIXELS = 50
    }
}
