package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import com.fluxa.app.player.TrailerCue

val LocalHeroTrailerSurface = staticCompositionLocalOf<(@Composable (String, List<TrailerCue>, (String) -> Unit, Modifier) -> Unit)?> { null }
