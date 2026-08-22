package com.fluxa.app.shared.feature.watchtogether

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlinx.serialization.json.JsonObject

/**
 * Shared Watch Together coordinator.
 *
 * Media never traverses the Watch Together server. The server only carries room metadata and
 * authoritative playback state. Every participant resolves/plays the stream locally in Fluxa.
 */
object WatchTogetherManager {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val controlScope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val _state = MutableStateFlow(WatchTogetherState())
    val state: StateFlow<WatchTogetherState> = _state.asStateFlow()

    private val _config = MutableStateFlow(WatchTogetherConfig())
    val config: StateFlow<WatchTogetherConfig> = _config.asStateFlow()

    private var factory: WatchTogetherTransportFactory? = null
    private var transport: WatchTogetherTransport? = null
    private var pendingAction: PendingRoomAction? = null
    private var endpoint: WatchTogetherPlaybackEndpoint? = null
    private var endpointContent: WatchTogetherContent? = null
    private var heartbeatJob: Job? = null
    private var syncApplyJob: Job? = null
    private var lastHostPlaying: Boolean? = null
    private var lastHostPositionMs: Long = -1L
    private var lastHostSentAt: Long = 0L
    private var lastGuestBuffering: Boolean? = null
    private var lastAppliedSequence: Long = -1L
    private var correctionSpeedActive = false
    private var serverClockOffsetMs = 0L
    private var hasServerClockOffset = false
    private var pausedForRemoteBuffering = false
    private var resumeAfterRemoteBuffering = false

    fun installTransportFactory(value: WatchTogetherTransportFactory) {
        factory = value
    }

    fun configure(serverUrl: String, serverSecret: String = "", displayName: String = "Guest") {
        _config.value = WatchTogetherAddress.sanitizeConfig(serverUrl, serverSecret, displayName)
    }

    fun createRoom() = connectAndRun(PendingRoomAction.Create)

    fun joinRoom(code: String) {
        val normalized = WatchTogetherAddress.normalizeRoomCode(code)
        if (!WatchTogetherAddress.isValidRoomCode(normalized)) {
            _state.value = _state.value.copy(errorMessage = "Enter a valid room code")
            return
        }
        connectAndRun(PendingRoomAction.Join(normalized))
    }

    fun leaveRoom() {
        sendObject(WatchTogetherProtocol.leave())
        stopHeartbeat()
        resetCorrectionSpeed()
        transport?.close()
        transport = null
        pendingAction = null
        _state.value = WatchTogetherState()
    }

    fun clearError() {
        _state.value = _state.value.copy(errorMessage = null)
    }

    fun attachPlayback(endpoint: WatchTogetherPlaybackEndpoint, content: WatchTogetherContent) {
        this.endpoint = endpoint
        this.endpointContent = content
        if (_state.value.inRoom) {
            if (_state.value.isHost) sendContent(content)
            startHeartbeat()
        }
    }

    fun updatePlaybackContent(content: WatchTogetherContent) {
        endpointContent = content
        if (_state.value.inRoom && _state.value.isHost) sendContent(content)
    }

    fun detachPlayback(endpoint: WatchTogetherPlaybackEndpoint? = null) {
        if (endpoint != null && this.endpoint !== endpoint) return
        stopHeartbeat()
        resetCorrectionSpeed()
        this.endpoint = null
        this.endpointContent = null
    }

    /** Used by player UI after an explicit host play/pause/seek for low-latency propagation. */
    fun notifyPlaybackChanged() {
        if (!_state.value.isHost) return
        sendHostState(force = true)
        // Player state flows can update a frame after the UI command. Send one follow-up snapshot
        // so a play/pause/seek is not delayed until the normal heartbeat.
        scope.launch {
            delay(100L)
            if (_state.value.isHost) sendHostState(force = true)
        }
    }

    private fun connectAndRun(action: PendingRoomAction) {
        val cfg = _config.value
        val resolved = WatchTogetherAddress.websocketUrl(cfg.serverUrl, cfg.serverSecret)
        if (resolved == null) {
            _state.value = WatchTogetherState(
                connectionState = WatchTogetherConnectionState.ERROR,
                errorMessage = "Enter a valid http(s):// or ws(s):// server URL",
            )
            return
        }
        val transportFactory = factory
        if (transportFactory == null) {
            _state.value = WatchTogetherState(
                connectionState = WatchTogetherConnectionState.ERROR,
                errorMessage = "Watch Together networking is unavailable on this platform",
            )
            return
        }

        transport?.close()
        transport = null
        pendingAction = action
        stopHeartbeat()
        _state.value = WatchTogetherState(connectionState = WatchTogetherConnectionState.CONNECTING)

        val next = transportFactory.create()
        transport = next
        next.connect(resolved, object : WatchTogetherTransport.Listener {
            override fun onOpen() {
                _state.value = _state.value.copy(
                    connectionState = WatchTogetherConnectionState.CONNECTED,
                    errorMessage = null,
                )
                pendingAction.also { pendingAction = null }?.let(::sendPendingAction)
            }

            override fun onMessage(text: String) = handleMessage(text)

            override fun onClosed() {
                if (transport !== next) return
                stopHeartbeat()
                resetCorrectionSpeed()
                transport = null
                _state.value = WatchTogetherState(connectionState = WatchTogetherConnectionState.DISCONNECTED)
            }

            override fun onFailure(message: String) {
                if (transport !== next) return
                stopHeartbeat()
                resetCorrectionSpeed()
                pendingAction = null
                transport = null
                next.close()
                _state.value = _state.value.copy(
                    connectionState = WatchTogetherConnectionState.ERROR,
                    errorMessage = message,
                )
            }
        })
    }

    private sealed interface PendingRoomAction {
        data object Create : PendingRoomAction
        data class Join(val roomCode: String) : PendingRoomAction
    }

    private fun sendPendingAction(action: PendingRoomAction) {
        val displayName = _config.value.displayName
        sendObject(
            when (action) {
                PendingRoomAction.Create -> WatchTogetherProtocol.create(displayName)
                is PendingRoomAction.Join -> WatchTogetherProtocol.join(action.roomCode, displayName)
            }
        )
    }

    private fun handleMessage(text: String) {
        val obj = WatchTogetherProtocol.parse(text) ?: return
        when (WatchTogetherProtocol.messageType(obj)) {
            WatchTogetherProtocol.ROOM -> handleRoom(obj)
            WatchTogetherProtocol.MEMBERS -> handleMembers(obj)
            WatchTogetherProtocol.SYNC -> handleSync(obj)
            WatchTogetherProtocol.BUFFERING -> handleBuffering(obj)
            WatchTogetherProtocol.CONTENT -> handleContent(obj)
            WatchTogetherProtocol.ERROR -> _state.value = _state.value.copy(
                errorMessage = WatchTogetherProtocol.errorMessage(obj) ?: "Watch Together error"
            )
            WatchTogetherProtocol.PONG -> handlePong(obj)
        }
    }

    private fun handlePong(obj: JsonObject) {
        val sentAt = WatchTogetherProtocol.clientTimeMs(obj) ?: return
        val serverTime = WatchTogetherProtocol.serverTimeMs(obj) ?: return
        val receivedAt = nowMs()
        val midpoint = sentAt + ((receivedAt - sentAt).coerceAtLeast(0L) / 2L)
        val sample = serverTime - midpoint
        serverClockOffsetMs = if (hasServerClockOffset) {
            // Smooth jitter without making clock correction sluggish.
            ((serverClockOffsetMs * 3L) + sample) / 4L
        } else {
            sample
        }
        hasServerClockOffset = true
    }

    private fun handleRoom(obj: JsonObject) {
        val clientId = WatchTogetherProtocol.clientId(obj)
        val hostId = WatchTogetherProtocol.hostId(obj)
        val room = WatchTogetherProtocol.roomCode(obj)
        val members = WatchTogetherProtocol.members(obj)
        _state.value = _state.value.copy(
            connectionState = WatchTogetherConnectionState.CONNECTED,
            roomCode = room,
            clientId = clientId,
            hostId = hostId,
            role = if (clientId != null && clientId == hostId) WatchTogetherRole.HOST else WatchTogetherRole.GUEST,
            members = members,
            errorMessage = null,
        )
        if (_state.value.isHost) endpointContent?.let(::sendContent)
        startHeartbeat()
        if (_state.value.isHost) sendHostState(force = true)
    }

    private fun handleMembers(obj: JsonObject) {
        val hostId = WatchTogetherProtocol.hostId(obj) ?: _state.value.hostId
        val clientId = _state.value.clientId
        val wasHost = _state.value.isHost
        _state.value = _state.value.copy(
            hostId = hostId,
            role = if (clientId != null && clientId == hostId) WatchTogetherRole.HOST else WatchTogetherRole.GUEST,
            members = WatchTogetherProtocol.members(obj),
        )
        if (!wasHost && _state.value.isHost) {
            endpointContent?.let(::sendContent)
            sendHostState(force = true)
        }
    }

    private fun handleContent(obj: JsonObject) {
        val content = WatchTogetherProtocol.contentFrom(obj) ?: return
        _state.value = _state.value.copy(content = content)
    }

    private fun handleBuffering(obj: JsonObject) {
        if (!_state.value.isHost) return
        val anyBuffering = WatchTogetherProtocol.anyBuffering(obj) ?: false
        controlScope.launch {
            val localEndpoint = endpoint ?: return@launch
            if (anyBuffering && !pausedForRemoteBuffering) {
                val local = localEndpoint.snapshot()
                pausedForRemoteBuffering = true
                resumeAfterRemoteBuffering = local.isPlaying
                if (local.isPlaying) localEndpoint.setPlaying(false)
                delay(40L)
                sendHostState(force = true)
            } else if (!anyBuffering && pausedForRemoteBuffering) {
                val shouldResume = resumeAfterRemoteBuffering
                pausedForRemoteBuffering = false
                resumeAfterRemoteBuffering = false
                if (shouldResume) localEndpoint.setPlaying(true)
                delay(40L)
                sendHostState(force = true)
            }
        }
    }

    private fun handleSync(obj: JsonObject) {
        val sequence = WatchTogetherProtocol.sequence(obj) ?: 0L
        if (sequence <= lastAppliedSequence) return
        lastAppliedSequence = sequence

        val remoteContent = WatchTogetherProtocol.contentFrom(obj)
        if (remoteContent != null) _state.value = _state.value.copy(content = remoteContent)
        if (_state.value.isHost && WatchTogetherProtocol.senderId(obj) != "server") return
        val localEndpoint = endpoint ?: return
        val localContent = endpointContent
        if (remoteContent != null && localContent != null && !remoteContent.matches(localContent)) return

        val position = WatchTogetherProtocol.positionMs(obj) ?: return
        val playing = WatchTogetherProtocol.playing(obj) ?: false
        val serverTime = WatchTogetherProtocol.serverTimeMs(obj) ?: serverNowMs()
        val expected = if (playing && hasServerClockOffset) {
            position + (serverNowMs() - serverTime).coerceIn(0L, 5_000L)
        } else {
            position
        }

        syncApplyJob?.cancel()
        syncApplyJob = controlScope.launch {
            val local = localEndpoint.snapshot()
            if (local.isPlaying != playing) localEndpoint.setPlaying(playing)
            when (val correction = WatchTogetherDriftCorrector.correction(
                localPositionMs = local.positionMs,
                expectedPositionMs = expected,
                hostPlaying = playing,
                speedCorrectionActive = correctionSpeedActive,
            )) {
                WatchTogetherCorrection.None -> Unit
                is WatchTogetherCorrection.Seek -> {
                    correctionSpeedActive = false
                    localEndpoint.setSpeed(WatchTogetherDriftCorrector.NORMAL_SPEED)
                    localEndpoint.seekTo(correction.positionMs)
                }
                is WatchTogetherCorrection.Speed -> {
                    correctionSpeedActive = true
                    localEndpoint.setSpeed(correction.value)
                }
                WatchTogetherCorrection.ResetSpeed -> {
                    correctionSpeedActive = false
                    localEndpoint.setSpeed(WatchTogetherDriftCorrector.NORMAL_SPEED)
                }
            }
        }
    }

    private fun startHeartbeat() {
        if (heartbeatJob?.isActive == true) return
        heartbeatJob = scope.launch {
            while (isActive && _state.value.inRoom) {
                val local = endpoint?.snapshot()
                if (local != null) {
                    if (_state.value.isHost) {
                        sendHostState(force = false)
                    } else if (lastGuestBuffering != local.isBuffering) {
                        lastGuestBuffering = local.isBuffering
                        sendObject(WatchTogetherProtocol.buffering(local.isBuffering))
                    }
                }
                sendObject(WatchTogetherProtocol.ping(nowMs()))
                delay(1_000L)
            }
        }
    }

    private fun stopHeartbeat() {
        heartbeatJob?.cancel()
        heartbeatJob = null
        syncApplyJob?.cancel()
        syncApplyJob = null
        lastHostPlaying = null
        lastHostPositionMs = -1L
        lastHostSentAt = 0L
        lastGuestBuffering = null
        lastAppliedSequence = -1L
        serverClockOffsetMs = 0L
        hasServerClockOffset = false
        pausedForRemoteBuffering = false
        resumeAfterRemoteBuffering = false
    }

    private fun sendHostState(force: Boolean) {
        if (!_state.value.isHost) return
        val local = endpoint?.snapshot() ?: return
        val now = nowMs()
        val stateChanged = lastHostPlaying != local.isPlaying
        val jumped = lastHostPositionMs >= 0L && kotlin.math.abs(local.positionMs - lastHostPositionMs) > 3_500L
        if (!force && !stateChanged && !jumped && now - lastHostSentAt < 2_000L) return
        lastHostPlaying = local.isPlaying
        lastHostPositionMs = local.positionMs
        lastHostSentAt = now
        sendObject(WatchTogetherProtocol.playbackState(local, endpointContent))
    }

    private fun sendContent(content: WatchTogetherContent) {
        if (!_state.value.inRoom || !_state.value.isHost) return
        sendObject(WatchTogetherProtocol.content(content))
    }

    private fun sendObject(obj: JsonObject) {
        transport?.send(obj.toString())
    }

    private fun resetCorrectionSpeed() {
        if (!correctionSpeedActive) return
        correctionSpeedActive = false
        controlScope.launch { endpoint?.setSpeed(WatchTogetherDriftCorrector.NORMAL_SPEED) }
    }

    @OptIn(ExperimentalTime::class)
    private fun nowMs(): Long = Clock.System.now().toEpochMilliseconds()
    private fun serverNowMs(): Long = nowMs() + if (hasServerClockOffset) serverClockOffsetMs else 0L

}
