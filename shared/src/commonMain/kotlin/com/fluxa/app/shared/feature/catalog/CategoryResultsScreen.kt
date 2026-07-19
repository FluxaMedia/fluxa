package com.fluxa.app.shared.feature.catalog

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.CatalogCard
import com.fluxa.app.ui.catalog.DeviceType

@Composable
fun CategoryResultsScreen(
    title: String,
    items: List<CatalogItemUiModel>,
    language: String?,
    onBack: () -> Unit,
    onItemSelected: (CatalogItemUiModel) -> Unit,
    deviceType: DeviceType = DeviceType.Mobile,
    modifier: Modifier = Modifier
) {
    val isTv = deviceType == DeviceType.TV
    Column(modifier = modifier.fillMaxSize()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = if (isTv) 48.dp else 16.dp, vertical = if (isTv) 28.dp else 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Button(onClick = onBack) {
                Text(AppStrings.t(language, "common.back"))
            }
            Text(
                text = title,
                color = Color.White,
                fontSize = if (isTv) 28.sp else 24.sp,
                fontWeight = FontWeight.Bold
            )
        }
        LazyVerticalGrid(
            columns = GridCells.Adaptive(if (isTv) 170.dp else 150.dp),
            modifier = Modifier.fillMaxSize(),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = if (isTv) 48.dp else 16.dp, vertical = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(if (isTv) 20.dp else 12.dp),
            verticalArrangement = Arrangement.spacedBy(if (isTv) 24.dp else 16.dp)
        ) {
            items(items, key = { "${it.type}:${it.id}" }) { item ->
                CatalogCard(model = item.card, onClick = { onItemSelected(item) })
            }
        }
    }
}
