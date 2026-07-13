package com.fluxa.app.shared.feature.player

import kotlinx.coroutines.flow.StateFlow

class PlayerStore(
    private val controller: PlaybackController
) {
    val state: StateFlow<PlayerUiState> = controller.state

    suspend fun dispatch(action: PlayerAction) {
        controller.dispatch(action)
    }
}
