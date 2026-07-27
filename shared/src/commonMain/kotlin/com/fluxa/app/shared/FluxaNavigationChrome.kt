package com.fluxa.app.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.ui.focus.focusRestorer
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState
import kotlinx.coroutines.delay
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.ui.draw.alpha
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.catalog.CatalogAction
import com.fluxa.app.shared.feature.catalog.CatalogBillboardUiModel
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CategoryResultsScreen
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.shared.feature.addonstore.AddonStoreAction
import com.fluxa.app.shared.feature.addonstore.AddonStoreScreen
import com.fluxa.app.shared.feature.addonstore.AddonStoreUiState
import com.fluxa.app.shared.feature.auth.AuthAction
import com.fluxa.app.shared.feature.auth.AuthScreen
import com.fluxa.app.shared.feature.auth.AuthUiState
import com.fluxa.app.shared.feature.calendar.CalendarAction
import com.fluxa.app.shared.feature.calendar.CalendarScreen
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import com.fluxa.app.shared.feature.calendar.NotificationsScreen
import com.fluxa.app.shared.feature.detail.DetailAction
import com.fluxa.app.shared.feature.detail.DetailRequestUiModel
import com.fluxa.app.shared.feature.detail.DetailScreen
import com.fluxa.app.shared.feature.detail.DetailUiState
import com.fluxa.app.shared.feature.detail.SourceSelectionScreen
import com.fluxa.app.shared.feature.discover.DiscoverAction
import com.fluxa.app.shared.feature.discover.DiscoverScreen
import com.fluxa.app.shared.feature.discover.DiscoverUiState
import com.fluxa.app.shared.feature.library.LibraryAction
import com.fluxa.app.shared.feature.library.LibraryFolderDetailScreen
import com.fluxa.app.shared.feature.library.LibraryScreen
import com.fluxa.app.shared.feature.library.LibraryUiState
import com.fluxa.app.shared.feature.plugins.PluginsAction
import com.fluxa.app.shared.feature.plugins.PluginsScreen
import com.fluxa.app.shared.feature.plugins.PluginsUiState
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileEditScreen
import com.fluxa.app.shared.feature.profile.ProfileEditTarget
import com.fluxa.app.shared.feature.profile.ProfileEditUiModel
import com.fluxa.app.shared.feature.profile.ProfileListScreen
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.shared.feature.settings.SettingsAction
import com.fluxa.app.shared.feature.settings.SettingsScreen
import com.fluxa.app.shared.feature.settings.SettingsUiState
import com.fluxa.app.shared.feature.search.SearchAction
import com.fluxa.app.shared.feature.search.SearchScreen
import com.fluxa.app.shared.feature.search.SearchUiState
import com.fluxa.app.shared.feature.player.PlayerControlsSurface
import com.fluxa.app.shared.feature.player.PlayerRenderAction
import com.fluxa.app.shared.feature.player.PlayerRenderState
import com.fluxa.app.shared.feature.player.TrailerCue
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.PosterActionSheet

private data class FluxaBottomNavItem(
    val destination: FluxaDestination,
    val selectedIcon: ImageVector,
    val icon: ImageVector
)

private val FluxaBottomNavItems = listOf(
    FluxaBottomNavItem(FluxaDestination.Home, FluxaIcons.BottomHome, FluxaIcons.BottomHomeOutline),
    FluxaBottomNavItem(FluxaDestination.Discover, FluxaIcons.BottomDiscover, FluxaIcons.BottomDiscoverOutline),
    FluxaBottomNavItem(FluxaDestination.Calendar, FluxaIcons.BottomCalendar, FluxaIcons.BottomCalendarOutline),
    FluxaBottomNavItem(FluxaDestination.Library, FluxaIcons.BottomLibrary, FluxaIcons.BottomLibraryOutline)
)

@Composable
internal fun FluxaNavigationRail(
    destination: FluxaDestination,
    accentColorArgb: Long?,
    showProfile: Boolean,
    showLabels: Boolean,
    language: String?,
    onDestinationSelected: (FluxaDestination) -> Unit,
    modifier: Modifier = Modifier
) {
    val selectedColor = accentColorArgb?.let { Color(it) } ?: Color.White
    val inactiveColor = Color(0xFFA0A5AD)
    val items = if (showProfile) {
        FluxaBottomNavItems + FluxaBottomNavItem(FluxaDestination.Settings, FluxaIcons.BottomSettings, FluxaIcons.BottomSettingsOutline)
    } else {
        FluxaBottomNavItems
    }
    Column(
        modifier = modifier
            .fillMaxHeight()
            .width(if (showLabels) 96.dp else 80.dp)
            .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Start))
            .background(Color(0xFF101012))
            .padding(vertical = 24.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        items.forEach { item ->
            val isSelected = item.destination == destination
            val tint by animateColorAsState(
                targetValue = if (isSelected) selectedColor else inactiveColor,
                label = "rail-item-tint"
            )
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(18.dp))
                    .clickable { onDestinationSelected(item.destination) }
                    .background(if (isSelected) Color.White.copy(alpha = 0.08f) else Color.Transparent)
                    .padding(vertical = 10.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                Icon(
                    if (isSelected) item.selectedIcon else item.icon,
                    contentDescription = null,
                    tint = tint,
                    modifier = Modifier.size(28.dp)
                )
                if (showLabels) {
                    Text(
                        text = AppStrings.t(language, item.destination.titleKey),
                        color = tint,
                        fontSize = 10.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }
    }
}

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable
internal fun FluxaTopNavBar(
    destination: FluxaDestination,
    accentColorArgb: Long?,
    showProfile: Boolean,
    showLabels: Boolean,
    isTv: Boolean,
    language: String?,
    onDestinationSelected: (FluxaDestination) -> Unit,
    modifier: Modifier = Modifier
) {
    val selectedColor = accentColorArgb?.let { Color(it) } ?: Color.White
    val inactiveColor = Color(0xFFA0A5AD)
    val items = if (showProfile) {
        FluxaBottomNavItems + FluxaBottomNavItem(FluxaDestination.Settings, FluxaIcons.BottomSettings, FluxaIcons.BottomSettingsOutline)
    } else {
        FluxaBottomNavItems
    }
    Row(
        modifier = modifier
            .fillMaxWidth()
            .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top))
            .background(Color(0xFF101012))
            .padding(horizontal = if (isTv) 48.dp else 20.dp, vertical = if (isTv) 16.dp else 10.dp)
            .then(if (isTv) Modifier.focusRestorer() else Modifier),
        horizontalArrangement = Arrangement.spacedBy(if (isTv) 12.dp else 6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        items.forEach { item ->
            val isSelected = item.destination == destination
            val tint by animateColorAsState(
                targetValue = if (isSelected) selectedColor else inactiveColor,
                label = "top-nav-item-tint"
            )
            val label = AppStrings.t(language, item.destination.titleKey)
            Row(
                modifier = Modifier
                    .clip(RoundedCornerShape(18.dp))
                    .clickable { onDestinationSelected(item.destination) }
                    .background(if (isSelected) Color.White.copy(alpha = 0.08f) else Color.Transparent)
                    .padding(horizontal = 16.dp, vertical = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    if (isSelected) item.selectedIcon else item.icon,
                    contentDescription = if (showLabels || isTv) null else label,
                    tint = tint,
                    modifier = Modifier.size(if (isTv) 28.dp else 24.dp)
                )
                if (showLabels || isTv) {
                    Text(
                        text = label,
                        color = tint,
                        fontSize = if (isTv) 15.sp else 13.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }
    }
}

@Composable
internal fun FluxaNavigationBar(
    destination: FluxaDestination,
    accentColorArgb: Long?,
    floating: Boolean,
    liquidGlass: Boolean,
    hazeState: dev.chrisbanes.haze.HazeState,
    showLabels: Boolean,
    showProfile: Boolean,
    profileAvatarUrl: String?,
    language: String?,
    onDestinationSelected: (FluxaDestination) -> Unit,
    modifier: Modifier = Modifier
) {
    val selectedColor = accentColorArgb?.let { Color(it) } ?: Color.White
    val inactiveColor = if (liquidGlass) Color.White.copy(alpha = 0.78f) else Color(0xFFA0A5AD)
    val barShape = RoundedCornerShape(if (floating) 28.dp else 0.dp)
    val items = if (showProfile) {
        FluxaBottomNavItems + FluxaBottomNavItem(FluxaDestination.Settings, FluxaIcons.BottomSettings, FluxaIcons.BottomSettingsOutline)
    } else {
        FluxaBottomNavItems
    }
    Box(
        modifier = modifier
            .fillMaxWidth()
            .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom))
            .padding(horizontal = if (floating) 12.dp else 0.dp)
            .padding(bottom = if (floating) 10.dp else 0.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .then(
                    if (liquidGlass) {
                        Modifier.shadow(
                            elevation = 18.dp,
                            shape = barShape,
                            ambientColor = Color.Black.copy(alpha = 0.35f),
                            spotColor = Color.Black.copy(alpha = 0.45f)
                        )
                    } else {
                        Modifier
                    }
                )
                .clip(barShape)
                .then(
                    if (liquidGlass) {
                        Modifier
                            .hazeEffect(
                                state = hazeState,
                                style = HazeStyle(
                                    backgroundColor = if (floating) Color(0xFF1C1C1E) else Color(0xFF101012),
                                    tints = listOf(
                                        HazeTint(Color.Black.copy(alpha = 0.4f)),
                                        HazeTint(Color.White.copy(alpha = 0.05f))
                                    ),
                                    blurRadius = 26.dp,
                                    noiseFactor = 0.1f
                                )
                            ) {
                                progressive = dev.chrisbanes.haze.HazeProgressive.RadialGradient(
                                    centerIntensity = 1f,
                                    radiusIntensity = 0.55f
                                )
                            }
                            .border(
                                width = 1.dp,
                                brush = Brush.verticalGradient(
                                    listOf(Color(0x73FFFFFF), Color.Transparent, Color(0x59000000))
                                ),
                                shape = barShape
                            )
                            .drawWithContent {
                                drawRect(
                                    Brush.verticalGradient(
                                        0f to Color.White.copy(alpha = 0.10f),
                                        0.22f to Color.Transparent,
                                        0.78f to Color.Transparent,
                                        1f to Color.Black.copy(alpha = 0.16f)
                                    )
                                )
                                drawContent()
                            }
                    } else {
                        Modifier.background(if (floating) Color(0xF2222222) else Color(0xFF111111))
                    }
                )
                .padding(horizontal = 12.dp)
                .height(if (showLabels) 68.dp else 64.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically
        ) {
            items.forEach { item ->
                val isSelected = item.destination == destination
                val tint by animateColorAsState(
                    targetValue = if (isSelected) selectedColor else inactiveColor,
                    label = "nav-item-tint"
                )
                Column(
                    modifier = Modifier
                        .weight(1f)
                        .clip(RoundedCornerShape(18.dp))
                        .clickable { onDestinationSelected(item.destination) }
                        .padding(vertical = 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        if (liquidGlass && isSelected) {
                            Box(
                                modifier = Modifier
                                    .size(50.dp)
                                    .background(
                                        Brush.radialGradient(
                                            listOf(selectedColor.copy(alpha = 0.28f), Color.Transparent)
                                        ),
                                        CircleShape
                                    )
                            )
                            Box(
                                modifier = Modifier
                                    .size(38.dp)
                                    .background(Color.White.copy(alpha = 0.1f), CircleShape)
                                    .border(1.dp, selectedColor.copy(alpha = 0.4f), CircleShape)
                            )
                        }
                        if (item.destination == FluxaDestination.Settings && !profileAvatarUrl.isNullOrBlank()) {
                            FluxaRemoteImage(
                                imageUrl = profileAvatarUrl,
                                cacheKey = "profile-avatar:$profileAvatarUrl",
                                contentDescription = null,
                                modifier = Modifier
                                    .size(32.dp)
                                    .then(
                                        if (isSelected) {
                                            Modifier.border(2.dp, tint, CircleShape).padding(2.dp)
                                        } else {
                                            Modifier
                                        }
                                    )
                                    .clip(CircleShape),
                                contentScale = ContentScale.Crop
                            )
                        } else {
                            Icon(
                                if (isSelected) item.selectedIcon else item.icon,
                                contentDescription = null,
                                tint = tint,
                                modifier = Modifier.size(29.dp)
                            )
                        }
                    }
                    if (showLabels) {
                        Text(
                            text = AppStrings.t(language, item.destination.titleKey),
                            color = tint,
                            fontSize = 10.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
        }
    }
}
