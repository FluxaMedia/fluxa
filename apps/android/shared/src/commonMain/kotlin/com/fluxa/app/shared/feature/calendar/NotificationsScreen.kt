package com.fluxa.app.shared.feature.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.image.FluxaRemoteImage
import com.fluxa.app.ui.catalog.FluxaColors
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlinx.datetime.LocalDate
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

private enum class NotificationBucket { New, Soon, Later }

@OptIn(ExperimentalTime::class)
@Composable
fun NotificationsScreen(
    state: CalendarUiState?,
    language: String?,
    onBack: () -> Unit,
    onItemSelected: (CalendarReleaseUiModel) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier.fillMaxSize().background(FluxaColors.background)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 44.dp, bottom = 16.dp, start = 12.dp, end = 20.dp)
        ) {
            Icon(
                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                contentDescription = AppStrings.t(language, "common.close"),
                tint = Color.White,
                modifier = Modifier.size(28.dp).clickable(onClick = onBack)
            )
            Text(
                text = AppStrings.t(language, "auto.notifications"),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 20.sp,
                modifier = Modifier.padding(start = 16.dp)
            )
        }
        when {
            state == null || state.isLoading -> Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Color.White)
            }
            state.items.isEmpty() -> Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(text = AppStrings.t(language, "auto.no_results_found"), color = Color.White.copy(alpha = 0.7f))
            }
            else -> {
                val today = Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).date
                val grouped = state.items.groupBy { it.bucket(today) }
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    listOfNotNull(
                        NotificationBucket.Soon to "auto.notifications_coming_soon",
                        NotificationBucket.New to "auto.notifications_new",
                        NotificationBucket.Later to "auto.notifications_later"
                    ).forEach { (bucket, titleKey) ->
                        val bucketItems = grouped[bucket].orEmpty()
                        if (bucketItems.isNotEmpty()) {
                            item(key = "header-$bucket") {
                                Text(
                                    text = AppStrings.t(language, titleKey),
                                    color = Color.White,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 15.sp,
                                    modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 20.dp, bottom = 8.dp)
                                )
                            }
                            items(bucketItems, key = { notificationItemKey(it.id, it.dateIso, it.subtitle) }) { release ->
                                NotificationRow(release = release, language = language, onClick = { onItemSelected(release) })
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun NotificationRow(release: CalendarReleaseUiModel, language: String?, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 20.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .width(72.dp)
                .aspectRatio(2f / 3f)
                .clip(RoundedCornerShape(4.dp))
                .background(FluxaColors.surfaceCard)
        ) {
            FluxaRemoteImage(
                imageUrl = release.artworkUrl,
                cacheKey = "notification:${release.id}",
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
        }
        Column(modifier = Modifier.padding(start = 12.dp).weight(1f)) {
            Text(
                text = release.title,
                color = Color.White,
                fontWeight = FontWeight.Medium,
                fontSize = 14.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (release.subtitle.isNotBlank()) {
                Text(
                    text = release.subtitle,
                    color = Color.White.copy(alpha = 0.6f),
                    fontSize = 12.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 2.dp)
                )
            }
            Text(
                text = notificationDisplayDate(release.dateIso, language),
                color = Color.White.copy(alpha = 0.45f),
                fontSize = 11.sp,
                modifier = Modifier.padding(top = 4.dp)
            )
        }
    }
}

internal fun notificationItemKey(id: String, dateIso: String, subtitle: String): String =
    "$id:$dateIso:$subtitle"

internal fun notificationDisplayDate(dateIso: String, language: String?): String {
    val date = runCatching { LocalDate.parse(dateIso) }.getOrNull() ?: return dateIso
    val monthKeys = listOf(
        "calendar.month_january",
        "calendar.month_february",
        "calendar.month_march",
        "calendar.month_april",
        "calendar.month_may",
        "calendar.month_june",
        "calendar.month_july",
        "calendar.month_august",
        "calendar.month_september",
        "calendar.month_october",
        "calendar.month_november",
        "calendar.month_december"
    )
    return "${date.day} ${AppStrings.t(language, monthKeys[date.month.ordinal])} ${date.year}"
}

private fun CalendarReleaseUiModel.bucket(today: LocalDate): NotificationBucket {
    val date = runCatching { LocalDate.parse(dateIso) }.getOrNull() ?: return NotificationBucket.Later
    val daysDiff = date.toEpochDays() - today.toEpochDays()
    return when {
        daysDiff < 0 -> NotificationBucket.New
        daysDiff <= 7 -> NotificationBucket.Soon
        else -> NotificationBucket.Later
    }
}
