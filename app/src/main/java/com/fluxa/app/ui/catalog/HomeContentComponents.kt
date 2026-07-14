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
import androidx.lifecycle.compose.collectAsStateWithLifecycle
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
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.PlatformTextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
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
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

@Composable
fun CategoryResultsScreen(
    activeProfile: UserProfile?,
    categoryId: String,
    title: String,
    onMovieClick: (Meta, String?, String?) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel
) {
    val deviceType = LocalDeviceType.current
    val categories by viewModel.categories.collectAsStateWithLifecycle()
    val collectionFolderCategories by viewModel.collectionFolderCategories.collectAsStateWithLifecycle()
    val category = categories.firstOrNull { it.id == categoryId } ?: collectionFolderCategories[categoryId]
    val items = category?.items.orEmpty()
    val layout = category?.let { resolveHomeCardLayout(it, activeProfile) }
        ?: if (activeProfile?.safePosterLandscapeMode == true) "horizontal" else activeProfile?.safeCardLayout ?: "vertical"

    LaunchedEffect(category?.id, items.size) {
        if (category != null && category.canLoadMore && items.isEmpty()) {
            viewModel.loadMore(category.id)
        }
    }

    if (deviceType == DeviceType.Mobile) {
        MobileCategoryResultsContent(
            activeProfile = activeProfile,
            title = title,
            category = category,
            items = items,
            layout = layout,
            onMovieClick = { meta ->
                val source = category?.resultSources?.get("${meta.type}:${meta.id}") ?: category?.resultSources?.get(meta.id)
                val fallback = if (category?.remoteSources.isNullOrEmpty()) category?.catalogSources?.firstOrNull() else null
                onMovieClick(meta, source?.transportUrl ?: fallback?.transportUrl, source?.type ?: fallback?.type)
            },
            onBack = onBack,
            viewModel = viewModel
        )
    } else {
        TvCategoryResultsContent(
            activeProfile = activeProfile,
            title = title,
            category = category,
            items = items,
            layout = layout,
            onMovieClick = { meta ->
                val source = category?.resultSources?.get("${meta.type}:${meta.id}") ?: category?.resultSources?.get(meta.id)
                val fallback = if (category?.remoteSources.isNullOrEmpty()) category?.catalogSources?.firstOrNull() else null
                onMovieClick(meta, source?.transportUrl ?: fallback?.transportUrl, source?.type ?: fallback?.type)
            },
            onBack = onBack,
            viewModel = viewModel
        )
    }
}

fun Meta.matchesFilter(filter: String): Boolean {
    return when (filter) {
        "movie" -> type == "movie"
        "series" -> type == "series" || type == "tv" || type == "anime"
        else -> true
    }
}


@Composable
fun MovieCard(
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
    topTenRank: Int? = null,
    loadArtwork: Boolean = true
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileMovieCard(
            meta = meta,
            onFocus = onFocus,
            onClick = onClick,
            cardLayout = cardLayout,
            onForgetProgress = onForgetProgress,
            onProgressActions = onProgressActions,
            artworkPreference = artworkPreference,
            profile = profile,
            cardScale = cardScale,
            showHorizontalLogo = showHorizontalLogo,
            topTenRank = topTenRank,
            loadArtwork = loadArtwork
        )
    } else {
        TvMovieCard(
            meta = meta,
            onFocus = onFocus,
            onClick = onClick,
            cardLayout = cardLayout,
            onForgetProgress = onForgetProgress,
            onProgressActions = onProgressActions,
            artworkPreference = artworkPreference,
            profile = profile,
            cardScale = cardScale,
            showHorizontalLogo = showHorizontalLogo,
            topTenRank = topTenRank
        )
    }
}
