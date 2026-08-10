package com.fluxa.app.mobilebenchmark

import android.os.SystemClock
import androidx.benchmark.macro.MacrobenchmarkScope

internal const val MOBILE_PACKAGE_NAME = "com.fluxa.app.mobile"

/**
 * Touch journey shared by the mobile frame benchmark and Baseline Profile generator.
 * Run it with a signed-in profile whose Home screen has multiple populated rows.
 */
internal fun MacrobenchmarkScope.browseMobileHome(
    includeBillboardRotation: Boolean,
) {
    val width = device.displayWidth
    val height = device.displayHeight

    // Exercise the hero pager and the first horizontal catalog row.
    repeat(3) {
        device.swipe(
            (width * 0.84f).toInt(),
            (height * 0.42f).toInt(),
            (width * 0.18f).toInt(),
            (height * 0.42f).toInt(),
            12,
        )
        SystemClock.sleep(180)
    }

    // Scroll through image-heavy Home rows in both directions.
    repeat(6) {
        device.swipe(
            width / 2,
            (height * 0.82f).toInt(),
            width / 2,
            (height * 0.28f).toInt(),
            14,
        )
        SystemClock.sleep(180)
    }
    repeat(3) {
        device.swipe(
            (width * 0.84f).toInt(),
            (height * 0.68f).toInt(),
            (width * 0.16f).toInt(),
            (height * 0.68f).toInt(),
            12,
        )
        SystemClock.sleep(180)
    }
    repeat(4) {
        device.swipe(
            width / 2,
            (height * 0.30f).toInt(),
            width / 2,
            (height * 0.80f).toInt(),
            14,
        )
        SystemClock.sleep(180)
    }

    if (includeBillboardRotation) {
        // Capture HomeBillboardRuntime's 18-second update and the frame burst after it.
        SystemClock.sleep(19_000)
        repeat(4) {
            device.swipe(
                width / 2,
                (height * 0.82f).toInt(),
                width / 2,
                (height * 0.30f).toInt(),
                14,
            )
            SystemClock.sleep(160)
        }
    }

    device.waitForIdle()
}
