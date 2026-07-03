@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
internal fun ContinueWatchingActionsSheet(
    meta: Meta,
    lang: String,
    onDismiss: () -> Unit,
    onDetails: () -> Unit,
    onForget: () -> Unit
) {
    val progress = ((meta.timeOffset ?: 0L).toFloat() / (meta.duration ?: 1L).toFloat()).coerceIn(0f, 1f)
    val episodeInfo = continueWatchingEpisodeLabel(meta)
    val typeLabel = if (meta.type == "movie") AppStrings.t(lang, "auto.movie") else AppStrings.t(lang, "auto.series")
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = Color(0xFF10131A),
        contentColor = Color.White
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 8.dp)
                .padding(bottom = 28.dp)
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                AsyncImage(
                    model = meta.continueWatchingPoster,
                    contentDescription = null,
                    modifier = Modifier
                        .width(78.dp)
                        .height(112.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .background(Color.White.copy(alpha = 0.06f)),
                    contentScale = ContentScale.Crop
                )
                Column(modifier = Modifier.weight(1f)) {
                    androidx.compose.material3.Text(
                        text = meta.name,
                        color = Color.White,
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Black,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(Modifier.height(6.dp))
                    androidx.compose.material3.Text(
                        text = listOfNotNull(typeLabel, meta.releaseInfo?.takeIf { it.isNotBlank() }, episodeInfo).joinToString("  "),
                        color = Color.White.copy(alpha = 0.62f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(Modifier.height(12.dp))
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(4.dp)
                            .clip(RoundedCornerShape(999.dp))
                            .background(Color.White.copy(alpha = 0.14f))
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(progress)
                                .fillMaxHeight()
                                .background(Color.White)
                        )
                    }
                }
            }

            Spacer(Modifier.height(20.dp))
            ContinueWatchingSheetButton(FluxaIcons.Info, AppStrings.t(lang, "home.view_details"), onDetails)
            Spacer(Modifier.height(10.dp))
            ContinueWatchingSheetButton(FluxaIcons.DeleteOutline, AppStrings.t(lang, "home.forget_progress"), onForget)
        }
    }
}

@Composable
private fun ContinueWatchingSheetButton(icon: ImageVector, label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(52.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.08f))
            .clickable { onClick() }
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        androidx.compose.material3.Icon(icon, null, tint = Color.White, modifier = Modifier.height(22.dp))
        androidx.compose.material3.Text(label, color = Color.White, fontSize = 15.sp, fontWeight = FontWeight.Bold)
    }
}
