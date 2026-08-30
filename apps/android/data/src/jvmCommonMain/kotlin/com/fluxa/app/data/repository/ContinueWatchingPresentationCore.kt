package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Meta

fun Meta.continueWatchingEpisodeLabelFromCore(): String {
    val line = FluxaCoreNative.formatEpisodeLine(lastEpisodeName, null, null, lastVideoId)
    val match = Regex("^S(\\d+):E(\\d+)\\s*").find(line) ?: return line
    return "S${match.groupValues[1]} E${match.groupValues[2]} · " + line.removePrefix(match.value)
}
