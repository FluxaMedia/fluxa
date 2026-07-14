package com.fluxa.app.shared.platform

import androidx.compose.ui.unit.dp
import com.fluxa.app.shared.feature.calendar.CalendarDataSource
import com.fluxa.app.shared.feature.calendar.CalendarReleaseUiModel
import com.fluxa.app.shared.feature.calendar.CalendarUiState
import com.fluxa.app.shared.feature.catalog.CatalogItemUiModel
import com.fluxa.app.shared.feature.catalog.CatalogSourceUiModel
import com.fluxa.app.ui.catalog.CatalogCardUiModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import platform.Foundation.NSCalendar
import platform.Foundation.NSCalendarUnitMonth
import platform.Foundation.NSCalendarUnitYear
import platform.Foundation.NSDate

data class AppleCalendarReleaseSnapshot(
    val id: String,
    val type: String,
    val dateIso: String,
    val title: String,
    val subtitle: String = "",
    val posterUrl: String? = null,
    val addonTransportUrl: String? = null,
    val catalogType: String? = null
)

data class AppleCalendarSnapshot(
    val year: Int,
    val month: Int,
    val items: List<AppleCalendarReleaseSnapshot> = emptyList(),
    val isLoading: Boolean = false
)

class AppleCalendarDataSource : CalendarDataSource {
    private val state = MutableStateFlow(CalendarUiState())
    private var onMonthRequested: (Int, Int) -> Unit = { _, _ -> }

    override fun observeCalendar(): Flow<CalendarUiState> = state.asStateFlow()

    override suspend fun refresh() {
        val current = NSCalendar.currentCalendar.components(
            NSCalendarUnitYear or NSCalendarUnitMonth,
            fromDate = NSDate()
        )
        loadMonth(current.year.toInt(), current.month.toInt())
    }

    override suspend fun loadMonth(year: Int, month: Int) {
        state.value = state.value.copy(year = year, month = month, isLoading = true)
        onMonthRequested(year, month)
    }

    fun setOnMonthRequested(handler: (Int, Int) -> Unit) {
        onMonthRequested = handler
    }

    fun update(snapshot: AppleCalendarSnapshot) {
        state.value = CalendarUiState(snapshot.year, snapshot.month, snapshot.items.map { it.toCalendarRelease() }, snapshot.isLoading)
    }
}

private fun AppleCalendarReleaseSnapshot.toCalendarRelease(): CalendarReleaseUiModel {
    val artworkUrl = posterUrl
    val item = CatalogItemUiModel(
        id = id,
        type = type,
        source = CatalogSourceUiModel(
            addonTransportUrl = addonTransportUrl,
            catalogType = catalogType
        ),
        card = CatalogCardUiModel(
            title = title,
            subtitle = subtitle,
            showTitleBar = true,
            artworkUrl = artworkUrl,
            artworkMemoryCacheKey = artworkUrl?.let { "apple-calendar:$it" },
            artworkDiskCacheKey = artworkUrl?.let { "apple-calendar:$it" },
            requestWidthPx = 264,
            requestHeightPx = 396,
            logoUrl = null,
            logoMemoryCacheKey = null,
            showLogo = false,
            allowCoverFallback = true,
            coverFallbackText = title,
            coverFallbackIsEmoji = false,
            width = 132.dp,
            imageHeight = 198.dp,
            outerWidth = 132.dp,
            cardBackgroundIsSurfaceCard = true,
            progress = 0f,
            showProgressBar = false,
            showUpNextBadge = false,
            upNextLabel = "",
            topTenRank = null,
            rankNumberBoxWidth = 0.dp,
            rankOffsetX = 0.dp,
            rankOffsetY = 0.dp,
            rankFontSizeRatio = 1f,
            loadArtwork = true
        )
    )
    return CalendarReleaseUiModel(
        id = id,
        dateIso = dateIso,
        title = title,
        subtitle = subtitle,
        artworkUrl = artworkUrl,
        item = item
    )
}
