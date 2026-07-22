package com.fluxa.app.player

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.shared.feature.player.Chapter
import com.google.gson.Gson

private val chapterSkipSegmentsJson = Gson()

fun deriveSkipSegmentsFromChapters(chapters: List<Chapter>): List<IntroTimestamps> =
    FluxaCoreNative.chapterSkipSegments(chapterSkipSegmentsJson.toJson(chapters))
