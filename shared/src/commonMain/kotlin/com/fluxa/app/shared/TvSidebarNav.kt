package com.fluxa.app.shared

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.LibraryBooks
import androidx.compose.material.icons.filled.Explore
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Today
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import com.fluxa.app.common.AppStrings

private data class TvSidebarItem(
    val destination: FluxaDestination,
    val icon: ImageVector,
    val labelKey: String
)

private val TvSidebarItems = listOf(
    TvSidebarItem(FluxaDestination.Home, Icons.Filled.Home, "nav.home"),
    TvSidebarItem(FluxaDestination.Search, Icons.Filled.Search, "auto.search"),
    TvSidebarItem(FluxaDestination.Discover, Icons.Filled.Explore, "nav.discover"),
    TvSidebarItem(FluxaDestination.Calendar, Icons.Filled.Today, "nav.calendar"),
    TvSidebarItem(FluxaDestination.Library, Icons.AutoMirrored.Filled.LibraryBooks, "nav.library"),
    TvSidebarItem(FluxaDestination.Settings, Icons.Filled.Settings, "nav.settings")
)

@Composable
fun TvSidebarNav(
    destination: FluxaDestination,
    language: String?,
    onDestinationSelected: (FluxaDestination) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxHeight()
            .width(84.dp)
            .background(Color.Black.copy(alpha = 0.55f))
            .padding(vertical = 24.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically)
    ) {
        TvSidebarItems.forEach { item ->
            TvSidebarButton(
                icon = item.icon,
                contentDescription = AppStrings.t(language, item.labelKey),
                selected = destination == item.destination,
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
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    Box(
        modifier = Modifier
            .padding(horizontal = 14.dp)
            .size(56.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(
                when {
                    focused -> Color.White
                    selected -> Color.White.copy(alpha = 0.18f)
                    else -> Color.Transparent
                }
            )
            .onFocusChanged { focused = it.isFocused }
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = contentDescription,
            tint = if (focused) Color.Black else if (selected) Color.White else Color.White.copy(alpha = 0.7f)
        )
    }
}
