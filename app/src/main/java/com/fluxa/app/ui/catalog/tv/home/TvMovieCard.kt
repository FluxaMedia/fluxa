@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import android.content.Intent
import android.graphics.drawable.BitmapDrawable
import android.net.Uri
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerDefaults
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.*
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import androidx.compose.ui.viewinterop.AndroidView
import androidx.palette.graphics.Palette
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as mobileGridItems
import androidx.tv.material3.*
import coil3.compose.AsyncImage
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.size.Size
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

@Composable
internal fun TvMovieCard(
    meta: Meta,
    onFocus: (Meta) -> Unit,
    onClick: () -> Unit,
    cardLayout: String,
    onForgetProgress: (() -> Unit)? = null,
    onProgressActions: (() -> Unit)? = null,
    artworkPreference: String? = null,
    profile: UserProfile? = null,
    cardScale: Float = 1f,
    showHorizontalLogo: Boolean = true,
    topTenRank: Int? = null
) {
    val deviceType = LocalDeviceType.current
    var isFocused by remember { mutableStateOf(false) }
    var showForgetOverlay by remember { mutableStateOf(false) }
    val effectiveCardLayout = if (meta.type == "catalog_folder") {
        when (meta.reason) {
            "wide" -> "horizontal"
            "square" -> "square"
            "poster" -> "vertical"
            else -> cardLayout
        }
    } else {
        cardLayout
    }
    val isHorizontal = effectiveCardLayout == "horizontal"
    val isEpisodeStyle = effectiveCardLayout == "episode"
    val radius = posterCornerRadius(profile?.safeCardCornerPreset ?: "soft")
    val animationDuration = when {
        profile?.safeAnimationsEnabled == false -> 0
        else -> 240
    }
    val focusedScale = 1.12f
    val hidePosterTitles = profile?.safePosterHideTitles == true || meta.hideTitle == true
    
    //  %25 LARGER SIZE FOR LIBRARY CARDS (Cumulative)
    //  %15 SMALLER SIZE FOR TV CONTENT (Requested)
    val width = (if (deviceType == DeviceType.TV) {
        when {
            isEpisodeStyle -> 356.dp
            isHorizontal -> horizontalCardWidth(profile?.safePosterWidthPreset ?: "medium", DeviceType.TV)
            else -> 136.dp
        }
    } else {
        when {
            isEpisodeStyle -> 244.dp
            isHorizontal -> horizontalCardWidth(profile?.safePosterWidthPreset ?: "medium", DeviceType.Mobile)
            else -> posterCardWidth(profile?.safePosterWidthPreset ?: "medium")
        }
    }) * cardScale
    val imageHeight = (if (deviceType == DeviceType.TV) {
        when {
            isEpisodeStyle -> 208.dp
            isHorizontal -> horizontalCardHeight(profile?.safePosterWidthPreset ?: "medium", DeviceType.TV)
            else -> 204.dp
        }
    } else {
        when {
            isEpisodeStyle -> 148.dp
            isHorizontal -> horizontalCardHeight(profile?.safePosterWidthPreset ?: "medium", DeviceType.Mobile)
            else -> posterCardHeight(profile?.safePosterWidthPreset ?: "medium")
        }
    }) * cardScale
    val metaHeight = if (deviceType == DeviceType.Mobile && !isEpisodeStyle && !hidePosterTitles) 42.dp else 0.dp
    val height = imageHeight + metaHeight

    val scale by animateFloatAsState(
        targetValue = if (isFocused) focusedScale else 1.0f,
        animationSpec = tween(animationDuration),
        label = "scale"
    )

    Surface(
        onClick = onClick,
        modifier = Modifier
            .width(width)
            .height(height)
            .animateContentSize(animationSpec = tween(animationDuration))
            .zIndex(if (isFocused) 100f else 1f)
            .padding(4.dp)
            .onFocusChanged { isFocused = it.isFocused; if (it.isFocused) onFocus(meta) },
        scale = ClickableSurfaceDefaults.scale(focusedScale = 1.1f),
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(radius)),
        colors = ClickableSurfaceDefaults.colors(containerColor = Color(0xFF12161D), focusedContainerColor = Color(0xFF1B212B))
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .then(
                    if (isFocused) {
                        Modifier.border(
                            3.5.dp,
                            if (meta.focusGlowEnabled == true) Color(profile?.colorArgb ?: 0xFFFFFFFF.toInt()) else Color.White,
                            RoundedCornerShape(radius)
                        )
                    } else {
                        Modifier
                    }
                )
        ) {
            MovieCardContent(meta, isUpcoming(meta.released), isFocused, effectiveCardLayout, artworkPreference, radius, hidePosterTitles, showHorizontalLogo, profile?.safeLanguage, contentWidth = width, contentHeight = imageHeight)
        }
    }
    
}
