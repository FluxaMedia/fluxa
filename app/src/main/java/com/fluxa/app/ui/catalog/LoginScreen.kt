@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
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
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.tv.material3.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.UUID

private enum class TvLoginView { Qr, Keyboard, Nuvio }

@Composable
fun TvLoginScreen(
    context: android.content.Context,
    onLoginSuccess: (UserProfile) -> Unit,
    onCancel: () -> Unit
) {
    var view by remember { mutableStateOf(TvLoginView.Qr) }

    when (view) {
        TvLoginView.Qr -> TvQrLoginView(
            context = context,
            onLoginSuccess = onLoginSuccess,
            onCancel = onCancel,
            onUseKeyboard = { view = TvLoginView.Keyboard }
        )
        TvLoginView.Keyboard -> TvKeyboardAuthForm(
            onBack = { view = TvLoginView.Qr },
            onLoginSuccess = onLoginSuccess,
            onNuvioClick = { view = TvLoginView.Nuvio }
        )
        TvLoginView.Nuvio -> TvNuvioLoginView(
            onBack = { view = TvLoginView.Keyboard },
            onImported = onLoginSuccess
        )
    }
}

@Composable
private fun TvQrLoginView(
    context: android.content.Context,
    onLoginSuccess: (UserProfile) -> Unit,
    onCancel: () -> Unit,
    onUseKeyboard: () -> Unit
) {
    val service = remember { AppContainer.authService }
    val profileManager = remember { AppContainer.profileManager }
    val nuvioCoordinator = remember { AppContainer.nuvioImportCoordinator }
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

    LaunchedEffect(loginToken) {
        val ip = IntegrationServer.getLocalIpAddress()
        val port = 8585
        if (ip != null) {
            val url = "http://$ip:$port/?token=$loginToken"
            localUrl = url
            qrBitmap = withContext(Dispatchers.Default) {
                QrCodeGenerator.generate(url, transparent = true)
            }

            server = IntegrationServer(
                port = port,
                authToken = loginToken,
                appName = context.getString(com.fluxa.app.R.string.app_name),
                onCredentialsReceived = { email, password ->
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
                },
                onNuvioCredentialsReceived = { email, password ->
                    scope.launch {
                        isLoggingIn = true
                        try {
                            val session = nuvioCoordinator.signIn(email, password).getOrThrow()
                            val baseProfile = UserProfile(
                                id = UUID.randomUUID().toString(),
                                email = session.user?.email ?: email,
                                authKey = ""
                            )
                            val profile = nuvioCoordinator.import(baseProfile, session) {}
                            onLoginSuccess(profile)
                        } catch (e: Exception) {
                            errorMessage = AppStrings.format("en", "login.connection_error", e.localizedMessage)
                        } finally {
                            isLoggingIn = false
                        }
                    }
                }
            )
            server?.start()
        } else {
            errorMessage = AppStrings.t("en", "login.local_network_missing")
        }
    }

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
                    colors = listOf(Color(0xFF221F1F), FluxaColors.backgroundNearBlack),
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
                        Button(
                            onClick = { onUseKeyboard() },
                            colors = ButtonDefaults.colors(containerColor = Color.Transparent)
                        ) {
                            Text(AppStrings.t(lang, "login.use_keyboard"), color = Color.White.copy(alpha = 0.6f))
                        }
                    }
                }
            }
        }
    }
}

private enum class TvAuthTab { Login, Signup }

@Composable
private fun TvKeyboardAuthForm(
    onBack: () -> Unit,
    onLoginSuccess: (UserProfile) -> Unit,
    onNuvioClick: () -> Unit
) {
    val lang = "en"
    val service = remember { AppContainer.authService }
    val profileManager = remember { AppContainer.profileManager }
    val scope = rememberCoroutineScope()

    var tab by remember { mutableStateOf(TvAuthTab.Login) }
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var confirmPassword by remember { mutableStateOf("") }
    var submitting by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }

    fun continueAsGuest() {
        val guest = UserProfile(
            id = UUID.randomUUID().toString(),
            email = AppStrings.t(lang, "auth.primary_profile_name"),
            authKey = "",
            isGuest = false
        )
        profileManager.saveProfile(guest)
        profileManager.setLastActiveProfile(guest)
        onLoginSuccess(guest)
    }

    fun submit() {
        errorMessage = null
        if (email.isBlank() || !android.util.Patterns.EMAIL_ADDRESS.matcher(email).matches()) {
            errorMessage = AppStrings.t(lang, "auth.error.email_invalid")
            return
        }
        if (password.length < 8) {
            errorMessage = AppStrings.t(lang, "auth.error.password_too_short")
            return
        }
        if (tab == TvAuthTab.Signup && password != confirmPassword) {
            errorMessage = AppStrings.t(lang, "auth.error.passwords_mismatch")
            return
        }
        submitting = true
        scope.launch {
            try {
                val request = LoginRequest(email.trim(), password)
                val response = if (tab == TvAuthTab.Login) service.login(request) else service.register(request)
                val result = response.body()?.result
                if (response.isSuccessful && result != null) {
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
                    errorMessage = AppStrings.t(lang, "login.stremio_failed")
                }
            } catch (e: Exception) {
                errorMessage = AppStrings.format(lang, "login.connection_error", e.localizedMessage)
            } finally {
                submitting = false
            }
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.radialGradient(
                    colors = listOf(Color(0xFF221F1F), FluxaColors.backgroundNearBlack),
                    center = androidx.compose.ui.geometry.Offset(0.5f, 0.5f)
                )
            ),
        contentAlignment = Alignment.Center
    ) {
        Column(
            modifier = Modifier.widthIn(max = 460.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = AppStrings.t(lang, if (tab == TvAuthTab.Login) "auth.log_in" else "auth.sign_up"),
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold,
                color = Color.White
            )
            Spacer(Modifier.height(24.dp))

            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                listOf(TvAuthTab.Login to AppStrings.t(lang, "auth.log_in"), TvAuthTab.Signup to AppStrings.t(lang, "auth.sign_up")).forEach { (value, label) ->
                    Button(
                        onClick = { tab = value; errorMessage = null },
                        colors = ButtonDefaults.colors(
                            containerColor = if (tab == value) Color.White.copy(alpha = 0.15f) else Color.Transparent
                        )
                    ) {
                        Text(label, color = Color.White)
                    }
                }
            }

            Spacer(Modifier.height(24.dp))

            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text(AppStrings.t(lang, "auth.field.email")) },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(12.dp))
            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text(AppStrings.t(lang, "auth.field.password")) },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            if (tab == TvAuthTab.Signup) {
                Spacer(Modifier.height(12.dp))
                OutlinedTextField(
                    value = confirmPassword,
                    onValueChange = { confirmPassword = it },
                    label = { Text(AppStrings.t(lang, "auth.field.confirm_password")) },
                    visualTransformation = PasswordVisualTransformation(),
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
            }

            errorMessage?.let {
                Spacer(Modifier.height(8.dp))
                Text(it, color = FluxaColors.errorRed)
            }

            Spacer(Modifier.height(20.dp))

            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Button(onClick = onBack, colors = ButtonDefaults.colors(containerColor = Color.White.copy(alpha = 0.1f))) {
                    Text(AppStrings.t(lang, "common.back"), color = Color.White)
                }
                Button(onClick = { submit() }, enabled = !submitting) {
                    Text(if (submitting) AppStrings.t(lang, "login.loading") else AppStrings.t(lang, "auth.log_in"))
                }
                Button(onClick = onNuvioClick, colors = ButtonDefaults.colors(containerColor = Color.White.copy(alpha = 0.1f))) {
                    Text(AppStrings.t(lang, "auth.continue_with_nuvio"), color = Color.White)
                }
            }

            Spacer(Modifier.height(16.dp))
            Button(onClick = { continueAsGuest() }, colors = ButtonDefaults.colors(containerColor = Color.Transparent)) {
                Text(AppStrings.t(lang, "auth.continue_without_account"), color = Color.White.copy(alpha = 0.5f))
            }
        }
    }
}

@Composable
private fun TvNuvioLoginView(
    onBack: () -> Unit,
    onImported: (UserProfile) -> Unit
) {
    val lang = "en"
    val coordinator = remember { AppContainer.nuvioImportCoordinator }
    val scope = rememberCoroutineScope()

    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var submitting by remember { mutableStateOf(false) }
    var importing by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var completedSteps by remember { mutableStateOf(setOf<NuvioImportStep>()) }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.radialGradient(
                    colors = listOf(Color(0xFF221F1F), FluxaColors.backgroundNearBlack),
                    center = androidx.compose.ui.geometry.Offset(0.5f, 0.5f)
                )
            ),
        contentAlignment = Alignment.Center
    ) {
        if (importing) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text(AppStrings.t(lang, "auth.nuvio.import.title"), color = Color.White, style = MaterialTheme.typography.headlineSmall)
                Spacer(Modifier.height(20.dp))
                listOf(
                    NuvioImportStep.PROFILE to AppStrings.t(lang, "auth.nuvio.import.profile"),
                    NuvioImportStep.ADDONS to AppStrings.t(lang, "auth.nuvio.import.addons"),
                    NuvioImportStep.LIBRARY to AppStrings.t(lang, "auth.nuvio.import.library"),
                    NuvioImportStep.PROGRESS to AppStrings.t(lang, "auth.nuvio.import.progress"),
                    NuvioImportStep.HISTORY to AppStrings.t(lang, "auth.nuvio.import.history"),
                    NuvioImportStep.COLLECTIONS to AppStrings.t(lang, "auth.nuvio.import.collections")
                ).forEach { (step, label) ->
                    val complete = completedSteps.contains(step)
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.padding(vertical = 6.dp)) {
                        Box(
                            modifier = Modifier.size(16.dp).background(
                                if (complete) Color.White.copy(alpha = 0.9f) else Color.White.copy(alpha = 0.08f),
                                CircleShape
                            )
                        )
                        Text(label, color = Color.White.copy(alpha = if (complete) 0.85f else 0.3f))
                    }
                }
            }
        } else {
            Column(modifier = Modifier.widthIn(max = 420.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                Text(AppStrings.t(lang, "auth.nuvio.title"), color = Color.White, style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
                Text(AppStrings.t(lang, "auth.nuvio.subtitle"), color = Color.White.copy(alpha = 0.5f))
                Spacer(Modifier.height(24.dp))
                OutlinedTextField(
                    value = email,
                    onValueChange = { email = it },
                    label = { Text(AppStrings.t(lang, "auth.field.email")) },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(Modifier.height(12.dp))
                OutlinedTextField(
                    value = password,
                    onValueChange = { password = it },
                    label = { Text(AppStrings.t(lang, "auth.field.password")) },
                    visualTransformation = PasswordVisualTransformation(),
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                errorMessage?.let {
                    Spacer(Modifier.height(8.dp))
                    Text(it, color = FluxaColors.errorRed)
                }
                Spacer(Modifier.height(20.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    Button(onClick = onBack, colors = ButtonDefaults.colors(containerColor = Color.White.copy(alpha = 0.1f))) {
                        Text(AppStrings.t(lang, "common.back"), color = Color.White)
                    }
                    Button(
                        enabled = !submitting,
                        onClick = {
                            if (email.isBlank() || password.isBlank()) {
                                errorMessage = AppStrings.t(lang, "auth.error.fill_required")
                                return@Button
                            }
                            submitting = true
                            errorMessage = null
                            scope.launch {
                                val session = coordinator.signIn(email.trim(), password).getOrElse {
                                    errorMessage = AppStrings.t(lang, "auth.error.invalid_credentials")
                                    submitting = false
                                    return@launch
                                }
                                importing = true
                                val baseProfile = UserProfile(
                                    id = UUID.randomUUID().toString(),
                                    email = session.user?.email ?: email,
                                    authKey = ""
                                )
                                val profile = coordinator.import(baseProfile, session) { step ->
                                    completedSteps = completedSteps + step
                                }
                                onImported(profile)
                            }
                        }
                    ) {
                        Text(if (submitting) AppStrings.t(lang, "auth.nuvio.signing_in") else AppStrings.t(lang, "auth.nuvio.sign_in"))
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
            animation = tween(FluxaDimensions.AnimDuration.loginPulse, easing = LinearEasing)
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
