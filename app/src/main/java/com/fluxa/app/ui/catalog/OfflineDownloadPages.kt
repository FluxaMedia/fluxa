@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
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
import coil3.request.ImageRequest
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import com.fluxa.app.data.local.OfflineDownloadItem

@Composable
internal fun MobileOfflineDownloadFoldersPage(
    lang: String,
    groups: List<OfflineDownloadGroup>,
    amoledMode: Boolean,
    onBack: () -> Unit,
    onGroupClick: (OfflineDownloadGroup) -> Unit
) {
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .background(if (amoledMode) Color.Black else Color(0xFF05070B)),
        contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 18.dp, bottom = 132.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            LibraryDownloadPageHeader(AppStrings.t(lang, "auto.downloads"), onBack)
        }
        if (groups.isEmpty()) {
            item { EmptyDownloadsText(lang) }
        } else {
            items(groups, key = { it.key }) { group ->
                OfflineDownloadFolderRow(group, lang, amoledMode) { onGroupClick(group) }
            }
        }
    }
}

@Composable
internal fun MobileOfflineDownloadGroupPage(
    lang: String,
    group: OfflineDownloadGroup,
    amoledMode: Boolean,
    onBack: () -> Unit,
    onOpenDownload: (OfflineDownloadItem) -> Unit
) {
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .background(if (amoledMode) Color.Black else Color(0xFF05070B)),
        contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 18.dp, bottom = 132.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item { LibraryDownloadPageHeader(group.title, onBack) }
        items(group.episodes, key = { it.id }) { item ->
            OfflineDownloadEpisodeRow(item, lang, amoledMode) { onOpenDownload(item) }
        }
    }
}

@Composable
private fun LibraryDownloadPageHeader(title: String, onBack: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        IconButton(onClick = onBack, modifier = Modifier.size(40.dp)) {
            Icon(FluxaIcons.ArrowBack, null, tint = Color.White, modifier = Modifier.size(24.dp))
        }
        Text(
            text = title,
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Black,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f)
        )
    }
}

@Composable
private fun OfflineDownloadFolderRow(
    group: OfflineDownloadGroup,
    lang: String,
    amoledMode: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val posterRequest = remember(group.poster) {
        ImageRequest.Builder(context).data(group.poster).memoryCacheKey("downloads-group:${group.poster}").diskCacheKey(group.poster).build()
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(92.dp)
            .background(if (amoledMode) FluxaColors.backgroundAmoled else FluxaColors.backgroundNavy.copy(alpha = 0.82f), RoundedCornerShape(8.dp))
            .border(1.dp, Color.White.copy(alpha = if (amoledMode) 0.045f else 0.04f), RoundedCornerShape(8.dp))
            .clickable { onClick() }
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .width(58.dp)
                .fillMaxHeight()
                .clip(RoundedCornerShape(6.dp))
                .background(Color.White.copy(alpha = 0.08f)),
            contentAlignment = Alignment.Center
        ) {
            if (!group.poster.isNullOrBlank()) {
                AsyncImage(model = posterRequest, contentDescription = group.title, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
            } else {
                Icon(FluxaIcons.Download, null, tint = Color.White.copy(alpha = 0.38f), modifier = Modifier.size(24.dp))
            }
        }
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(5.dp)) {
            Text(group.title, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                text = AppStrings.format(lang, "downloads.folder_summary", group.episodes.size, group.totalBytes.formatDownloadBytes()),
                color = Color.White.copy(alpha = 0.58f),
                fontSize = 12.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
        Icon(FluxaIcons.KeyboardArrowRight, null, tint = Color.White.copy(alpha = 0.48f), modifier = Modifier.size(20.dp))
    }
}

@Composable
private fun OfflineDownloadEpisodeRow(
    item: OfflineDownloadItem,
    lang: String,
    amoledMode: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current
    val artwork = item.localBackgroundPath?.toFileImageModel() ?: item.background ?: item.localPosterPath?.toFileImageModel() ?: item.poster
    val artworkRequest = remember(artwork) {
        ImageRequest.Builder(context).data(artwork).memoryCacheKey("downloads-episode:$artwork").diskCacheKey(artwork).build()
    }
    val statusText = when (item.status) {
        "downloaded" -> AppStrings.t(lang, "downloads.status_downloaded")
        "failed" -> AppStrings.t(lang, "downloads.status_failed")
        "paused" -> AppStrings.t(lang, "downloads.status_paused")
        "downloading" -> AppStrings.format(lang, "downloads.status_downloading", item.progress)
        else -> AppStrings.t(lang, "downloads.status_queued")
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(if (amoledMode) FluxaColors.backgroundAmoled else FluxaColors.backgroundNavy.copy(alpha = 0.82f), RoundedCornerShape(8.dp))
            .border(1.dp, Color.White.copy(alpha = if (amoledMode) 0.045f else 0.04f), RoundedCornerShape(8.dp))
            .clickable(enabled = item.isPlayable) { onClick() }
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .width(92.dp)
                .height(58.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(Color.White.copy(alpha = 0.08f)),
            contentAlignment = Alignment.Center
        ) {
            if (!artwork.isNullOrBlank()) {
                AsyncImage(model = artworkRequest, contentDescription = item.episodeTitle ?: item.title, modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
            } else {
                Icon(FluxaIcons.PlayCircle, null, tint = Color.White.copy(alpha = 0.38f), modifier = Modifier.size(24.dp))
            }
        }
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(5.dp)) {
            Text(
                text = item.episodeTitle?.takeIf { it.isNotBlank() } ?: item.title,
                color = Color.White,
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = listOfNotNull(statusText, item.effectiveSizeLabel()).joinToString("  "),
                color = Color.White.copy(alpha = 0.58f),
                fontSize = 11.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            item.streamTitle?.takeIf { it.isNotBlank() }?.let { source ->
                Text(AppStrings.format(lang, "downloads.video_source", source), color = Color.White.copy(alpha = 0.5f), fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(
                text = AppStrings.format(lang, "downloads.subtitle_label", item.subtitleLabel?.takeIf { it.isNotBlank() } ?: AppStrings.t(lang, "downloads.no_subtitle")),
                color = Color.White.copy(alpha = 0.45f),
                fontSize = 11.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (item.status != "downloaded") {
                LinearProgressIndicator(
                    progress = { item.progress.coerceIn(0, 100) / 100f },
                    modifier = Modifier.fillMaxWidth().height(4.dp).clip(CircleShape),
                    color = Color.White,
                    trackColor = Color.White.copy(alpha = 0.12f)
                )
            }
        }
    }
}

@Composable
private fun EmptyDownloadsText(lang: String) {
    Box(modifier = Modifier.fillMaxWidth().padding(vertical = 32.dp), contentAlignment = Alignment.Center) {
        Text(AppStrings.t(lang, "downloads.empty"), color = Color.White.copy(alpha = 0.5f), fontSize = 14.sp)
    }
}
