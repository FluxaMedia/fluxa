package com.fluxa.app.shared.feature.calendar

import kotlin.test.Test
import kotlin.test.assertNotEquals

class NotificationsScreenTest {
    @Test
    fun episodesReleasedOnTheSameDayHaveUniqueKeys() {
        assertNotEquals(
            notificationItemKey("tt1", "2026-07-03", "S1:E1"),
            notificationItemKey("tt1", "2026-07-03", "S1:E2")
        )
    }
}
