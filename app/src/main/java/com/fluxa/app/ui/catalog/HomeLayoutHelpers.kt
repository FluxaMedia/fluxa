@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)
@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.foundation.ExperimentalFoundationApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
@file:kotlin.jvm.JvmName("SharedHomeScreenKt")
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.distinctUntilChanged
import androidx.compose.runtime.snapshotFlow
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.abs

const val CONTINUE_WATCHING_CATEGORY_ID = "continue_watching"

fun HomeCategory.isContinueWatchingCategory(): Boolean = id == CONTINUE_WATCHING_CATEGORY_ID

fun resolveHomeCardLayout(category: HomeCategory, profile: UserProfile?): String {
    return if (category.isContinueWatchingCategory()) {
        profile?.resolvedContinueWatchingLayout ?: "horizontal"
    } else {
        if (profile?.safePosterLandscapeMode == true) "horizontal" else profile?.safeCardLayout ?: "vertical"
    }
}

fun resolveContinueWatchingArtworkPreference(category: HomeCategory, profile: UserProfile?): String? {
    return if (category.isContinueWatchingCategory()) profile?.safeContinueWatchingArtwork ?: "episode" else null
}

fun Meta.isTraktContinueWatchingSource(): Boolean {
    return reason.equals("Trakt.tv", ignoreCase = true)
}

@Composable
internal fun TraktContinueWatchingBadge(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .size(24.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = 0.74f))
            .border(1.dp, Color.White.copy(alpha = 0.14f), CircleShape)
            .padding(4.dp),
        contentAlignment = Alignment.Center
    ) {
        Image(
            painter = painterResource(id = R.drawable.ic_trakt),
            contentDescription = null,
            modifier = Modifier.fillMaxSize()
        )
    }
}

fun posterCornerRadius(value: String): Dp = with(FluxaDimensions.CornerPresets) {
    when (value) {
        "sharp" -> sharp
        "classic", "small" -> classic
        "rounded", "large" -> rounded
        "pill" -> pill
        else -> soft
    }
}

fun posterCardWidth(value: String): Dp = with(FluxaDimensions.PosterPresets) {
    when (value) {
        "xsmall" -> xsmall
        "small" -> small
        "large" -> large
        "xlarge" -> xlarge
        else -> medium
    }
}

fun posterCardHeight(value: String): Dp = posterCardWidth(value) * FluxaDimensions.PosterPresets.heightRatio

fun horizontalCardWidth(value: String, deviceType: DeviceType): Dp {
    val base = if (deviceType == DeviceType.TV) FluxaDimensions.HorizontalCard.tvBase else FluxaDimensions.HorizontalCard.mobileBase
    val delta = with(FluxaDimensions.HorizontalCard) {
        when (value) {
            "xsmall" -> deltaXsmall
            "small" -> deltaSmall
            "large" -> deltaLarge
            "xlarge" -> deltaXlarge
            else -> 0.dp
        }
    }
    return base + delta
}

fun horizontalCardHeight(value: String, deviceType: DeviceType): Dp = horizontalCardWidth(value, deviceType) * FluxaDimensions.HorizontalCard.heightRatio

fun horizontalArtworkCandidates(meta: Meta, preferContinueWatchingArtwork: Boolean = false): List<String> {
    val existingBackdrop = meta.background
        ?.takeIf { it.isNotBlank() }
        ?.takeUnless { it == meta.poster }
        ?.takeUnless { it.contains("/poster/", ignoreCase = true) }
    val continueWatchingBackdrop = meta.continueWatchingBackground
        ?.takeIf { it.isNotBlank() }
        ?.takeUnless { it == meta.poster }
        ?.takeUnless { it.contains("/poster/", ignoreCase = true) }
    return buildList {
        if (preferContinueWatchingArtwork) {
            continueWatchingBackdrop?.let(::add)
            existingBackdrop?.let(::add)
        } else {
            existingBackdrop?.let(::add)
            continueWatchingBackdrop?.let(::add)
        }
    }.distinct()
}

fun preferredHorizontalArtwork(meta: Meta): String? {
    return horizontalArtworkCandidates(meta).firstOrNull()
}

fun mobileHeroArtworkCandidates(meta: Meta, seasonPostersOnHero: Boolean = true): List<String> {
    val seasonPoster = meta.latestSeasonPoster()
    val existingBackdrop = meta.background
        ?.takeIf { it.isNotBlank() }
        ?.takeUnless { it == meta.poster }
        ?.takeUnless { it.contains("/poster/", ignoreCase = true) }
        ?.takeUnless { !seasonPostersOnHero && it == seasonPoster }
    val posterFallback = meta.poster?.takeIf { it.isNotBlank() }
    return buildList {
        seasonPoster?.takeIf { seasonPostersOnHero }?.let(::add)
        existingBackdrop?.let(::add)
        posterFallback?.let(::add)
    }.distinct()
}

fun Meta.homeHeroBackdrop(seasonPostersOnHero: Boolean = true): String? {
    val seasonPoster = latestSeasonPoster()
    return if (seasonPostersOnHero) {
        seasonPoster ?: background
    } else {
        background?.takeUnless { it == seasonPoster }
    }
}

fun Meta.latestSeasonPoster(): String? {
    return seasonPosters
        ?.maxByOrNull { (key, _) -> key.toIntOrNull() ?: 0 }
        ?.value
        ?.takeIf { it.isNotBlank() }
}

@Composable
internal fun HomeEmptyProviderState(
    lang: String?,
    onOpenSettings: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(420.dp)
            .padding(horizontal = 28.dp),
        contentAlignment = Alignment.Center
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Icon(
                FluxaIcons.ExtensionOff,
                null,
                tint = Color.White.copy(alpha = 0.58f),
                modifier = Modifier.size(56.dp)
            )
            Spacer(Modifier.height(18.dp))
            Text(
                text = AppStrings.t(lang, "home.no_catalog_providers"),
                color = Color.White,
                fontSize = 22.sp,
                lineHeight = 26.sp,
                fontWeight = FontWeight.Black
            )
            Spacer(Modifier.height(8.dp))
            Text(
                text = AppStrings.t(lang, "home.add_catalog_addon"),
                color = Color.White.copy(alpha = 0.58f),
                fontSize = 13.sp,
                lineHeight = 18.sp,
                fontWeight = FontWeight.SemiBold
            )
            if (onOpenSettings != null) {
                Spacer(Modifier.height(20.dp))
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(Color.White)
                        .clickable { onOpenSettings() }
                        .padding(horizontal = 22.dp, vertical = 12.dp)
                ) {
                    Text(
                        text = AppStrings.t(lang, "settings.open_addon_settings"),
                        color = Color.Black,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Black
                    )
                }
            }
        }
    }
}
