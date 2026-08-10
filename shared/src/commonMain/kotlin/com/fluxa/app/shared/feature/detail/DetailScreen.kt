package com.fluxa.app.shared.feature.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.LocalDeviceType
import com.fluxa.app.ui.catalog.LocalWindowWidthClass
import com.fluxa.app.ui.catalog.WindowWidthClass
import com.fluxa.app.ui.catalog.FluxaColors

internal enum class DetailTab { Episodes, MoreLikeThis }

val LocalDetailRatingLogo = staticCompositionLocalOf<@Composable (String, String, Modifier) -> Unit> {
    { source, _, modifier -> Text(source, modifier = modifier, color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.SemiBold) }
}

internal val ratingDisplayOrder = listOf("imdb", "tmdb", "trakt", "tomatoes", "popcorn", "metacritic", "metacriticuser", "letterboxd", "myanimelist")

internal fun DetailRatingUiModel.normalizedSource(): String = source.lowercase().replace(" ", "")

internal fun DetailRatingUiModel.displayValue(): String {
    val number = value.toFloatOrNull() ?: return value
    return when (normalizedSource()) {
        "tmdb", "trakt", "tomatoes", "popcorn" -> "${number.toInt()}%"
        "metacritic" -> number.toInt().toString()
        else -> ((number * 10f).toInt() / 10f).toString()
    }
}

internal fun DetailRatingUiModel.scoreColor(): Color = when (normalizedSource()) {
    "metacritic" -> when {
        (value.toFloatOrNull() ?: 0f) >= 61f -> Color(0xFF66CC33)
        (value.toFloatOrNull() ?: 0f) >= 40f -> Color(0xFFFFCC33)
        else -> Color(0xFFFF3333)
    }
    "metacriticuser" -> when {
        (value.toFloatOrNull() ?: 0f) >= 7.5f -> Color(0xFF66CC33)
        (value.toFloatOrNull() ?: 0f) >= 5f -> Color(0xFFFFCC33)
        else -> Color(0xFFFF3333)
    }
    else -> Color.White
}

internal fun DetailUiModel.visibleRatings(): List<DetailRatingUiModel> = ratings.ifEmpty {
    ratingLabel.takeIf { it.isNotBlank() }?.let { listOf(DetailRatingUiModel("IMDb", it)) }.orEmpty()
}.map { rating -> rating.copy(source = rating.normalizedSource()) }
    .filter { it.source in ratingDisplayOrder }
    .sortedBy { ratingDisplayOrder.indexOf(it.source) }

@Composable
fun DetailScreen(
    state: DetailUiState,
    language: String?,
    onAction: (DetailAction) -> Unit,
    onBack: () -> Unit = {},
    onShareRequested: () -> Unit = {},
    presentation: DetailPresentationOptions = DetailPresentationOptions(),
    modifier: Modifier = Modifier
) {
    val content = state.content
    val isMobile = LocalDeviceType.current == DeviceType.Mobile
    var showStickyContext by remember(content?.id) { mutableStateOf(false) }
    Box(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        when {
            content != null -> DetailContent(
                content = content,
                language = language,
                onAction = onAction,
                presentation = presentation,
                onStickyContextVisibilityChanged = { showStickyContext = it }
            )
            state.isLoading -> DetailLoading()
            else -> DetailEmpty(text = AppStrings.t(language, state.errorKey ?: "auto.no_results_found"))
        }
        TopBar(
            content = content,
            language = language,
            showStickyContext = isMobile && showStickyContext,
            preferClearlogo = presentation.preferClearlogo,
            screenStyle = presentation.screenStyle,
            onAction = onAction,
            onBack = onBack,
            onShareRequested = onShareRequested
        )
    }
}

@Composable
internal fun TopBar(
    content: DetailUiModel?,
    language: String?,
    showStickyContext: Boolean,
    preferClearlogo: Boolean,
    screenStyle: DetailScreenStyle,
    onAction: (DetailAction) -> Unit,
    onBack: () -> Unit,
    onShareRequested: () -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile && screenStyle == DetailScreenStyle.Cinematic && !showStickyContext) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top))
                .padding(start = 16.dp, end = 16.dp, top = 10.dp, bottom = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(Color.Black.copy(alpha = 0.38f))
                    .clickable(onClick = onBack),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                    contentDescription = AppStrings.t(language, "common.back"),
                    tint = Color.White,
                    modifier = Modifier.size(23.dp)
                )
            }
        }
        return
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .then(
                if (showStickyContext) Modifier.background(Color.Black.copy(alpha = 0.94f)) else Modifier.background(
                    Brush.verticalGradient(
                    colorStops = arrayOf(0f to Color.Black.copy(alpha = 0.45f), 1f to Color.Transparent)
                    )
                )
            )
            .then(if (showStickyContext) Modifier.windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top)) else Modifier)
            .padding(
                top = if (showStickyContext) 0.dp else 44.dp,
                bottom = if (showStickyContext) 8.dp else 24.dp,
                start = if (showStickyContext) 14.dp else 12.dp,
                end = if (showStickyContext) 14.dp else 12.dp
            ),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
            contentDescription = AppStrings.t(language, "common.back"),
            tint = Color.White,
            modifier = Modifier.size(22.dp).clickable(onClick = onBack)
        )
        if (showStickyContext && content != null) {
            StickyContentIdentity(
                content = content,
                preferClearlogo = preferClearlogo,
                modifier = Modifier.width(130.dp).padding(start = 10.dp, end = 6.dp)
            )
        }
        Box(modifier = Modifier.weight(1f))
        if (content != null && content.supportsWatchlist) {
            Icon(
                imageVector = if (content.isInWatchlist) Icons.Filled.Check else Icons.Filled.Add,
                contentDescription = AppStrings.t(language, if (content.isInWatchlist) "auto.in_list" else "auto.my_list"),
                tint = Color.White,
                modifier = Modifier.size(22.dp).clickable { onAction(DetailAction.ToggleWatchlist) }
            )
        }
        Icon(
            imageVector = Icons.Filled.Share,
            contentDescription = AppStrings.t(language, "common.share"),
            tint = Color.White,
            modifier = Modifier.padding(start = 16.dp).size(22.dp).clickable(onClick = onShareRequested)
        )
    }
}

@Composable
internal fun StickyContentIdentity(
    content: DetailUiModel,
    preferClearlogo: Boolean = true,
    modifier: Modifier = Modifier
) {
    if (preferClearlogo && content.logoUrl != null) {
        FluxaRemoteImage(
            imageUrl = content.logoUrl,
            cacheKey = "detail-sticky-logo:${content.id}",
            contentDescription = content.title,
            modifier = modifier.height(28.dp),
            contentScale = ContentScale.Fit,
            trimTransparentPadding = true
        )
    } else {
        Text(
            text = content.title,
            color = Color.White,
            fontSize = 16.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = modifier
        )
    }
}

@Composable
internal fun DetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    presentation: DetailPresentationOptions,
    onStickyContextVisibilityChanged: (Boolean) -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        when (presentation.screenStyle) {
            DetailScreenStyle.Cinematic -> CinematicMobileDetailContent(
                content = content,
                language = language,
                onAction = onAction,
                presentation = presentation,
                onStickyContextVisibilityChanged = onStickyContextVisibilityChanged
            )
            DetailScreenStyle.Compact -> CompactMobileDetailContent(
                content = content,
                language = language,
                onAction = onAction,
                presentation = presentation,
                onStickyContextVisibilityChanged = onStickyContextVisibilityChanged
            )
            DetailScreenStyle.Classic -> ClassicDetailContent(
                content = content,
                language = language,
                onAction = onAction,
                presentation = presentation,
                onStickyContextVisibilityChanged = onStickyContextVisibilityChanged
            )
        }
        return
    }

    ClassicDetailContent(
        content = content,
        language = language,
        onAction = onAction,
        presentation = presentation.copy(
            screenStyle = DetailScreenStyle.Classic,
            episodeCardsLayout = if (LocalDeviceType.current == DeviceType.Desktop) "desktop_grid" else "list"
        ),
        onStickyContextVisibilityChanged = onStickyContextVisibilityChanged
    )
}

@Composable
internal fun ClassicDetailContent(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    presentation: DetailPresentationOptions,
    onStickyContextVisibilityChanged: (Boolean) -> Unit
) {
    val isSeries = content.type == "series" && content.availableSeasons.isNotEmpty()
    val hasSecondary = isSeries || (presentation.showRecommendations && content.relatedItems.isNotEmpty())
    val twoPane = LocalWindowWidthClass.current == WindowWidthClass.Expanded &&
        LocalDeviceType.current != DeviceType.TV &&
        LocalDeviceType.current != DeviceType.Desktop
    if (twoPane && hasSecondary) {
        val listState = rememberLazyListState()
        ObserveStickyContextVisibility(listState, onStickyContextVisibilityChanged)
        Row(modifier = Modifier.fillMaxSize()) {
            LazyColumn(state = listState, modifier = Modifier.width(400.dp).fillMaxHeight()) {
                item(key = "hero") { Hero(content = content, language = language, preferClearlogo = presentation.preferClearlogo) }
                item(key = "body") { DetailBody(content = content, language = language, onAction = onAction, presentation = presentation) }
                item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
            }
            DetailSecondaryPane(
                content = content,
                language = language,
                onAction = onAction,
                isSeries = isSeries,
                presentation = presentation,
                modifier = Modifier.weight(1f).fillMaxHeight()
            )
        }
    } else {
        val isExpanded = LocalWindowWidthClass.current == WindowWidthClass.Expanded &&
            LocalDeviceType.current != DeviceType.TV
        var activeTab by remember(content.id) { mutableStateOf(DetailTab.Episodes) }
        val listState = rememberLazyListState()
        ObserveStickyContextVisibility(listState, onStickyContextVisibilityChanged)
        if (isExpanded) {
            androidx.compose.foundation.layout.BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
                val density = LocalDensity.current
                val widthPx = with(density) { maxWidth.toPx() }
                val heightPx = with(density) { maxHeight.toPx() }
                FluxaRemoteImage(
                    imageUrl = content.backgroundUrl ?: content.posterUrl,
                    cacheKey = "detail-hero:${content.id}",
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    alignment = Alignment.TopCenter
                )
                Box(
                    modifier = Modifier.fillMaxSize().background(
                        Brush.linearGradient(
                            colorStops = arrayOf(
                                0f to FluxaColors.background,
                                0.35f to FluxaColors.background.copy(alpha = 0.92f),
                                0.65f to FluxaColors.background.copy(alpha = 0.45f),
                                1f to Color.Transparent
                            ),
                            start = androidx.compose.ui.geometry.Offset(0f, 0f),
                            end = androidx.compose.ui.geometry.Offset(widthPx * 0.62f, heightPx * 0.78f)
                        )
                    )
                )
                LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
                    item(key = "spacer-top") { Box(modifier = Modifier.height(140.dp)) }
                    item(key = "identity") {
                        ExpandedDetailIdentityBlock(content = content, language = language, onAction = onAction, preferClearlogo = presentation.preferClearlogo)
                    }
                    if (presentation.showCast && content.cast.isNotEmpty()) {
                        item(key = "cast") {
                            CastSection(members = content.cast, language = language, modifier = Modifier.padding(top = 28.dp, start = 40.dp))
                        }
                    }
                    detailSecondaryItems(content, language, onAction, isSeries, activeTab, presentation) { activeTab = it }
                    item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
                }
            }
        } else {
            LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
                item(key = "hero") { Hero(content = content, language = language, preferClearlogo = presentation.preferClearlogo) }
                item(key = "body") { DetailBody(content = content, language = language, onAction = onAction, presentation = presentation) }
                detailSecondaryItems(content, language, onAction, isSeries, activeTab, presentation) { activeTab = it }
                item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
            }
        }
    }
}

@Composable
internal fun ObserveStickyContextVisibility(
    listState: LazyListState,
    onStickyContextVisibilityChanged: (Boolean) -> Unit,
    threshold: Dp = 390.dp
) {
    val logoExitOffset = with(LocalDensity.current) { threshold.roundToPx() }
    val showStickyContext by remember(listState, logoExitOffset) {
        derivedStateOf {
            listState.firstVisibleItemIndex > 0 ||
                (listState.firstVisibleItemIndex == 0 && listState.firstVisibleItemScrollOffset >= logoExitOffset)
        }
    }
    LaunchedEffect(showStickyContext) {
        onStickyContextVisibilityChanged(showStickyContext)
    }
}

@Composable
internal fun DetailSecondaryPane(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    isSeries: Boolean,
    presentation: DetailPresentationOptions,
    modifier: Modifier = Modifier
) {
    var activeTab by remember(content.id) { mutableStateOf(DetailTab.Episodes) }
    val effectivePresentation = if (LocalDeviceType.current == DeviceType.Desktop) {
        presentation.copy(episodeCardsLayout = "desktop_grid")
    } else {
        presentation
    }
    LazyColumn(modifier = modifier) {
        detailSecondaryItems(content, language, onAction, isSeries, activeTab, effectivePresentation) { activeTab = it }
        item(key = "bottom-spacer") { Box(modifier = Modifier.height(32.dp)) }
    }
}

internal fun androidx.compose.foundation.lazy.LazyListScope.detailSecondaryItems(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    isSeries: Boolean,
    activeTab: DetailTab,
    presentation: DetailPresentationOptions,
    onTabSelected: (DetailTab) -> Unit
) {
    val hasRelated = presentation.showRecommendations && content.relatedItems.isNotEmpty()
    if (isSeries || hasRelated) {
        item(key = "tabs") {
            TabRow(
                isSeries = isSeries,
                hasRelated = hasRelated,
                activeTab = activeTab,
                language = language,
                onTabSelected = onTabSelected,
                modifier = Modifier.padding(top = 24.dp)
            )
        }
    }
    when {
        isSeries && activeTab == DetailTab.Episodes -> {
            item(key = "season-selector") {
                SeasonSelector(content = content, language = language, onAction = onAction, mode = presentation.seasonSelectorMode)
            }
            detailEpisodeItems(
                content = content,
                language = language,
                presentation = presentation,
                onAction = onAction
            )
        }
        hasRelated -> {
            item(key = "related") {
                RelatedGrid(items = content.relatedItems, onAction = onAction)
            }
        }
        else -> Unit
    }
}

@Composable
internal fun DetailBody(
    content: DetailUiModel,
    language: String?,
    onAction: (DetailAction) -> Unit,
    presentation: DetailPresentationOptions
) {
    val isExpanded = LocalWindowWidthClass.current == WindowWidthClass.Expanded
    Column(
        modifier = Modifier
            .then(if (isExpanded) Modifier.widthIn(max = 640.dp) else Modifier)
            .padding(horizontal = 20.dp)
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.padding(top = 8.dp)) {
            ResumeButton(content = content, language = language, onAction = onAction)
            if (content.resumeProgress > 0L && content.resumeVideoId != null) {
                RestartButton(language = language, onClick = { onAction(DetailAction.Play(fromStart = true)) })
            }
            DownloadButton(content = content, language = language, onAction = onAction)
        }
        if (content.description.isNotBlank()) {
            Text(
                text = content.description,
                color = Color.White,
                fontSize = 14.sp,
                lineHeight = 19.sp,
                modifier = Modifier.padding(top = 16.dp)
            )
        }
        if (presentation.showCast && content.cast.isNotEmpty()) {
            CastSection(members = content.cast, language = language, modifier = Modifier.padding(top = 20.dp))
        }
        DiscussionSection(AppStrings.t(language, "detail.trakt_comments"), content.traktComments, language)
        DiscussionSection(AppStrings.t(language, "detail.mdblist_discussion"), content.mdblistDiscussion, language)
    }
}
