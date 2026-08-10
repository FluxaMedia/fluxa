package com.fluxa.app.shared.feature.watchtogether

data class WatchTogetherConfig(
    val serverUrl: String = "",
    val serverSecret: String = "",
    val displayName: String = "Guest",
)

data class WatchTogetherContent(
    val id: String,
    val type: String,
    val videoId: String? = null,
    val title: String = "",
) {
    fun matches(other: WatchTogetherContent): Boolean =
        id == other.id && (videoId == null || other.videoId == null || videoId == other.videoId)
}

data class WatchTogetherMember(
    val id: String,
    val name: String,
    val isHost: Boolean = false,
    val buffering: Boolean = false,
)

enum class WatchTogetherConnectionState { DISCONNECTED, CONNECTING, CONNECTED, ERROR }
enum class WatchTogetherRole { NONE, HOST, GUEST }

data class WatchTogetherState(
    val connectionState: WatchTogetherConnectionState = WatchTogetherConnectionState.DISCONNECTED,
    val roomCode: String? = null,
    val clientId: String? = null,
    val hostId: String? = null,
    val role: WatchTogetherRole = WatchTogetherRole.NONE,
    val members: List<WatchTogetherMember> = emptyList(),
    val content: WatchTogetherContent? = null,
    val errorMessage: String? = null,
) {
    val inRoom: Boolean get() = !roomCode.isNullOrBlank()
    val isHost: Boolean get() = role == WatchTogetherRole.HOST
    val canControlLocally: Boolean get() = !inRoom || isHost
}

data class WatchTogetherPlaybackSnapshot(
    val positionMs: Long,
    val durationMs: Long,
    val isPlaying: Boolean,
    val isBuffering: Boolean = false,
)

interface WatchTogetherPlaybackEndpoint {
    fun snapshot(): WatchTogetherPlaybackSnapshot
    fun setPlaying(playing: Boolean)
    fun seekTo(positionMs: Long)
    fun setSpeed(speed: Float)
}

/**
 * Small adapter used by platform players so endpoint wiring stays declarative instead of each
 * platform re-implementing the same forwarding object.
 */
class LambdaWatchTogetherPlaybackEndpoint(
    private val snapshotProvider: () -> WatchTogetherPlaybackSnapshot,
    private val playingSetter: (Boolean) -> Unit,
    private val seekHandler: (Long) -> Unit,
    private val speedSetter: (Float) -> Unit,
) : WatchTogetherPlaybackEndpoint {
    override fun snapshot(): WatchTogetherPlaybackSnapshot = snapshotProvider()
    override fun setPlaying(playing: Boolean) = playingSetter(playing)
    override fun seekTo(positionMs: Long) = seekHandler(positionMs)
    override fun setSpeed(speed: Float) = speedSetter(speed)
}
