package com.fluxa.app.shared.feature.watchtogether

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch

class AppleWatchTogetherStateSnapshot(
    val inRoom: Boolean,
    val roomCode: String?,
    val isHost: Boolean,
    val memberNames: List<String>,
    val errorMessage: String?,
    val connecting: Boolean,
)

class AppleWatchTogetherCallbacks(
    val setPlaying: (Boolean) -> Unit,
    val seekTo: (Long) -> Unit,
    val setSpeed: (Float) -> Unit,
    val stateChanged: (AppleWatchTogetherStateSnapshot) -> Unit,
)

class AppleWatchTogetherBridge(
    private val callbacks: AppleWatchTogetherCallbacks,
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val configStore = AppleWatchTogetherConfigStore()
    private var latest = WatchTogetherPlaybackSnapshot(0L, 0L, false, false)
    private val endpoint = LambdaWatchTogetherPlaybackEndpoint(
        snapshotProvider = { latest },
        playingSetter = callbacks.setPlaying,
        seekHandler = callbacks.seekTo,
        speedSetter = callbacks.setSpeed,
    )

    init {
        WatchTogetherManager.installTransportFactory { AppleWatchTogetherTransport() }
        scope.launch {
            WatchTogetherManager.state.collect { state ->
                callbacks.stateChanged(
                    AppleWatchTogetherStateSnapshot(
                        inRoom = state.inRoom,
                        roomCode = state.roomCode,
                        isHost = state.isHost,
                        memberNames = state.members.map { member ->
                            buildString {
                                if (member.isHost) append("★ ")
                                append(member.name)
                                if (member.buffering) append(" · buffering")
                            }
                        },
                        errorMessage = state.errorMessage,
                        connecting = state.connectionState == WatchTogetherConnectionState.CONNECTING,
                    )
                )
            }
        }
    }

    fun loadConfiguration(defaultDisplayName: String): WatchTogetherConfig =
        configStore.loadIntoManager(defaultDisplayName)

    fun configure(serverUrl: String, serverSecret: String, displayName: String) {
        configStore.saveAndConfigure(serverUrl, serverSecret, displayName)
    }

    fun createRoom() = WatchTogetherManager.createRoom()

    fun joinRoom(code: String) = WatchTogetherManager.joinRoom(code)

    fun leaveRoom() = WatchTogetherManager.leaveRoom()

    fun attachPlayback(contentId: String, contentType: String, videoId: String?, title: String) {
        WatchTogetherManager.attachPlayback(
            endpoint,
            WatchTogetherContent(contentId, contentType, videoId, title),
        )
    }

    fun updatePlayback(positionMs: Long, durationMs: Long, isPlaying: Boolean, isBuffering: Boolean) {
        latest = WatchTogetherPlaybackSnapshot(
            positionMs = positionMs.coerceAtLeast(0L),
            durationMs = durationMs.coerceAtLeast(0L),
            isPlaying = isPlaying,
            isBuffering = isBuffering,
        )
    }

    fun detachPlayback() {
        WatchTogetherManager.detachPlayback(endpoint)
    }

    fun notifyPlaybackChanged() = WatchTogetherManager.notifyPlaybackChanged()

    fun close() {
        WatchTogetherManager.detachPlayback(endpoint)
        scope.cancel()
    }
}
