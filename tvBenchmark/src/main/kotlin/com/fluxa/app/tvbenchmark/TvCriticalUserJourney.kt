package com.fluxa.app.tvbenchmark

import android.os.SystemClock
import androidx.benchmark.macro.MacrobenchmarkScope

internal const val TV_PACKAGE_NAME = "com.fluxa.app.tv"

/**
 * D-pad path shared by the frame benchmark and Baseline Profile generator.
 * Keep the test device signed in with a profile whose Home screen contains several rows.
 */
internal fun MacrobenchmarkScope.browseTvHome(
    includeBillboardRotation: Boolean,
) {
    repeat(24) {
        device.pressDPadRight()
        SystemClock.sleep(90)
    }
    repeat(4) {
        device.pressDPadDown()
        SystemClock.sleep(140)
    }
    repeat(16) {
        device.pressDPadLeft()
        SystemClock.sleep(90)
    }

    if (includeBillboardRotation) {
        // HomeBillboardRuntime rotates at 18 seconds. Capture the emission and the next input burst.
        SystemClock.sleep(19_000)
        repeat(12) {
            device.pressDPadRight()
            SystemClock.sleep(90)
        }
    }

    device.waitForIdle()
}
