package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.R

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import java.util.UUID

private enum class MobileAuthTab { Login, Signup }

private enum class MobileAuthView { Account, Nuvio, Stremio }

@Composable
internal fun MobileLoginScreen(
    context: android.content.Context,
    onLoginSuccess: (UserProfile) -> Unit,
    onCancel: () -> Unit,
    startOnNuvio: Boolean = false
) {
    var view by remember { mutableStateOf(if (startOnNuvio) MobileAuthView.Nuvio else MobileAuthView.Account) }
    BackHandler(enabled = view != MobileAuthView.Account) {
        view = MobileAuthView.Account
    }

    when (view) {
        MobileAuthView.Account -> MobileAccountLoginView(
            onCancel = onCancel,
            onLoginSuccess = onLoginSuccess,
            onNuvioClick = { view = MobileAuthView.Nuvio },
            onStremioClick = { view = MobileAuthView.Stremio }
        )
        MobileAuthView.Nuvio -> MobileNuvioLoginView(
            onBack = { view = MobileAuthView.Account },
            onImported = onLoginSuccess
        )
        MobileAuthView.Stremio -> MobileStremioLoginView(
            onBack = { view = MobileAuthView.Account },
            onLoginSuccess = onLoginSuccess
        )
    }
}

@Composable
private fun MobileAccountLoginView(
    onCancel: () -> Unit,
    onLoginSuccess: (UserProfile) -> Unit,
    onNuvioClick: () -> Unit,
    onStremioClick: () -> Unit
) {
    MobileAccountCredentialsView(
        onBack = onCancel,
        onLoginSuccess = onLoginSuccess,
        showProviderActions = true,
        allowSignup = true,
        onNuvioClick = onNuvioClick,
        onStremioClick = onStremioClick
    )
}

@Composable
private fun MobileStremioLoginView(
    onBack: () -> Unit,
    onLoginSuccess: (UserProfile) -> Unit
) {
    MobileAccountCredentialsView(
        onBack = onBack,
        onLoginSuccess = onLoginSuccess,
        showProviderActions = false,
        allowSignup = false,
        onNuvioClick = {},
        onStremioClick = {}
    )
}

@Composable
private fun MobileAccountCredentialsView(
    onBack: () -> Unit,
    onLoginSuccess: (UserProfile) -> Unit,
    showProviderActions: Boolean,
    allowSignup: Boolean,
    onNuvioClick: () -> Unit,
    onStremioClick: () -> Unit
) {
    val lang = "en"
    val service = remember { AppContainer.authService }
    val profileManager = remember { AppContainer.profileManager }
    val scope = rememberCoroutineScope()

    var tab by remember { mutableStateOf(MobileAuthTab.Login) }
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var confirmPassword by remember { mutableStateOf("") }
    var showPassword by remember { mutableStateOf(false) }
    var showConfirm by remember { mutableStateOf(false) }
    var submitting by remember { mutableStateOf(false) }
    var emailError by remember { mutableStateOf<String?>(null) }
    var passwordError by remember { mutableStateOf<String?>(null) }
    var confirmError by remember { mutableStateOf<String?>(null) }
    var globalError by remember { mutableStateOf<String?>(null) }

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

    fun resetTabState(next: MobileAuthTab) {
        tab = next
        emailError = null
        passwordError = null
        confirmError = null
        globalError = null
        password = ""
        confirmPassword = ""
        showPassword = false
        showConfirm = false
    }

    fun validate(): Boolean {
        var valid = true
        emailError = null
        passwordError = null
        confirmError = null

        if (email.isBlank()) {
            emailError = AppStrings.t(lang, "auth.error.email_required")
            valid = false
        } else if (!android.util.Patterns.EMAIL_ADDRESS.matcher(email).matches()) {
            emailError = AppStrings.t(lang, "auth.error.email_invalid")
            valid = false
        }

        if (password.isEmpty()) {
            passwordError = AppStrings.t(lang, "auth.error.password_required")
            valid = false
        } else if (password.length < 8) {
            passwordError = AppStrings.t(lang, "auth.error.password_too_short")
            valid = false
        }

        if (allowSignup && tab == MobileAuthTab.Signup && password != confirmPassword) {
            confirmError = AppStrings.t(lang, "auth.error.passwords_mismatch")
            valid = false
        }

        return valid
    }

    fun submit() {
        if (!validate()) return
        submitting = true
        globalError = null
        scope.launch {
            try {
                val request = com.fluxa.app.data.remote.LoginRequest(email.trim(), password)
                val response = if (allowSignup && tab == MobileAuthTab.Signup) service.register(request) else service.login(request)
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
                    globalError = AppStrings.t(lang, "login.stremio_failed")
                }
            } catch (e: Exception) {
                globalError = AppStrings.format(lang, "login.connection_error", e.localizedMessage)
            } finally {
                submitting = false
            }
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(FluxaColors.backgroundNearBlack)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 24.dp)
        ) {
            IconButton(onClick = onBack, modifier = Modifier.padding(top = 8.dp)) {
                Icon(FluxaIcons.ArrowBack, contentDescription = AppStrings.t(lang, "common.back"), tint = Color.White)
            }

            Spacer(Modifier.height(24.dp))

            if (showProviderActions) {
                MobileAuthProviderButton(
                    label = AppStrings.t(lang, "auth.continue_with_nuvio"),
                    iconRes = R.drawable.ic_nuvio,
                    containerColor = Color.White,
                    contentColor = Color.Black,
                    onClick = onNuvioClick,
                )
                Spacer(Modifier.height(12.dp))
                MobileAuthProviderButton(
                    label = AppStrings.t(lang, "auth.continue_with_stremio"),
                    iconRes = R.drawable.ic_stremio,
                    containerColor = Color.White.copy(alpha = 0.08f),
                    contentColor = Color.White,
                    onClick = onStremioClick,
                )
            } else {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Image(
                        painter = painterResource(id = R.drawable.ic_stremio),
                        contentDescription = null,
                        modifier = Modifier.size(28.dp)
                    )
                    Text(AppStrings.t(lang, "auth.login_with_stremio"), color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                }
            }

            Spacer(Modifier.height(24.dp))

            if (allowSignup) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color.White.copy(alpha = 0.05f))
                        .padding(4.dp)
                ) {
                    listOf(
                        MobileAuthTab.Login to AppStrings.t(lang, "auth.log_in"),
                        MobileAuthTab.Signup to AppStrings.t(lang, "auth.sign_up")
                    ).forEach { (value, label) ->
                        val selected = tab == value
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .clip(RoundedCornerShape(10.dp))
                                .background(if (selected) Color.White.copy(alpha = 0.12f) else Color.Transparent)
                                .clickable { resetTabState(value) }
                                .padding(vertical = 10.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(label, color = Color.White, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal)
                        }
                    }
                }
            } else {
                Text(AppStrings.t(lang, "auth.log_in"), color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
            }

            Spacer(Modifier.height(20.dp))

            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text(AppStrings.t(lang, "auth.field.email")) },
                placeholder = { Text(AppStrings.t(lang, "auth.placeholder.email")) },
                isError = emailError != null,
                supportingText = emailError?.let { { Text(it) } },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                colors = mobileAuthFieldColors()
            )

            Spacer(Modifier.height(12.dp))

            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text(AppStrings.t(lang, "auth.field.password")) },
                placeholder = {
                    Text(
                        if (tab == MobileAuthTab.Login) AppStrings.t(lang, "auth.placeholder.password_login")
                        else AppStrings.t(lang, "auth.placeholder.password_signup")
                    )
                },
                isError = passwordError != null,
                supportingText = passwordError?.let { { Text(it) } },
                visualTransformation = if (showPassword) VisualTransformation.None else PasswordVisualTransformation(),
                trailingIcon = {
                    Icon(
                        imageVector = if (showPassword) FluxaIcons.Visibility else FluxaIcons.VisibilityOff,
                        contentDescription = null,
                        tint = Color.White.copy(alpha = 0.6f),
                        modifier = Modifier.size(20.dp).clickable { showPassword = !showPassword }
                    )
                },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                colors = mobileAuthFieldColors()
            )

            if (allowSignup && tab == MobileAuthTab.Signup) {
                Spacer(Modifier.height(12.dp))
                OutlinedTextField(
                    value = confirmPassword,
                    onValueChange = { confirmPassword = it },
                    label = { Text(AppStrings.t(lang, "auth.field.confirm_password")) },
                    placeholder = { Text(AppStrings.t(lang, "auth.placeholder.confirm_password")) },
                    isError = confirmError != null,
                    supportingText = confirmError?.let { { Text(it) } },
                    visualTransformation = if (showConfirm) VisualTransformation.None else PasswordVisualTransformation(),
                    trailingIcon = {
                        Icon(
                            imageVector = if (showConfirm) FluxaIcons.Visibility else FluxaIcons.VisibilityOff,
                            contentDescription = null,
                            tint = Color.White.copy(alpha = 0.6f),
                            modifier = Modifier.size(20.dp).clickable { showConfirm = !showConfirm }
                        )
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = mobileAuthFieldColors()
                )
            }

            if (tab == MobileAuthTab.Login) {
                Row(modifier = Modifier.fillMaxWidth().padding(top = 6.dp), horizontalArrangement = Arrangement.End) {
                    TextButton(onClick = {}) {
                        Text(AppStrings.t(lang, "auth.forgot_password"), color = Color.White.copy(alpha = 0.6f), fontSize = 13.sp)
                    }
                }
            }

            globalError?.let {
                Text(it, color = FluxaColors.errorRed, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
            }

            Spacer(Modifier.height(16.dp))

            Button(
                onClick = { submit() },
                enabled = !submitting,
                modifier = Modifier.fillMaxWidth().height(52.dp),
                shape = RoundedCornerShape(14.dp),
                colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
            ) {
                if (submitting) {
                    CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.Black, strokeWidth = 2.dp)
                } else {
                    Text(
                        if (allowSignup && tab == MobileAuthTab.Signup) AppStrings.t(lang, "auth.create_account") else AppStrings.t(lang, "auth.log_in"),
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }

            if (showProviderActions) {
                Spacer(Modifier.height(12.dp))

                TextButton(
                    onClick = { continueAsGuest() },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(AppStrings.t(lang, "auth.continue_without_account"), color = Color.White.copy(alpha = 0.5f))
                }
            }

            Spacer(Modifier.height(24.dp))
        }
    }
}

@Composable
private fun MobileAuthProviderButton(
    label: String,
    iconRes: Int,
    containerColor: Color,
    contentColor: Color,
    onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(52.dp),
        shape = RoundedCornerShape(14.dp),
        colors = ButtonDefaults.buttonColors(containerColor = containerColor, contentColor = contentColor)
    ) {
        Image(
            painter = painterResource(id = iconRes),
            contentDescription = null,
            modifier = Modifier.size(22.dp)
        )
        Spacer(Modifier.size(12.dp))
        Text(label, color = contentColor, fontWeight = FontWeight.Medium)
    }
}

@Composable
internal fun mobileAuthFieldColors() = OutlinedTextFieldDefaults.colors(
    focusedTextColor = Color.White,
    unfocusedTextColor = Color.White,
    focusedBorderColor = Color.White.copy(alpha = 0.4f),
    unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
    focusedLabelColor = Color.White.copy(alpha = 0.7f),
    unfocusedLabelColor = Color.White.copy(alpha = 0.4f),
    cursorColor = Color.White,
    focusedPlaceholderColor = Color.White.copy(alpha = 0.3f),
    unfocusedPlaceholderColor = Color.White.copy(alpha = 0.3f)
)
