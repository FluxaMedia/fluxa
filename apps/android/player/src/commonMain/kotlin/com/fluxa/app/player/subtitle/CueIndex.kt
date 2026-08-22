package com.fluxa.app.player.subtitle

class CueIndex(cues: List<TextCue>) {
    private val cues = cues.sortedBy { it.startUs }

    fun activeAt(positionUs: Long): List<TextCue> {
        if (cues.isEmpty()) return emptyList()
        val insertion = upperBoundByStart(positionUs)
        val result = mutableListOf<TextCue>()
        var i = insertion - 1
        while (i >= 0 && cues[i].endUs > positionUs) {
            result += cues[i]
            i--
        }
        return result.asReversed()
    }

    fun nextBoundaryUs(positionUs: Long): Long? {
        val nextStart = cues.getOrNull(upperBoundByStart(positionUs))?.startUs
        val nextEnd = activeAt(positionUs).minOfOrNull { it.endUs }
        return listOfNotNull(nextStart, nextEnd).minOrNull()
    }

    private fun upperBoundByStart(positionUs: Long): Int {
        var lo = 0
        var hi = cues.size
        while (lo < hi) {
            val mid = (lo + hi) / 2
            if (cues[mid].startUs <= positionUs) lo = mid + 1 else hi = mid
        }
        return lo
    }
}
