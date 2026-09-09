@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)

package com.fluxa.app.shared

import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusGroup
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.shared.feature.profile.ProfileDefaultAvatar
import com.fluxa.app.ui.catalog.LocalAccentColor

private data class TvSidebarItem(
    val destination: FluxaDestination,
    val icon: @Composable () -> ImageVector,
    val labelKey: String
)

private val TvSidebarItems = listOf(
    TvSidebarItem(FluxaDestination.Home, { FluxaIcons.Navigation.Home }, "nav.home"),
    TvSidebarItem(FluxaDestination.Discover, { FluxaIcons.Navigation.Discover }, "nav.discover"),
    TvSidebarItem(FluxaDestination.Calendar, { FluxaIcons.Navigation.Calendar }, "nav.calendar"),
    TvSidebarItem(FluxaDestination.Library, { FluxaIcons.Navigation.Library }, "nav.library"),
    TvSidebarItem(FluxaDestination.Settings, { FluxaIcons.Navigation.Settings }, "nav.settings")
)

@Composable
fun TvSidebarNav(
    destination: FluxaDestination,
    language: String?,
    profileAvatarUrl: String?,
    onDestinationSelected: (FluxaDestination) -> Unit,
    rightFocusRequester: FocusRequester? = null,
    onHomeRightPressed: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxHeight()
            .width(80.dp)
            .padding(top = 48.dp, bottom = 20.dp)
            .focusRestorer(),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        TvSidebarItems.forEach { item ->
            TvSidebarButton(
                icon = item.icon(),
                contentDescription = AppStrings.t(language, item.labelKey),
                selected = destination == item.destination,
                showAvatar = item.destination == FluxaDestination.Settings,
                avatarUrl = profileAvatarUrl,
                rightFocusRequester = rightFocusRequester,
                onRightPressed = onHomeRightPressed,
                onClick = { onDestinationSelected(item.destination) }
            )
        }
    }
}

@Composable
private fun TvSidebarButton(
    icon: ImageVector,
    contentDescription: String,
    selected: Boolean,
    showAvatar: Boolean,
    avatarUrl: String?,
    rightFocusRequester: FocusRequester?,
    onRightPressed: (() -> Unit)?,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .padding(horizontal = 16.dp)
            .size(56.dp)
            .focusProperties {
                rightFocusRequester?.let { right = it }
            }
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown && event.key == Key.DirectionRight && onRightPressed != null) {
                    onRightPressed()
                    true
                } else {
                    false
                }
            }
            .onFocusChanged { focused = it.isFocused }
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        if (selected) {
            Box(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .size(width = 24.dp, height = 3.dp)
                    .background(LocalAccentColor.current, RoundedCornerShape(50))
            )
        }
        if (showAvatar && !avatarUrl.isNullOrBlank()) {
            FluxaRemoteImage(
                imageUrl = avatarUrl,
                cacheKey = "tv-sidebar-profile:$avatarUrl",
                contentDescription = contentDescription,
                modifier = Modifier
                    .size(34.dp)
                    .clip(CircleShape)
                    .graphicsLayer { alpha = if (focused) 1f else 0.55f },
                contentScale = ContentScale.Crop,
            )
        } else if (showAvatar) {
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .clip(CircleShape)
                    .background(Color.White.copy(alpha = 0.12f))
                    .padding(7.dp)
            ) {
                ProfileDefaultAvatar(
                    modifier = Modifier,
                    tint = if (focused) Color.White else Color.White.copy(alpha = 0.55f)
                )
            }
        } else {
            Icon(
                imageVector = icon,
                contentDescription = contentDescription,
                modifier = Modifier.size(28.dp),
                tint = if (focused) Color.White else Color.White.copy(alpha = 0.55f)
            )
        }
    }
}
