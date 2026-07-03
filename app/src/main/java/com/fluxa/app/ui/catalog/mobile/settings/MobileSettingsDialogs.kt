@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.annotation.DrawableRes
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.drawscope.Stroke
import coil3.compose.AsyncImage
import java.util.Locale

@Composable
internal fun MobileActionRow(
    title: String,
    value: String? = null,
    icon: ImageVector? = null,
    @DrawableRes iconRes: Int? = null,
    onClick: () -> Unit,
    destructive: Boolean = false,
    prominent: Boolean = false,
    connected: Boolean = false
) {
    val colors = LocalMobileSettingsPalette.current

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(horizontal = if (prominent) 16.dp else 12.dp, vertical = if (prominent) 17.dp else 11.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(if (prominent) 14.dp else 10.dp)
    ) {
        Box(
            modifier = Modifier.size(if (prominent) 34.dp else 24.dp),
            contentAlignment = Alignment.Center
        ) {
            when {
                iconRes != null -> {
                    Image(
                        painter = painterResource(id = iconRes),
                        contentDescription = title,
                        modifier = Modifier.size(if (prominent) 28.dp else 20.dp)
                    )
                }

                icon != null -> {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = if (destructive) {
                            Color(0xFFFF5A5A)
                        } else {
                            colors.text.copy(alpha = 0.7f)
                        },
                        modifier = Modifier.size(if (prominent) 24.dp else 18.dp)
                    )
                }
            }
        }

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                color = if (destructive) Color(0xFFFF5A5A) else colors.text.copy(alpha = 0.9f),
                fontSize = if (prominent) 18.sp else 16.sp,
                fontWeight = FontWeight.Bold
            )

            value?.let {
                Text(
                    text = it,
                    color = colors.text.copy(alpha = 0.45f),
                    fontSize = 12.sp
                )
            }
        }

        if (connected) {
            Icon(
                imageVector = FluxaIcons.Check,
                contentDescription = null,
                tint = FluxaColors.successGreen,
                modifier = Modifier.size(18.dp)
            )
        }
        Icon(
            imageVector = FluxaIcons.ChevronRight,
            contentDescription = null,
            tint = colors.text.copy(alpha = 0.34f),
            modifier = Modifier.size(20.dp)
        )
    }
}

@Composable
internal fun MobileInfoRow(title: String, value: String, icon: ImageVector? = null) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 11.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        icon?.let { Icon(it, null, tint = Color.White.copy(alpha = 0.7f), modifier = Modifier.size(17.dp)) }
        Column(modifier = Modifier.weight(1f)) {
            Text(text = title, color = Color.White.copy(alpha = 0.9f), fontSize = 12.sp, fontWeight = FontWeight.Bold)
            Text(text = value, color = Color.White.copy(alpha = 0.42f), fontSize = 9.sp)
        }
    }
}

@Composable
internal fun MobileDivider() {
    val colors = LocalMobileSettingsPalette.current
    Spacer(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 10.dp)
            .height(1.dp)
            .background(colors.divider)
    )
}

@Composable
internal fun MobileChoiceDialog(
    title: String,
    options: List<ChoiceOption>,
    selected: String,
    onDismiss: () -> Unit,
    onSelect: (String) -> Unit
) {
    val colors = LocalMobileSettingsPalette.current
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = colors.card,
        contentColor = colors.text,
        dragHandle = {
            Box(
                modifier = Modifier
                    .padding(top = 8.dp, bottom = 6.dp)
                    .width(38.dp)
                    .height(4.dp)
                    .clip(CircleShape)
                    .background(colors.text.copy(alpha = 0.18f))
            )
        }
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = 520.dp)
                .padding(start = 18.dp, end = 18.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            item {
                Text(title, color = colors.text, fontSize = 16.sp, fontWeight = FontWeight.Black, modifier = Modifier.padding(bottom = 6.dp))
            }
            items(options, key = { option: ChoiceOption -> option.value }) { option ->
                val isSelected = option.value == selected
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .background(if (isSelected) colors.accent.copy(alpha = 0.16f) else Color.Transparent)
                        .clickable { onSelect(option.value) }
                        .padding(horizontal = 14.dp, vertical = 13.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(option.label, color = if (isSelected) colors.accent else colors.text.copy(alpha = 0.72f), fontSize = 14.sp, fontWeight = FontWeight.Bold)
                    if (isSelected) {
                        Icon(FluxaIcons.Check, null, tint = colors.accent, modifier = Modifier.size(18.dp))
                    }
                }
            }
        }
    }
}

internal data class MobileChoiceDialogState(
    val title: String,
    val options: List<ChoiceOption>,
    val selected: String,
    val onSelect: (String) -> Unit
)
