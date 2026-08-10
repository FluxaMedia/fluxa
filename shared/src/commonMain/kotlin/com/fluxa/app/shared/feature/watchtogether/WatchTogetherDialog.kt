package com.fluxa.app.shared.feature.watchtogether

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun WatchTogetherDialog(
    state: WatchTogetherState,
    config: WatchTogetherConfig,
    localContent: WatchTogetherContent?,
    defaultDisplayName: String,
    onConfigure: (serverUrl: String, secret: String, displayName: String) -> Unit,
    onCreateRoom: () -> Unit,
    onJoinRoom: (String) -> Unit,
    onLeaveRoom: () -> Unit,
    onDismiss: () -> Unit,
) {
    var serverUrl by remember(config.serverUrl) { mutableStateOf(config.serverUrl) }
    var secret by remember(config.serverSecret) { mutableStateOf(config.serverSecret) }
    var displayName by remember(config.displayName, defaultDisplayName) {
        mutableStateOf(config.displayName.takeUnless { it == "Guest" || it.isBlank() } ?: defaultDisplayName.ifBlank { "Guest" })
    }
    var roomCode by remember { mutableStateOf("") }

    LaunchedEffect(state.roomCode) {
        if (state.inRoom) roomCode = ""
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(if (state.inRoom) "Watch Party" else "Start or join a Watch Party") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                if (state.inRoom) {
                    Text(
                        text = state.roomCode.orEmpty(),
                        color = Color.White,
                        fontSize = 34.sp,
                        fontWeight = FontWeight.Black,
                        textAlign = TextAlign.Center,
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(Color.White.copy(alpha = 0.08f), RoundedCornerShape(14.dp))
                            .padding(vertical = 14.dp),
                    )
                    Text(
                        if (state.isHost) "Share this code. You control playback for the room."
                        else "Connected. Playback follows the host.",
                        color = Color.White.copy(alpha = 0.78f),
                    )

                    val remoteContent = state.content
                    if (!state.isHost && remoteContent != null && localContent != null && !remoteContent.matches(localContent)) {
                        Text(
                            text = "The host is watching ${remoteContent.title.ifBlank { remoteContent.id }}. Open the same title/episode for sync.",
                            color = Color(0xFFFFC86B),
                            fontSize = 13.sp,
                        )
                    }

                    if (state.members.isNotEmpty()) {
                        Spacer(Modifier.height(2.dp))
                        Text("Participants (${state.members.size})", fontWeight = FontWeight.SemiBold)
                        state.members.forEach { member ->
                            Text(
                                buildString {
                                    append(if (member.isHost) "★ " else "• ")
                                    append(member.name)
                                    if (member.buffering) append("  · buffering")
                                },
                                color = Color.White.copy(alpha = 0.82f),
                                fontSize = 13.sp,
                            )
                        }
                    }
                } else {
                    OutlinedTextField(
                        value = serverUrl,
                        onValueChange = { serverUrl = it },
                        label = { Text("Self-hosted server") },
                        placeholder = { Text("https://watch.example.com") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    OutlinedTextField(
                        value = secret,
                        onValueChange = { secret = it },
                        label = { Text("Server secret (optional)") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    OutlinedTextField(
                        value = displayName,
                        onValueChange = { displayName = it },
                        label = { Text("Display name") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    OutlinedTextField(
                        value = roomCode,
                        onValueChange = { roomCode = WatchTogetherAddress.normalizeRoomCode(it).take(12) },
                        label = { Text("Room code") },
                        placeholder = { Text("ABC123") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    if (state.connectionState == WatchTogetherConnectionState.CONNECTING) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            CircularProgressIndicator(modifier = Modifier.width(18.dp).height(18.dp), strokeWidth = 2.dp)
                            Text("Connecting…", color = Color.White.copy(alpha = 0.75f))
                        }
                    }
                }

                state.errorMessage?.let {
                    Text(it, color = Color(0xFFFF7B7B), fontSize = 13.sp)
                }
            }
        },
        confirmButton = {
            if (state.inRoom) {
                TextButton(onClick = onDismiss) { Text("Done") }
            } else {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Button(
                        enabled = serverUrl.isNotBlank() && state.connectionState != WatchTogetherConnectionState.CONNECTING,
                        onClick = {
                            onConfigure(serverUrl, secret, displayName)
                            onCreateRoom()
                        },
                    ) { Text("Create room") }
                    Button(
                        enabled = serverUrl.isNotBlank() && WatchTogetherAddress.isValidRoomCode(roomCode) && state.connectionState != WatchTogetherConnectionState.CONNECTING,
                        onClick = {
                            onConfigure(serverUrl, secret, displayName)
                            onJoinRoom(roomCode)
                        },
                    ) { Text("Join") }
                }
            }
        },
        dismissButton = {
            if (state.inRoom) {
                TextButton(onClick = onLeaveRoom) { Text("Leave room", color = Color(0xFFFF8A8A)) }
            } else {
                TextButton(onClick = onDismiss) { Text("Cancel") }
            }
        },
    )
}
