@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
package com.fluxa.app.shared.feature.player

import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.DeviceType
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.FluxaDimensions
import com.fluxa.app.ui.catalog.FluxaIcons

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun PlayerTopIconButton(icon: ImageVector, onClick: () -> Unit, modifier: Modifier = Modifier, contentDescription: String? = null) {
    Box(
        modifier = modifier
            .size(34.dp)
            .clip(CircleShape)
            .background(Color.Black.copy(alpha = 0.38f))
            .border(1.dp, Color.White.copy(alpha = 0.10f), CircleShape)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(icon, contentDescription, tint = Color.White, modifier = Modifier.size(18.dp))
    }
}

@Composable
fun PlayerFlatIconButton(
    icon: ImageVector,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
    enabled: Boolean = true,
    size: Dp = FluxaDimensions.PlayerChrome.iconSize,
    touchSize: Dp = 44.dp,
    pressScale: Boolean = false
) {
    val interactionSource = remember { MutableInteractionSource() }
    val pressed by interactionSource.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressScale && pressed) 0.86f else 1f,
        animationSpec = tween(FluxaDimensions.AnimDuration.blink),
        label = "iconPressScale"
    )
    Box(
        modifier = modifier
            .size(touchSize)
            .graphicsLayer { scaleX = scale; scaleY = scale }
            .clip(CircleShape)
            .clickable(
                interactionSource = interactionSource,
                indication = null,
                enabled = enabled
            ) { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(
            icon,
            contentDescription,
            tint = Color.White.copy(alpha = if (enabled) FluxaDimensions.PlayerChrome.textAlphaPrimary else FluxaDimensions.PlayerChrome.textAlphaDisabled),
            modifier = Modifier.size(size)
        )
    }
}

@Composable
internal fun PlayerOverflowMenuButton(
    lang: String,
    showOverflowMenu: Boolean,
    onToggle: () -> Unit,
    onDismiss: () -> Unit,
    onPictureInPicture: () -> Unit,
    onCast: () -> Unit,
    onOpenInExternalPlayer: () -> Unit
) {
    Box {
        Row(verticalAlignment = Alignment.CenterVertically) {
            PlayerFlatIconButton(
                icon = FluxaIcons.PictureInPictureAlt,
                onClick = onPictureInPicture,
                contentDescription = AppStrings.t(lang, "common.picture_in_picture")
            )
            PlayerFlatIconButton(
                icon = FluxaIcons.MoreVert,
                onClick = onToggle,
                contentDescription = AppStrings.t(lang, "player.more_options")
            )
        }
        AnimatedVisibility(
            visible = showOverflowMenu,
            modifier = Modifier.align(Alignment.TopEnd).offset(y = 48.dp),
            enter = fadeIn(),
            exit = fadeOut()
        ) {
            Column(
                modifier = Modifier
                    .width(200.dp)
                    .clip(RoundedCornerShape(14.dp))
                    .background(Color.Black.copy(alpha = 0.85f))
            ) {
                PlayerOverflowMenuItem(FluxaIcons.Cast, AppStrings.t(lang, "auto.cast")) {
                    onDismiss()
                    onCast()
                }
                PlayerOverflowMenuItem(FluxaIcons.OpenInNew, AppStrings.t(lang, "common.external_player")) {
                    onDismiss()
                    onOpenInExternalPlayer()
                }
            }
        }
    }
}

@Composable
fun PlayerOverflowMenuItem(icon: ImageVector, label: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        Icon(icon, null, tint = Color.White, modifier = Modifier.size(FluxaDimensions.PlayerChrome.iconSize))
        Text(label, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Medium)
    }
}

@Composable
fun MobileBottomAction(
    icon: ImageVector,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    iconOnly: Boolean = false
) {
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(16.dp))
            .clickable { onClick() }
            .padding(horizontal = 4.dp, vertical = 6.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = if (iconOnly) label else null,
            tint = Color.White,
            modifier = Modifier.size(24.dp)
        )
        if (!iconOnly) {
            Spacer(modifier = Modifier.width(6.dp))
            Text(
                text = label,
                color = Color.White,
                fontSize = FluxaDimensions.PlayerChrome.actionLabelTextSize,
                fontWeight = FontWeight.Medium,
                textAlign = TextAlign.Start,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

internal fun Modifier.overlayAboveBottom(gap: Dp): Modifier = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    val gapPx = gap.roundToPx()
    layout(0, 0) {
        placeable.place(0, -gapPx - placeable.height)
    }
}

