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
import androidx.compose.foundation.shape.CircleShape
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
internal fun CalendarMonthGrid(
    weekdays: List<String>,
    days: List<MonthCell?>,
    itemsByDate: Map<String, List<CalendarUpcomingItem>>,
    selectedDateIso: String?,
    todayIso: String,
    accent: Color,
    onDayClick: (MonthCell) -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(5.dp)
        ) {
            weekdays.forEach { day ->
                Text(
                    text = day,
                    color = Color.White.copy(alpha = 0.46f),
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.weight(1f)
                )
            }
        }
        days.chunked(7).forEach { rowDays ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(5.dp)
            ) {
                rowDays.forEach { day ->
                    val dayEvents = day?.let { itemsByDate[it.dateIso] }.orEmpty()
                    CalendarDayCell(
                        day = day,
                        isSelected = day?.dateIso == selectedDateIso,
                        isToday = day?.dateIso == todayIso,
                        posterUrl = day?.takeIf { it.isCurrentMonth }?.let { dayEvents.firstOrNull()?.artworkUrl() },
                        accent = accent,
                        onClick = { day?.takeIf { it.isCurrentMonth }?.let(onDayClick) },
                        modifier = Modifier.weight(1f)
                    )
                }
            }
        }
    }
}

@Composable
private fun CalendarDayCell(
    day: MonthCell?,
    isSelected: Boolean,
    isToday: Boolean,
    posterUrl: String?,
    accent: Color,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .height(60.dp)
            .clickable(enabled = day?.isCurrentMonth == true) { onClick() }
            .padding(6.dp)
    ) {
        Column(
            modifier = Modifier.align(Alignment.Center),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .clip(CircleShape)
                    .background(
                        when {
                            isSelected -> accent.copy(alpha = 0.20f)
                            isToday -> Color.White.copy(alpha = 0.10f)
                            else -> Color.Transparent
                        }
                    )
                    .border(
                        width = if (isSelected) 1.4.dp else 1.dp,
                        color = when {
                            isSelected -> accent
                            isToday -> Color.White.copy(alpha = 0.22f)
                            else -> Color.Transparent
                        },
                        shape = CircleShape
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = day?.day?.toString().orEmpty(),
                    color = when {
                        day == null -> Color.Transparent
                        day.isCurrentMonth -> Color.White.copy(alpha = 0.9f)
                        else -> Color.White.copy(alpha = 0.28f)
                    },
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium,
                    textAlign = TextAlign.Center
                )
            }
            Spacer(Modifier.height(5.dp))
            if (posterUrl != null) {
                AsyncImage(
                    model = posterUrl,
                    contentDescription = null,
                    modifier = Modifier
                        .size(width = 14.dp, height = 9.dp)
                        .clip(RoundedCornerShape(2.dp))
                        .background(Color.White.copy(alpha = 0.1f)),
                    contentScale = ContentScale.Crop
                )
            } else {
                Box(modifier = Modifier.size(6.dp))
            }
        }
    }
}
