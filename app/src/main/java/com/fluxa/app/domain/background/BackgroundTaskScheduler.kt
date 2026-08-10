package com.fluxa.app.domain.background

interface BackgroundTaskScheduler {
    fun scheduleEpisodeReleaseChecks()
    fun schedulePluginAutoUpdate()
    fun cancelPluginAutoUpdate()
}
