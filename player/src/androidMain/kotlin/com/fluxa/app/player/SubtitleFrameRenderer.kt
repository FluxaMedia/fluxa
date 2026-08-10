@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.player

import androidx.media3.common.text.Cue
import androidx.media3.ui.SubtitleView
import com.fluxa.app.player.subtitle.SubtitleFrame
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class SubtitleFrameRenderer(private val subtitleView: SubtitleView) {
    fun render(frame: SubtitleFrame) {
        val cues = frame.cues.map { cue -> Cue.Builder().setText(cue.text).build() }
        subtitleView.setCues(cues)
    }
}

fun SubtitleFrameRenderer.collectFrom(scope: CoroutineScope, frames: StateFlow<SubtitleFrame>) {
    scope.launch { frames.collect { render(it) } }
}
