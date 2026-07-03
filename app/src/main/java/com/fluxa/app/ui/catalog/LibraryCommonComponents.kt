@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.fluxa.app.data.remote.Meta

@Composable
internal fun LibraryPosterGridCard(item: Meta, modifier: Modifier = Modifier, onClick: () -> Unit) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(8.dp))
            .clickable { onClick() }
    ) {
        AsyncImage(
            model = item.poster,
            contentDescription = item.name,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(2f / 3f)
                .clip(RoundedCornerShape(8.dp))
                .background(Color.White.copy(alpha = 0.08f)),
            contentScale = ContentScale.Crop
        )
        Spacer(Modifier.height(7.dp))
        Text(
            text = item.name,
            color = Color.White,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            lineHeight = 13.sp
        )
    }
}

@Composable
internal fun LibraryFilterChip(label: String, selected: Boolean, accent: Color, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .height(34.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(if (selected) accent else Color(0xFF121722))
            .clickable { onClick() }
            .padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (selected) Color.White else Color.White.copy(alpha = 0.72f),
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
internal fun LibraryStatRow(icon: ImageVector, title: String, count: Int, selected: Boolean, amoledMode: Boolean = false, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(52.dp)
            .background(if (selected) Color.White.copy(alpha = if (amoledMode) 0.06f else 0.08f) else Color.Transparent)
            .clickable { onClick() }
            .padding(horizontal = 14.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(icon, null, tint = Color.White.copy(alpha = 0.86f), modifier = Modifier.size(20.dp))
        Spacer(Modifier.width(14.dp))
        Text(
            text = title,
            color = Color.White.copy(alpha = 0.88f),
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.weight(1f)
        )
        Text(
            text = count.toString(),
            color = Color.White.copy(alpha = 0.58f),
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold
        )
        Icon(FluxaIcons.KeyboardArrowRight, null, tint = Color.White.copy(alpha = 0.48f), modifier = Modifier.size(20.dp))
    }
}

@Composable
internal fun LibraryItemRow(item: Meta, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(72.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(FluxaColors.backgroundNavy.copy(alpha = 0.78f))
            .clickable { onClick() }
            .padding(6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        AsyncImage(
            model = item.poster,
            contentDescription = item.name,
            modifier = Modifier
                .width(52.dp)
                .fillMaxHeight()
                .clip(RoundedCornerShape(6.dp))
                .background(Color.White.copy(alpha = 0.08f)),
            contentScale = ContentScale.Crop
        )
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(item.name, color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                listOfNotNull(item.releaseInfo, item.lastEpisodeName).joinToString("  "),
                color = Color.White.copy(alpha = 0.48f),
                fontSize = 11.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        Icon(FluxaIcons.KeyboardArrowRight, null, tint = Color.White.copy(alpha = 0.48f), modifier = Modifier.size(20.dp))
    }
}

@Composable
internal fun LibraryDivider(amoledMode: Boolean = false) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 48.dp)
            .height(1.dp)
            .background(Color.White.copy(alpha = if (amoledMode) 0.035f else 0.05f))
    )
}

@Composable
internal fun LibraryCollectionRow(
    collection: LibraryCollectionUi,
    amoledMode: Boolean = false,
    editMode: Boolean = false,
    onEdit: (LibraryCollectionUi) -> Unit = {},
    onDelete: (LibraryCollectionUi) -> Unit = {},
    onClick: () -> Unit
) {
    val poster = collection.items.firstOrNull()?.poster
    val isEditable = collection.userCollectionId != null && !collection.locked
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(62.dp)
            .background(if (amoledMode) FluxaColors.backgroundAmoled else FluxaColors.backgroundNavy.copy(alpha = 0.78f), RoundedCornerShape(8.dp))
            .border(1.dp, Color.White.copy(alpha = if (amoledMode) 0.045f else 0.035f), RoundedCornerShape(8.dp))
            .clickable(enabled = collection.items.isNotEmpty() && !editMode) { onClick() }
            .padding(6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .width(58.dp)
                .fillMaxHeight()
                .clip(RoundedCornerShape(6.dp))
                .background(Color.White.copy(alpha = 0.08f))
        ) {
            if (!poster.isNullOrBlank()) {
                AsyncImage(
                    model = poster,
                    contentDescription = collection.title,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop
                )
            }
        }
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = collection.title,
                color = Color.White,
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Spacer(Modifier.height(3.dp))
            Text(
                text = collection.subtitle,
                color = Color.White.copy(alpha = 0.48f),
                fontSize = 11.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        if (editMode && isEditable) {
            IconButton(onClick = { onEdit(collection) }, modifier = Modifier.size(36.dp)) {
                Icon(FluxaIcons.Edit, null, tint = Color.White.copy(alpha = 0.76f), modifier = Modifier.size(18.dp))
            }
            IconButton(onClick = { onDelete(collection) }, modifier = Modifier.size(36.dp)) {
                Icon(FluxaIcons.Delete, null, tint = FluxaColors.errorRed, modifier = Modifier.size(18.dp))
            }
        } else if (collection.locked) {
            Icon(FluxaIcons.Lock, null, tint = Color.White.copy(alpha = 0.58f), modifier = Modifier.size(18.dp))
        } else if (collection.items.isNotEmpty()) {
            Icon(FluxaIcons.KeyboardArrowRight, null, tint = Color.White.copy(alpha = 0.48f), modifier = Modifier.size(20.dp))
        }
    }
}

@Composable
fun EmptyLibraryState(lang: String) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.alpha(0.6f)) {
            Icon(FluxaIcons.BookmarkBorder, null, modifier = Modifier.size(100.dp), tint = Color.Gray)
            Spacer(modifier = Modifier.height(24.dp))
            Text(text = AppStrings.t(lang, "auto.your_library_is_empty"), style = androidx.compose.material3.MaterialTheme.typography.headlineMedium, color = Color.White, fontWeight = FontWeight.Bold)
        }
    }
}
