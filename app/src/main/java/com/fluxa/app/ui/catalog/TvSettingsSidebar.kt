@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class, androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.ui.res.painterResource
import com.fluxa.app.R
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.ClickableSurfaceDefaults
import androidx.tv.material3.Surface
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun SettingsSidebarItem(
    tab: SettingsTab,
    isSelected: Boolean,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    var focused by remember { mutableStateOf(false) }
    val contentColor = if (isSelected || focused) Color.White else Color.White.copy(alpha = 0.6f)
    val iconColor = if (isSelected || focused) Color.White else Color.White.copy(alpha = 0.45f)
    Surface(
        onClick = onClick,
        modifier = modifier
            .fillMaxWidth()
            .height(58.dp)
            .onFocusChanged { focused = it.isFocused },
        shape = ClickableSurfaceDefaults.shape(RoundedCornerShape(16.dp)),
        colors = ClickableSurfaceDefaults.colors(
            containerColor = if (isSelected) Color.White.copy(alpha = 0.1f) else Color.Transparent,
            contentColor = contentColor
        ),
        border = ClickableSurfaceDefaults.border(
            focusedBorder = androidx.tv.material3.Border(
                androidx.compose.foundation.BorderStroke(2.dp, Color.White),
                shape = RoundedCornerShape(16.dp)
            )
        )
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            if (isSelected) {
                Box(
                    modifier = Modifier
                        .width(3.dp)
                        .height(20.dp)
                        .clip(RoundedCornerShape(2.dp))
                        .background(Color.White)
                )
            } else {
                Spacer(Modifier.width(3.dp))
            }
            Icon(tab.icon, null, tint = iconColor)
            Text(tab.label, color = contentColor, fontWeight = if (isSelected) FontWeight.Bold else FontWeight.SemiBold, fontSize = 14.sp)
        }
    }
}
