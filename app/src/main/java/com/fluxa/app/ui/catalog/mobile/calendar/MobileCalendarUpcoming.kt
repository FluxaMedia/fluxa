@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import java.text.DateFormatSymbols
import java.text.ParseException
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.Calendar as JavaCalendar

@Composable
internal fun CalendarUpcomingSection(
    dateIso: String,
    locale: Locale,
    lang: String,
    events: List<CalendarUpcomingItem>,
    onMovieClick: (Meta) -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(
            text = AppStrings.format(lang, "calendar.upcoming_on", formatCalendarListDate(dateIso, locale)),
            color = Color.White,
            fontSize = 16.sp,
            fontWeight = FontWeight.Black,
            modifier = Modifier.padding(top = 2.dp)
        )
        events.forEach { event ->
            CalendarUpcomingCard(
                event = event,
                lang = lang,
                onClick = { onMovieClick(event.meta) }
            )
        }
    }
}

@Composable
private fun CalendarUpcomingCard(
    event: CalendarUpcomingItem,
    lang: String,
    onClick: () -> Unit
) {
    val seasonEpisode = remember(lang, event.subtitle) { event.calendarSeasonEpisodeText(lang) }
    val episodeTitle = remember(event.subtitle) { event.calendarSecondaryTitle() }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(Color.White.copy(alpha = 0.065f))
            .border(1.dp, Color.White.copy(alpha = 0.055f), RoundedCornerShape(8.dp))
            .clickable(onClick = onClick)
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        AsyncImage(
            model = event.artworkUrl(),
            contentDescription = null,
            modifier = Modifier
                .clip(RoundedCornerShape(8.dp))
                .width(96.dp)
                .height(68.dp)
                .background(Color.White.copy(alpha = 0.08f)),
            contentScale = ContentScale.Crop
        )
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = event.title,
                color = Color.White,
                fontSize = 15.sp,
                fontWeight = FontWeight.Black,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (seasonEpisode.isNotBlank()) {
                Text(
                    text = seasonEpisode,
                    color = Color.White.copy(alpha = 0.66f),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
            if (episodeTitle.isNotBlank()) {
                Text(
                    text = episodeTitle,
                    color = Color.White.copy(alpha = 0.84f),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = 3.dp)
                )
            }
        }
        Icon(FluxaIcons.ChevronRight, null, tint = Color.White.copy(alpha = 0.44f), modifier = Modifier.size(22.dp))
    }
}

