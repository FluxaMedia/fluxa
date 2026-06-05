@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import androidx.tv.material3.Icon

@Composable
internal fun HomeNavRail(
    lang: String,
    onSearchClick: () -> Unit,
    onWatchlistClick: () -> Unit,
    onExploreClick: () -> Unit,
    onProfileClick: () -> Unit,
    contentFocusRequester: FocusRequester?
) {
    val navItems = listOf(
        HomeNavItem(AppStrings.t(lang, "nav.home"), FluxaIcons.Home) {},
        HomeNavItem(AppStrings.t(lang, "auto.search"), FluxaIcons.Search, onSearchClick),
        HomeNavItem(AppStrings.t(lang, "nav.library"), FluxaIcons.Bookmark, onWatchlistClick),
        HomeNavItem(AppStrings.t(lang, "nav.discover"), FluxaIcons.Extension, onExploreClick),
        HomeNavItem(AppStrings.t(lang, "nav.settings"), FluxaIcons.Settings, onProfileClick)
    )

    Column(
        modifier = Modifier
            .width(84.dp)
            .fillMaxHeight()
            .background(
                Brush.horizontalGradient(
                    colors = listOf(
                        Color(0xFF090B12).copy(alpha = 0.96f),
                        Color(0xFF090B12).copy(alpha = 0.56f),
                        Color.Transparent
                    )
                )
            )
            .padding(top = 122.dp, start = 18.dp, end = 14.dp),
        verticalArrangement = Arrangement.spacedBy(18.dp)
    ) {
        navItems.forEach { item ->
            var focused by remember { mutableStateOf(false) }
            val scale by animateFloatAsState(if (focused) 1.08f else 1f, tween(160), label = "navScale")
            Box(
                modifier = Modifier
                    .size(44.dp)
                    .scale(scale)
                    .clip(RoundedCornerShape(14.dp))
                    .background(if (focused) Color.White.copy(alpha = 0.16f) else Color.Transparent)
                    .onFocusChanged { focused = it.isFocused }
                    .then(
                        if (contentFocusRequester != null) {
                            Modifier.focusProperties { right = contentFocusRequester }
                        } else {
                            Modifier
                        }
                    )
                    .clickable { item.onClick() },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = item.icon,
                    contentDescription = item.label,
                    tint = if (focused) Color.White else Color.White.copy(alpha = 0.7f),
                    modifier = Modifier.size(22.dp)
                )
            }
        }
    }
}

private data class HomeNavItem(
    val label: String,
    val icon: ImageVector,
    val onClick: () -> Unit
)
