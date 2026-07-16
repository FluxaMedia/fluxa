package com.fluxa.app.shared.feature.player

import com.fluxa.app.ui.catalog.DeviceType

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun MarkSegmentSidebar(
    deviceType: DeviceType,
    lang: String,
    selectedType: String?,
    startMs: Long?,
    endMs: Long?,
    currentPositionMs: Long,
    submitting: Boolean,
    cooldownRemainingSec: Long?,
    feedback: String?,
    onSelectType: (String) -> Unit,
    onMarkStart: () -> Unit,
    onMarkEnd: () -> Unit,
    onAdjustStart: (Long) -> Unit,
    onAdjustEnd: (Long) -> Unit,
    onSubmit: () -> Unit,
    onClose: () -> Unit
) {
    PlayerSidebarShell(
        title = playerText(lang, "mark_segment"),
        subtitle = playerText(lang, "mark_segment_subtitle"),
        deviceType = deviceType,
        onClose = onClose,
        sideSheetOnMobile = false
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            SegmentTypeChip(playerText(lang, "mark_segment_intro"), selectedType == "intro") { onSelectType("intro") }
            SegmentTypeChip(playerText(lang, "mark_segment_recap"), selectedType == "recap") { onSelectType("recap") }
            SegmentTypeChip(playerText(lang, "mark_segment_outro"), selectedType == "outro") { onSelectType("outro") }
        }

        if (selectedType != null) {
            Column(modifier = Modifier.padding(top = 16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(
                    text = formatSegmentTime(currentPositionMs),
                    color = Color.White.copy(alpha = 0.5f),
                    fontSize = 12.sp
                )

                MarkSegmentTimeRow(
                    label = playerText(lang, "mark_segment_set_start"),
                    valueMs = startMs,
                    onMark = onMarkStart,
                    onAdjust = onAdjustStart
                )
                MarkSegmentTimeRow(
                    label = playerText(lang, "mark_segment_set_end"),
                    valueMs = endMs,
                    onMark = onMarkEnd,
                    onAdjust = onAdjustEnd,
                    enabled = startMs != null
                )

                val canSubmit = startMs != null && endMs != null && endMs - startMs >= 2000 && cooldownRemainingSec == null
                MarkSegmentSubmitButton(
                    enabled = canSubmit && !submitting,
                    submitting = submitting,
                    label = when {
                        cooldownRemainingSec != null -> formatCooldown(cooldownRemainingSec)
                        else -> playerText(lang, "mark_segment_submit")
                    },
                    onClick = onSubmit
                )

                feedback?.let {
                    Text(text = it, color = Color.White.copy(alpha = 0.8f), fontSize = 13.sp)
                }
            }
        }
    }
}

@Composable
private fun SegmentTypeChip(label: String, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(if (selected) Color.White else Color.White.copy(alpha = 0.1f))
            .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { onClick() }
            .padding(horizontal = 14.dp, vertical = 8.dp)
    ) {
        Text(
            text = label,
            color = if (selected) Color.Black else Color.White,
            fontSize = 13.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium
        )
    }
}

@Composable
private fun MarkSegmentTimeRow(
    label: String,
    valueMs: Long?,
    onMark: () -> Unit,
    onAdjust: (Long) -> Unit,
    enabled: Boolean = true
) {
    if (valueMs == null) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(44.dp)
                .clip(RoundedCornerShape(10.dp))
                .background(Color.White.copy(alpha = if (enabled) 0.1f else 0.04f))
                .then(
                    if (enabled) {
                        Modifier.clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { onMark() }
                    } else {
                        Modifier
                    }
                ),
            contentAlignment = Alignment.Center
        ) {
            Text(label, color = Color.White.copy(alpha = if (enabled) 0.9f else 0.35f), fontSize = 13.sp, fontWeight = FontWeight.Bold)
        }
    } else {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(
                text = "$label: ${formatSegmentTime(valueMs)}",
                color = Color.White,
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(end = 4.dp)
            )
            NudgeButton("-5s") { onAdjust(-5000) }
            NudgeButton("-1s") { onAdjust(-1000) }
            NudgeButton("+1s") { onAdjust(1000) }
            NudgeButton("+5s") { onAdjust(5000) }
        }
    }
}

@Composable
private fun NudgeButton(label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(8.dp))
            .background(Color.White.copy(alpha = 0.1f))
            .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { onClick() }
            .padding(horizontal = 8.dp, vertical = 6.dp)
    ) {
        Text(label, color = Color.White, fontSize = 11.sp, fontWeight = FontWeight.Bold)
    }
}

@Composable
private fun MarkSegmentSubmitButton(enabled: Boolean, submitting: Boolean, label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(46.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(if (enabled) Color.White else Color.White.copy(alpha = 0.15f))
            .then(
                if (enabled) {
                    Modifier.clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { onClick() }
                } else {
                    Modifier
                }
            ),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = if (submitting) "…" else label,
            color = if (enabled) Color.Black else Color.White.copy(alpha = 0.5f),
            fontSize = 14.sp,
            fontWeight = FontWeight.Black
        )
    }
}

private fun formatSegmentTime(ms: Long): String {
    val totalSeconds = ms / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds - minutes * 60
    return "${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}"
}

private fun formatCooldown(remainingSec: Long): String {
    val minutes = remainingSec / 60
    val seconds = remainingSec % 60
    return if (minutes > 0) "${minutes}m ${seconds}s" else "${seconds}s"
}
