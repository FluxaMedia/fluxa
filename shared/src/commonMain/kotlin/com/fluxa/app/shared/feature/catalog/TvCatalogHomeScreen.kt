package com.fluxa.app.shared.feature.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.common.AppStrings

@Composable
fun TvCatalogHomeScreen(
    state: CatalogHomeUiState,
    onAction: (CatalogAction) -> Unit,
    language: String? = null,
    modifier: Modifier = Modifier
) {
    Box(modifier = modifier.fillMaxSize()) {
        if (state.rows.isEmpty()) {
            TvCatalogHomeLoading(Modifier.fillMaxSize())
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(top = 44.dp, bottom = 64.dp),
                verticalArrangement = Arrangement.spacedBy(30.dp)
            ) {
                items(state.rows, key = { it.id }) { row ->
                    Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
                        Text(
                            text = row.title,
                            color = Color.White,
                            fontSize = 24.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.padding(horizontal = 58.dp)
                        )
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 58.dp),
                            horizontalArrangement = Arrangement.spacedBy(20.dp)
                        ) {
                            items(row.items, key = { it.id }) { item ->
                                CatalogCard(
                                    model = item.card,
                                    onClick = { onAction(CatalogAction.ItemSelected(item)) },
                                    modifier = Modifier.padding(4.dp)
                                )
                            }
                            if (row.canLoadMore) {
                                item(key = "${row.id}:load-more") {
                                    Box(
                                        modifier = Modifier
                                            .width(124.dp)
                                            .height(186.dp)
                                            .clip(androidx.compose.foundation.shape.RoundedCornerShape(14.dp))
                                            .background(FluxaColors.surfaceRaised)
                                            .clickable { onAction(CatalogAction.LoadMore(row.id)) }
                                            .padding(16.dp),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        androidx.compose.material3.Icon(
                                            imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                            contentDescription = AppStrings.t(null, "common.view_all"),
                                            tint = Color.White
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun TvCatalogHomeLoading(modifier: Modifier) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 58.dp, vertical = 44.dp),
        verticalArrangement = Arrangement.spacedBy(28.dp)
    ) {
        repeat(3) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(0.28f)
                    .height(28.dp)
                    .background(FluxaColors.surfaceRaised)
            )
            Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                repeat(6) {
                    Box(
                        modifier = Modifier
                            .width(132.dp)
                            .height(198.dp)
                            .background(FluxaColors.surfaceRaised)
                    )
                }
            }
        }
    }
}
