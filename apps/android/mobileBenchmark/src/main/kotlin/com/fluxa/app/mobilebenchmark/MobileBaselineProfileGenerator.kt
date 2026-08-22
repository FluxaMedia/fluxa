package com.fluxa.app.mobilebenchmark

import androidx.benchmark.macro.junit4.BaselineProfileRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import androidx.test.filters.SdkSuppress
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
@LargeTest
@SdkSuppress(minSdkVersion = 33)
class MobileBaselineProfileGenerator {
    @get:Rule
    val baselineProfileRule = BaselineProfileRule()

    @Test
    fun generateMobileCriticalUserJourney() = baselineProfileRule.collect(
        packageName = MOBILE_PACKAGE_NAME,
        outputFilePrefix = "mobile-home",
        includeInStartupProfile = false,
    ) {
        pressHome()
        startActivityAndWait()
        device.waitForIdle()
        browseMobileHome(includeBillboardRotation = false)
    }
}
