package com.fluxa.app.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
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
import coil3.compose.AsyncImage
import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.remote.Meta

@Composable
internal fun PlayerTerminalRecommendations(
    items: List<Meta>,
    language: String?,
    onSelected: (Meta) -> Unit,
    onDismiss: () -> Unit,
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xEE06080D))
            .padding(horizontal = 28.dp, vertical = 24.dp),
        contentAlignment = Alignment.Center,
    ) {
        Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(14.dp)) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = AppStrings.t(language, "player.recommendations"),
                    color = Color.White,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Bold,
                )
                Text(
                    text = AppStrings.t(language, "common.close"),
                    color = Color.LightGray,
                    modifier = Modifier
                        .clickable(onClick = onDismiss)
                        .focusable()
                        .padding(8.dp),
                )
            }
            LazyRow(
                contentPadding = PaddingValues(vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                items(items.take(10), key = { it.id }) { item ->
                    Column(
                        modifier = Modifier
                            .width(132.dp)
                            .clickable { onSelected(item) }
                            .focusable(),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        AsyncImage(
                            model = item.poster ?: item.background,
                            contentDescription = item.name,
                            contentScale = ContentScale.Crop,
                            modifier = Modifier
                                .width(132.dp)
                                .height(194.dp)
                                .clip(RoundedCornerShape(6.dp)),
                        )
                        Text(
                            text = item.name,
                            color = Color.White,
                            fontSize = 13.sp,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
            }
        }
    }
}
