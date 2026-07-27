package com.fluxa.app.shared.feature.player

import com.fluxa.app.ui.catalog.DeviceType

import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.time.Clock
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
fun Modifier.playerInputControls(
    deviceType: DeviceType,
    hasStartedPlaying: Boolean,
    showControls: Boolean,
    holdToSpeedEnabled: Boolean,
    holdSpeed: Float,
    playbackSpeed: Float,
    onSetSpeed: (Float) -> Unit,
    onRaiseVolume: () -> Unit,
    onLowerVolume: () -> Unit,
    onShowControlsTemp: () -> Unit,
    onHideControls: () -> Unit,
    onHoldSpeedVisibleChanged: (Boolean) -> Unit,
    onRelativeSeek: (Int) -> Unit,
    onClosePlayer: () -> Unit,
    onPinchZoom: (Float) -> Unit = {},
    onPinchZoomGestureStart: () -> Unit = {},
    isLocked: Boolean = false,
    onToggleLockHint: () -> Unit = {},
    onVolumeSwipe: (Float) -> Unit = {},
    onBrightnessSwipe: (Float) -> Unit = {},
    onSettingsSwipeUp: () -> Unit = {},
    onEpisodeSwipeRight: () -> Unit = {}
): Modifier {
    if (isLocked) {
        return onKeyEvent { event ->
            if (event.type == KeyEventType.KeyDown && event.key == Key.Back) {
                onClosePlayer()
                true
            } else {
                false
            }
        }.then(
            if (deviceType == DeviceType.Mobile) {
                Modifier.pointerInput(Unit) {
                    detectTapGestures(onTap = { onToggleLockHint() })
                }
            } else {
                Modifier
            }
        )
    }
    return onKeyEvent { event ->
        if (event.type != KeyEventType.KeyDown) return@onKeyEvent false
        when (event.key) {
            Key.VolumeUp -> {
                onRaiseVolume()
                true
            }
            Key.VolumeDown -> {
                onLowerVolume()
                true
            }
            Key.DirectionLeft -> {
                if (hasStartedPlaying && !showControls) {
                    onRelativeSeek(-1)
                    true
                } else {
                    false
                }
            }
            Key.DirectionRight -> {
                if (hasStartedPlaying && !showControls) {
                    onRelativeSeek(1)
                    true
                } else {
                    false
                }
            }
            Key.DirectionUp, Key.DirectionDown, Key.Enter, Key.DirectionCenter -> {
                if (hasStartedPlaying && !showControls) {
                    onShowControlsTemp()
                    true
                } else {
                    false
                }
            }
            Key.Back -> {
                onClosePlayer()
                true
            }
            else -> false
        }
    }
        .then(
            if (deviceType == DeviceType.Mobile) {
                Modifier.pointerInput(
                    hasStartedPlaying,
                    showControls,
                    holdToSpeedEnabled,
                    holdSpeed,
                    playbackSpeed
                ) {
                    detectTapGestures(
                        onPress = {
                            coroutineScope {
                                val originalSpeed = playbackSpeed
                                var appliedHoldSpeed = false
                                val holdJob = launch {
                                    delay(260)
                                    if (hasStartedPlaying && holdToSpeedEnabled) {
                                        appliedHoldSpeed = true
                                        onHoldSpeedVisibleChanged(true)
                                        onSetSpeed(holdSpeed)
                                    }
                                }
                                tryAwaitRelease()
                                holdJob.cancel()
                                if (appliedHoldSpeed) {
                                    onSetSpeed(originalSpeed)
                                    onHoldSpeedVisibleChanged(false)
                                }
                            }
                        },
                        onTap = {
                            if (!hasStartedPlaying) return@detectTapGestures
                            if (showControls) onHideControls() else onShowControlsTemp()
                        },
                        onDoubleTap = { offset ->
                            if (!hasStartedPlaying) return@detectTapGestures
                            onRelativeSeek(if (offset.x > size.width / 2) 1 else -1)
                            onHideControls()
                        }
                    )
                }.pointerInput(Unit) {
                    var lastZoomMs = 0L
                    detectTransformGestures { _, _, zoom, _ ->
                        if (zoom != 1.0f) {
                            val now = Clock.System.now().toEpochMilliseconds()
                            if (now - lastZoomMs > 350L) onPinchZoomGestureStart()
                            lastZoomMs = now
                            onPinchZoom(zoom)
                        }
                    }
                }.pointerInput(hasStartedPlaying) {
                    if (!hasStartedPlaying) return@pointerInput
                    var gestureTarget = 0
                    var settingsDragDistance = 0f
                    detectVerticalDragGestures(
                        onDragStart = { offset ->
                            val width = size.width.toFloat()
                            val height = size.height.toFloat()
                            gestureTarget = when {
                                offset.x < width * 0.30f -> 1
                                offset.x > width * 0.70f -> 2
                                offset.y > height * 0.70f -> 3
                                else -> 0
                            }
                            settingsDragDistance = 0f
                        },
                        onVerticalDrag = { change, dragAmount ->
                            when (gestureTarget) {
                                1 -> {
                                    change.consume()
                                    onBrightnessSwipe(-dragAmount / size.height.toFloat())
                                }
                                2 -> {
                                    change.consume()
                                    onVolumeSwipe(-dragAmount / size.height.toFloat())
                                }
                                3 -> {
                                    change.consume()
                                    settingsDragDistance += dragAmount
                                }
                            }
                        },
                        onDragEnd = {
                            if (gestureTarget == 3 && settingsDragDistance < -72f) onSettingsSwipeUp()
                        }
                    )
                }.pointerInput(hasStartedPlaying) {
                    if (!hasStartedPlaying) return@pointerInput
                    var episodeDragDistance = 0f
                    var startsAtLeftEdge = false
                    detectHorizontalDragGestures(
                        onDragStart = { offset ->
                            startsAtLeftEdge = offset.x < size.width * 0.24f
                            episodeDragDistance = 0f
                        },
                        onHorizontalDrag = { change, dragAmount ->
                            if (startsAtLeftEdge && dragAmount > 0f) {
                                change.consume()
                                episodeDragDistance += dragAmount
                            }
                        },
                        onDragEnd = {
                            if (startsAtLeftEdge && episodeDragDistance > 96f) onEpisodeSwipeRight()
                        }
                    )
                }
            } else {
                Modifier
            }
        )
        .focusable()
}
