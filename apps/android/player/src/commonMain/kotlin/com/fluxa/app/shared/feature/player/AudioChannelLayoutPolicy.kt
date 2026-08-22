package com.fluxa.app.shared.feature.player

/**
 * Ordered PCM layouts offered to an audio sink when it cannot keep the
 * encoded bitstream. The first layout is the widest layout the route can
 * represent; later entries are explicit, loss-minimizing fallbacks.
 *
 * Keep this policy independent of MPV/Media3 so every PCM fallback path can
 * use the same channel-preservation rule.
 */
object AudioChannelLayoutPolicy {
    fun preferredLayouts(maxPcmChannels: Int): String = when {
        maxPcmChannels >= 8 -> "7.1,6.1,5.1,5.0,quad,3.0,stereo"
        maxPcmChannels >= 7 -> "6.1,5.1,5.0,quad,3.0,stereo"
        maxPcmChannels >= 6 -> "5.1,5.0,quad,3.0,stereo"
        maxPcmChannels >= 5 -> "5.0,quad,3.0,stereo"
        maxPcmChannels >= 4 -> "quad,3.0,stereo"
        maxPcmChannels >= 3 -> "3.0,stereo"
        else -> "stereo"
    }
}
