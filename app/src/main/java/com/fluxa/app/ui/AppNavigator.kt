package com.fluxa.app.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.runtime.mutableStateListOf

@Stable
class AppNavigator internal constructor(
    private val screenStack: SnapshotStateList<Screen>
) {
    val currentScreen: Screen
        get() = screenStack.lastOrNull() ?: Screen.Profiles

    fun navigateTo(screen: Screen, clearStack: Boolean = false) {
        if (clearStack) {
            screenStack.clear()
            screenStack.add(screen)
            return
        }
        if (screenStack.lastOrNull() != screen) {
            screenStack.add(screen)
        }
    }

    fun navigateBack() {
        if (screenStack.size > 1) {
            screenStack.removeAt(screenStack.lastIndex)
        } else if (currentScreen !is Screen.Profiles) {
            screenStack.clear()
            screenStack.add(Screen.Profiles)
        }
    }

    fun canNavigateBack(): Boolean = screenStack.size > 1
}

@Composable
fun rememberAppNavigator(initialScreen: Screen = Screen.Profiles): AppNavigator {
    val screenStack = remember { mutableStateListOf<Screen>(initialScreen) }
    return remember(screenStack) { AppNavigator(screenStack) }
}
