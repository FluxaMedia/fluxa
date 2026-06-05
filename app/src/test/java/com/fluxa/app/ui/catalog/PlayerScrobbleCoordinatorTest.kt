package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlayerScrobbleCoordinatorTest {

    @Test
    fun progressPercentIsClampedAndZeroForMissingDuration() {
        assertEquals(0f, PlayerScrobbleCoordinator.progressPercent(1_000L, 0L), 0.001f)
        assertEquals(50f, PlayerScrobbleCoordinator.progressPercent(5_000L, 10_000L), 0.001f)
        assertEquals(100f, PlayerScrobbleCoordinator.progressPercent(20_000L, 10_000L), 0.001f)
    }

    @Test
    fun startRequiresTokenPlayingAndInitialProgress() {
        assertFalse(PlayerScrobbleCoordinator.shouldSendStart(null, true, false, 1f))
        assertFalse(PlayerScrobbleCoordinator.shouldSendStart("token", false, false, 1f))
        assertFalse(PlayerScrobbleCoordinator.shouldSendStart("token", true, true, 1f))
        assertFalse(PlayerScrobbleCoordinator.shouldSendStart("token", true, false, 0.1f))
        assertTrue(PlayerScrobbleCoordinator.shouldSendStart("token", true, false, 0.3f))
    }

    @Test
    fun stopUsesTraktWatchedThreshold() {
        assertFalse(PlayerScrobbleCoordinator.shouldMarkStopped(false, 79.9f))
        assertTrue(PlayerScrobbleCoordinator.shouldMarkStopped(false, 80f))
        assertFalse(PlayerScrobbleCoordinator.shouldMarkStopped(true, 90f))
    }

    @Test
    fun durablePauseAndStopRequireAtLeastOnePercent() {
        assertFalse(PlayerScrobbleCoordinator.shouldEnqueueDurable("pause", "token", 0.5f))
        assertFalse(PlayerScrobbleCoordinator.shouldEnqueueDurable("stop", "token", 0.5f))
        assertTrue(PlayerScrobbleCoordinator.shouldEnqueueDurable("pause", "token", 1f))
        assertTrue(PlayerScrobbleCoordinator.shouldEnqueueDurable("start", "token", 0.1f))
    }

    @Test
    fun periodicAndDisposeSaveUseCentralThresholds() {
        assertFalse(PlayerScrobbleCoordinator.shouldSavePeriodicProgress(true, 30_000L, 0L))
        assertTrue(PlayerScrobbleCoordinator.shouldSavePeriodicProgress(true, 30_001L, 0L))
        assertFalse(PlayerScrobbleCoordinator.shouldSavePeriodicProgress(false, 60_000L, 0L))

        assertFalse(PlayerScrobbleCoordinator.shouldSaveOnDispose(5_000L))
        assertTrue(PlayerScrobbleCoordinator.shouldSaveOnDispose(5_001L))
    }
}
