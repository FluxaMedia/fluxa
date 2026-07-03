@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.ui.graphics.Brush
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Icon
import androidx.tv.material3.Surface
import androidx.tv.material3.Text

@Composable
internal fun TvHomeNavRail(
    lang: String,
    onSearchClick: () -> Unit,
    onWatchlistClick: () -> Unit,
    onExploreClick: () -> Unit,
    onProfileClick: () -> Unit,
    contentFocusRequester: FocusRequester?
) {
    val items = listOf(
        HomeNavItem(AppStrings.t(lang, "nav.home"), FluxaIcons.Home, isActive = true, onClick = {}),
        HomeNavItem(AppStrings.t(lang, "auto.search"), FluxaIcons.Search, onClick = onSearchClick),
        HomeNavItem(AppStrings.t(lang, "nav.library"), FluxaIcons.Bookmark, onClick = onWatchlistClick),
        HomeNavItem(AppStrings.t(lang, "nav.discover"), FluxaIcons.Extension, onClick = onExploreClick),
        HomeNavItem(AppStrings.t(lang, "nav.settings"), FluxaIcons.Settings, onClick = onProfileClick)
    )

    Box(modifier = Modifier.fillMaxHeight()) {
        Box(
            modifier = Modifier
                .fillMaxHeight()
                .width(80.dp)
                .background(
                    Brush.horizontalGradient(
                        listOf(Color.Black.copy(alpha = 0.72f), Color.Transparent)
                    )
                )
        )
        Column(
            modifier = Modifier.padding(top = 122.dp, start = 18.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items.forEach { item ->
                NavRailItem(item = item, contentFocusRequester = contentFocusRequester)
            }
        }
    }
}

@Composable
private fun NavRailItem(item: HomeNavItem, contentFocusRequester: FocusRequester?) {
    val interactionSource = remember { MutableInteractionSource() }
    val isFocused by interactionSource.collectIsFocusedAsState()

    Surface(
        onClick = item.onClick,
        modifier = Modifier
            .height(44.dp)
            .then(
                if (contentFocusRequester != null)
                    Modifier.focusProperties { right = contentFocusRequester }
                else Modifier
            ),
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(14.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = Color.Transparent,
            focusedContainerColor = Color.Transparent
        ),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.0f),
        interactionSource = interactionSource
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Icon(
                imageVector = item.icon,
                contentDescription = item.label,
                tint = if (isFocused || item.isActive) Color.White else Color.White.copy(alpha = 0.62f),
                modifier = Modifier.size(22.dp)
            )
        }
    }
}

private data class HomeNavItem(
    val label: String,
    val icon: ImageVector,
    val isActive: Boolean = false,
    val onClick: () -> Unit
)

@Composable
internal fun TvHomeTopBar(
    lang: String,
    onSearchClick: () -> Unit,
    onWatchlistClick: () -> Unit,
    onExploreClick: () -> Unit,
    onProfileClick: () -> Unit,
    contentFocusRequester: FocusRequester?,
    firstItemFocusRequester: FocusRequester? = null
) {
    val items = listOf(
        HomeNavItem(AppStrings.t(lang, "nav.home"), FluxaIcons.Home, isActive = true, onClick = {}),
        HomeNavItem(AppStrings.t(lang, "auto.search"), FluxaIcons.Search, onClick = onSearchClick),
        HomeNavItem(AppStrings.t(lang, "nav.library"), FluxaIcons.Bookmark, onClick = onWatchlistClick),
        HomeNavItem(AppStrings.t(lang, "nav.discover"), FluxaIcons.Extension, onClick = onExploreClick),
        HomeNavItem(AppStrings.t(lang, "nav.settings"), FluxaIcons.Settings, onClick = onProfileClick)
    )

    Box(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .align(Alignment.TopStart)
                .padding(top = 18.dp, start = 24.dp)
                .background(
                    Color(0xFF0A0C14).copy(alpha = 0.96f),
                    RoundedCornerShape(999.dp)
                )
                .padding(6.dp),
            horizontalArrangement = Arrangement.spacedBy(2.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            items.forEachIndexed { index, item ->
                TopBarItem(
                    item = item,
                    contentFocusRequester = contentFocusRequester,
                    focusRequester = if (index == 0) firstItemFocusRequester else null
                )
            }
        }
    }
}

@Composable
private fun TopBarItem(item: HomeNavItem, contentFocusRequester: FocusRequester?, focusRequester: FocusRequester?) {
    val interactionSource = remember { MutableInteractionSource() }
    val isFocused by interactionSource.collectIsFocusedAsState()

    Surface(
        onClick = item.onClick,
        modifier = Modifier
            .height(36.dp)
            .then(if (focusRequester != null) Modifier.focusRequester(focusRequester) else Modifier)
            .then(
                if (contentFocusRequester != null)
                    Modifier.focusProperties { down = contentFocusRequester }
                else Modifier
            ),
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(999.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = Color.Transparent,
            focusedContainerColor = Color.White.copy(alpha = 0.10f)
        ),
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.0f),
        interactionSource = interactionSource
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            Icon(
                imageVector = item.icon,
                contentDescription = item.label,
                tint = if (item.isActive) Color.White else if (isFocused) Color.White.copy(alpha = 0.90f) else Color.White.copy(alpha = 0.65f),
                modifier = Modifier.size(20.dp)
            )
            Text(
                text = item.label,
                color = if (item.isActive) Color.White else if (isFocused) Color.White.copy(alpha = 0.90f) else Color.White.copy(alpha = 0.65f),
                fontSize = 13.sp,
                fontWeight = if (item.isActive) FontWeight.Bold else FontWeight.SemiBold
            )
        }
    }
}
