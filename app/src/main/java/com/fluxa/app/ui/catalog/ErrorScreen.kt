package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*

@OptIn(ExperimentalTvMaterial3Api::class)
@Composable
fun ErrorScreen(
    title: String? = null,
    message: String? = null,
    errorId: String = "ERR-STREMIO-500",
    lang: String = "en",
    icon: ImageVector = FluxaIcons.Warning,
    onRetry: () -> Unit
) {
    val resolvedTitle = title ?: AppStrings.t(lang, "error.generic_title")
    val resolvedMessage = message ?: AppStrings.t(lang, "error.generic_message")
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(FluxaColors.backgroundNearBlack),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.width(600.dp)
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = Color.White,
                modifier = Modifier.size(100.dp)
            )
            
            Spacer(modifier = Modifier.height(32.dp))
            
            Text(
                text = resolvedTitle,
                style = MaterialTheme.typography.displaySmall,
                color = Color.White,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            Text(
                text = resolvedMessage,
                style = MaterialTheme.typography.bodyLarge,
                color = Color.LightGray,
                textAlign = TextAlign.Center,
                lineHeight = 28.sp
            )
            
            Spacer(modifier = Modifier.height(12.dp))
            
            Text(
                text = AppStrings.format(lang, "common.error_code", errorId),
                style = MaterialTheme.typography.labelMedium,
                color = Color.Gray
            )
            
            Spacer(modifier = Modifier.height(48.dp))
            
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Button(
                    onClick = onRetry,
                    modifier = Modifier.width(220.dp)
                ) {
                    Icon(FluxaIcons.Refresh, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(AppStrings.t(lang, "common.retry"))
                }
                
                OutlinedButton(
                    onClick = { /* Yardm Sayfas */ },
                    modifier = Modifier.width(220.dp)
                ) {
                    Text(AppStrings.t(lang, "common.help"))
                }
            }
        }
    }
}
