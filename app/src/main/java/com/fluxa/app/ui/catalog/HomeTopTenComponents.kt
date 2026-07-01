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
fun TopTenRankedPosterCard(
    rank: Int,
    posterWidth: Dp,
    posterHeight: Dp,
    modifier: Modifier = Modifier,
    posterCornerRadius: Dp = 12.dp,
    isFocused: Boolean = false,
    posterContent: @Composable BoxScope.() -> Unit
) {
    val density = LocalDensity.current

    val numberBoxWidth = when {
        rank >= 10 -> posterWidth * 1.08f
        rank == 1 -> posterWidth * 0.54f
        else -> posterWidth * 0.82f
    }

    val posterOverlap = when {
        rank >= 10 -> posterWidth * 0.33f
        rank == 1 -> posterWidth * 0.13f
        else -> posterWidth * 0.24f
    }

    val fontSize = remember(posterHeight, density) {
        with(density) { (posterHeight.toPx() * 0.90f).toSp() }
    }

    val totalWidth = numberBoxWidth + posterWidth - posterOverlap

    Box(
        modifier = modifier
            .width(totalWidth)
            .height(posterHeight),
        contentAlignment = Alignment.CenterStart
    ) {
        Box(
            modifier = Modifier
                .align(Alignment.CenterStart)
                .width(numberBoxWidth)
                .fillMaxHeight()
                .zIndex(0f),
            contentAlignment = Alignment.CenterEnd
        ) {
            TopTenRankNumber(
                rank = rank,
                fontSize = fontSize,
                modifier = Modifier.offset(
                    x = when {
                        rank == 1 -> 8.dp
                        rank >= 10 -> 0.dp
                        else -> 3.dp
                    }
                )
            )
        }

        Box(
            modifier = Modifier
                .align(Alignment.CenterEnd)
                .width(posterWidth)
                .height(posterHeight)
                .clip(RoundedCornerShape(posterCornerRadius))
                .zIndex(1f)
                .then(
                    if (isFocused) {
                        Modifier.border(
                            width = 2.dp,
                            color = Color.White.copy(alpha = 0.86f),
                            shape = RoundedCornerShape(posterCornerRadius)
                        )
                    } else {
                        Modifier
                    }
                )
        ) {
            posterContent()
        }
    }
}

@Composable
fun TopTenRankNumber(
    rank: Int,
    modifier: Modifier = Modifier,
    fontSize: TextUnit = 178.sp
) {
    val deviceType = LocalDeviceType.current
    if (deviceType == DeviceType.Mobile) {
        Text(
            text = rank.toString(),
            color = Color(0xFF111111),
            fontSize = fontSize,
            lineHeight = fontSize,
            fontFamily = FluxaDisplay,
            fontWeight = FontWeight.Black,
            maxLines = 1,
            softWrap = false,
            modifier = modifier,
            style = TextStyle(
                platformStyle = PlatformTextStyle(includeFontPadding = false)
            )
        )
        return
    }

    Box(modifier = modifier) {
        Text(
            text = rank.toString(),
            color = Color.White.copy(alpha = 0.86f),
            fontSize = fontSize,
            lineHeight = fontSize,
            fontFamily = FluxaDisplay,
            fontWeight = FontWeight.Black,
            maxLines = 1,
            softWrap = false,
            style = TextStyle(
                drawStyle = Stroke(width = 6f),
                shadow = Shadow(
                    color = Color.Black.copy(alpha = 0.65f),
                    offset = Offset(0f, 4f),
                    blurRadius = 8f
                ),
                platformStyle = PlatformTextStyle(includeFontPadding = false)
            )
        )

        Text(
            text = rank.toString(),
            color = Color(0xFF0B0B0B),
            fontSize = fontSize,
            lineHeight = fontSize,
            fontFamily = FluxaDisplay,
            fontWeight = FontWeight.Black,
            maxLines = 1,
            softWrap = false,
            style = TextStyle(
                platformStyle = PlatformTextStyle(includeFontPadding = false)
            )
        )
    }
}
