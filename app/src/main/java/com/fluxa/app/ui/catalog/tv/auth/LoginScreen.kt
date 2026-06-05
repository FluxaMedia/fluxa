@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.UUID

@Composable
fun LoginScreen(
    context: android.content.Context,
    onLoginSuccess: (UserProfile) -> Unit,
    onCancel: () -> Unit
) {
    val service = remember { AppContainer.authService }
    val profileManager = remember { AppContainer.profileManager }
    val scope = rememberCoroutineScope()
    val lang = "en"
    
    var qrBitmap by remember { mutableStateOf<android.graphics.Bitmap?>(null) }
    var localUrl by remember { mutableStateOf<String?>(null) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var isLoggingIn by remember { mutableStateOf(false) }
    var server: IntegrationServer? by remember { mutableStateOf(null) }
    val loginToken = remember { UUID.randomUUID().toString() }

    DisposableEffect(Unit) {
        onDispose { qrBitmap?.recycle() }
    }

    // Start local server and generate QR
    LaunchedEffect(loginToken) {
        val ip = IntegrationServer.getLocalIpAddress()
        val port = 8585
        if (ip != null) {
            val url = "http://$ip:$port/?token=$loginToken"
            localUrl = url
            qrBitmap = withContext(Dispatchers.Default) {
                QrCodeGenerator.generate(url, transparent = true)
            }
            
            server = IntegrationServer(port = port, authToken = loginToken, appName = context.getString(com.fluxa.app.R.string.app_name), onCredentialsReceived = { email, password ->
                scope.launch {
                    isLoggingIn = true
                    try {
                        val response = service.login(LoginRequest(email, password))
                        if (response.isSuccessful && response.body()?.result != null) {
                            val result = response.body()?.result ?: return@launch
                            val existing = profileManager.getProfiles().firstOrNull {
                                it.id == result.user.id || (!it.isGuest && it.email == result.user.email)
                            }
                            val profile = existing?.copy(
                                id = result.user.id,
                                email = result.user.email,
                                authKey = result.user.authKey,
                                isGuest = false
                            ) ?: UserProfile(result.user.id, result.user.email, result.user.authKey)
                            profileManager.saveProfile(profile)
                            profileManager.setLastActiveProfile(profile)
                            onLoginSuccess(profile)
                        } else {
                            errorMessage = AppStrings.t("en", "login.stremio_failed")
                        }
                    } catch (e: Exception) {
                        errorMessage = AppStrings.format("en", "login.connection_error", e.localizedMessage)
                    } finally {
                        isLoggingIn = false
                    }
                }
            })
            server?.start()
        } else {
            errorMessage = AppStrings.t("en", "login.local_network_missing")
        }
    }

    // Cleanup server on dispose
    DisposableEffect(Unit) {
        onDispose {
            server?.stop()
        }
    }

    val currentError = errorMessage

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.radialGradient(
                    colors = listOf(Color(0xFF221F1F), Color(0xFF0F0F0F)),
                    center = androidx.compose.ui.geometry.Offset(0.5f, 0.5f)
                )
            ),
        contentAlignment = Alignment.Center
    ) {
        if (isLoggingIn) {
            PremiumLoadingIndicator(AppStrings.t(lang, "login.loading"))
        } else if (currentError != null) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text("", fontSize = 64.sp)
                Spacer(modifier = Modifier.height(16.dp))
                Text(currentError, color = Color.White, style = MaterialTheme.typography.bodyLarge)
                Spacer(modifier = Modifier.height(32.dp))
                Button(onClick = { onCancel() }) { Text(AppStrings.t(lang, "common.back")) }
            }
        } else {
            Row(
                modifier = Modifier.fillMaxWidth(0.9f),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(64.dp)
            ) {
                // Left: QR with Glow
                Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.weight(1f)) {
                    Box(
                        modifier = Modifier
                            .size(340.dp)
                            .clip(RoundedCornerShape(32.dp))
                            .background(Color.White)
                            .padding(24.dp)
                    ) {
                        qrBitmap?.let {
                            Image(
                                bitmap = it.asImageBitmap(),
                                contentDescription = AppStrings.t(lang, "login.qr_content_description"),
                                modifier = Modifier.fillMaxSize()
                            )
                        }
                    }
                    Spacer(modifier = Modifier.height(24.dp))
                    Text(
                        text = localUrl?.removePrefix("http://") ?: "",
                        color = Color.Gray,
                        style = MaterialTheme.typography.labelLarge,
                        letterSpacing = 2.sp
                    )
                }

                // Right: Premium Instructions
                Column(modifier = Modifier.weight(1.2f)) {
                    Text(
                        text = AppStrings.t(lang, "login.instant_title"),
                        style = MaterialTheme.typography.displayMedium,
                        fontWeight = FontWeight.Black,
                        color = Color.White
                    )
                    Text(
                        text = AppStrings.t(lang, "login.instant_subtitle"),
                        style = MaterialTheme.typography.bodyLarge,
                        color = Color.White,
                        fontWeight = FontWeight.Medium
                    )
                    
                    Spacer(modifier = Modifier.height(48.dp))
                    
                    InstructionRow("1", AppStrings.t(lang, "login.instruction_scan_qr"))
                    InstructionRow("2", AppStrings.t(lang, "login.instruction_enter_credentials"))
                    InstructionRow("3", AppStrings.t(lang, "login.instruction_synced"))

                    Spacer(modifier = Modifier.height(64.dp))
                    
                    Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        Button(
                            onClick = { onCancel() },
                            colors = ButtonDefaults.colors(containerColor = Color.White.copy(alpha = 0.1f))
                        ) {
                            Text(AppStrings.t(lang, "common.cancel"), color = Color.White)
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun InstructionRow(num: String, text: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.padding(vertical = 10.dp)
    ) {
        Box(
            modifier = Modifier
                .size(32.dp)
                .background(Color.White, RoundedCornerShape(8.dp)),
            contentAlignment = Alignment.Center
        ) {
            Text(num, color = Color.White, fontWeight = FontWeight.Bold)
        }
        Spacer(modifier = Modifier.width(16.dp))
        Text(text, color = Color.LightGray, style = MaterialTheme.typography.bodyLarge)
    }
}

@Composable
fun PremiumLoadingIndicator(text: String) {
    val infiniteTransition = rememberInfiniteTransition(label = "")
    val rotation by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = LinearEasing)
        ),
        label = ""
    )

    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Canvas(modifier = Modifier.size(80.dp)) {
            drawArc(
                color = Color.White,
                startAngle = rotation,
                sweepAngle = 280f,
                useCenter = false,
                style = Stroke(width = 8.dp.toPx(), cap = StrokeCap.Round)
            )
        }
        Spacer(modifier = Modifier.height(24.dp))
        Text(text, color = Color.White, style = MaterialTheme.typography.headlineSmall)
    }
}
