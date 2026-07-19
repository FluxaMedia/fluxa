package com.fluxa.app.shared.feature.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun TvAuthScreen(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit,
    stremioIcon: @Composable () -> Unit,
    modifier: Modifier = Modifier
) {
    LaunchedEffect(state.isAuthenticated) {
        if (state.isAuthenticated) onAction(AuthAction.Completed)
    }

    Box(
        modifier = modifier.fillMaxSize().background(FluxaColors.backgroundNearBlack),
        contentAlignment = Alignment.Center
    ) {
        Box(modifier = Modifier.widthIn(max = 480.dp).fillMaxWidth()) {
            when (state.stage) {
                AuthStage.Credentials -> TvCredentialsStage(state, language, onAction, nuvioIcon, stremioIcon)
                AuthStage.Nuvio -> TvNuvioCredentialsStage(state, language, onAction, nuvioIcon)
                AuthStage.NuvioImporting -> TvNuvioImportingStage(state, language, onAction)
            }
        }
    }
}

@Composable
private fun TvFocusRing(
    modifier: Modifier = Modifier,
    focused: Boolean,
    shape: androidx.compose.ui.graphics.Shape,
    content: @Composable () -> Unit
) {
    Box(
        modifier = modifier
            .clip(shape)
            .border(width = if (focused) 3.dp else 0.dp, color = Color.White, shape = shape)
    ) {
        content()
    }
}

@Composable
private fun TvAuthProviderButton(
    label: String,
    icon: @Composable () -> Unit,
    containerColor: Color,
    contentColor: Color,
    onClick: () -> Unit,
    focusRequester: FocusRequester? = null
) {
    var focused by remember { mutableStateOf(false) }
    TvFocusRing(focused = focused, shape = RoundedCornerShape(14.dp)) {
        Row(
            modifier = Modifier
                .let { if (focusRequester != null) it.focusRequester(focusRequester) else it }
                .onFocusChanged { focused = it.isFocused }
                .fillMaxWidth()
                .height(56.dp)
                .background(containerColor)
                .clickable(onClick = onClick)
                .padding(horizontal = 20.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            icon()
            Spacer(Modifier.width(12.dp))
            Text(label, color = contentColor, fontWeight = FontWeight.Bold, fontSize = 15.sp)
        }
    }
}

@Composable
private fun TvAuthSubmitButton(
    label: String,
    isSubmitting: Boolean,
    onClick: () -> Unit,
    focusRequester: FocusRequester? = null
) {
    var focused by remember { mutableStateOf(false) }
    TvFocusRing(focused = focused, shape = RoundedCornerShape(14.dp)) {
        Box(
            modifier = Modifier
                .let { if (focusRequester != null) it.focusRequester(focusRequester) else it }
                .onFocusChanged { focused = it.isFocused }
                .fillMaxWidth()
                .height(52.dp)
                .background(if (isSubmitting) Color.White.copy(alpha = 0.6f) else Color.White)
                .clickable(enabled = !isSubmitting, onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            if (isSubmitting) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.Black, strokeWidth = 2.dp)
            } else {
                Text(label, color = Color.Black, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

@Composable
private fun TvAuthBackButton(onClick: () -> Unit) {
    var focused by remember { mutableStateOf(false) }
    TvFocusRing(focused = focused, shape = CircleShape) {
        Box(
            modifier = Modifier
                .size(40.dp)
                .onFocusChanged { focused = it.isFocused }
                .background(Color.White.copy(alpha = 0.05f))
                .clickable(onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
        }
    }
}

@Composable
private fun TvCredentialsStage(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit,
    stremioIcon: @Composable () -> Unit
) {
    val primaryFocusRequester = remember { FocusRequester() }
    LaunchedEffect(state.showProviderActions) {
        primaryFocusRequester.requestFocus()
    }

    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 40.dp)) {
        TvAuthBackButton(onClick = { onAction(if (state.showProviderActions) AuthAction.BackRequested else AuthAction.BackToRoot) })
        Spacer(Modifier.height(24.dp))

        if (state.showProviderActions) {
            TvAuthProviderButton(
                label = AppStrings.t(language, "auth.continue_with_nuvio"),
                icon = nuvioIcon,
                containerColor = Color.White,
                contentColor = Color.Black,
                onClick = { onAction(AuthAction.ContinueWithNuvio) },
                focusRequester = primaryFocusRequester
            )
            Spacer(Modifier.height(12.dp))
            TvAuthProviderButton(
                label = AppStrings.t(language, "auth.continue_with_stremio"),
                icon = stremioIcon,
                containerColor = Color.White.copy(alpha = 0.08f),
                contentColor = Color.White,
                onClick = {}
            )
            Spacer(Modifier.height(24.dp))
        } else {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                stremioIcon()
                Text(AppStrings.t(language, "auth.login_with_stremio"), color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
            }
            Spacer(Modifier.height(24.dp))
        }

        if (state.allowSignup) {
            var plannedFocused by remember { mutableStateOf(false) }
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White.copy(alpha = 0.05f))
                    .padding(4.dp)
            ) {
                listOf(false to AppStrings.t(language, "auth.log_in"), true to AppStrings.t(language, "auth.sign_up")).forEach { (signup, label) ->
                    val selected = state.isSignupTab == signup
                    var focused by remember(signup) { mutableStateOf(false) }
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(10.dp))
                            .border(width = if (focused) 2.dp else 0.dp, color = Color.White, shape = RoundedCornerShape(10.dp))
                            .background(if (selected) Color.White.copy(alpha = 0.12f) else Color.Transparent)
                            .onFocusChanged { focused = it.isFocused }
                            .clickable { onAction(AuthAction.TabChanged(signup)) }
                            .padding(vertical = 10.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(label, color = Color.White, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal)
                    }
                }
            }
        } else {
            Text(AppStrings.t(language, "auth.log_in"), color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        }

        Spacer(Modifier.height(20.dp))

        OutlinedTextField(
            value = state.email,
            onValueChange = { onAction(AuthAction.EmailChanged(it)) },
            label = { Text(AppStrings.t(language, "auth.field.email")) },
            isError = state.emailError != null,
            supportingText = state.emailError?.let { { Text(it) } },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = tvAuthFieldColors()
        )
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            value = state.password,
            onValueChange = { onAction(AuthAction.PasswordChanged(it)) },
            label = { Text(AppStrings.t(language, "auth.field.password")) },
            isError = state.passwordError != null,
            supportingText = state.passwordError?.let { { Text(it) } },
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = tvAuthFieldColors()
        )
        if (state.allowSignup && state.isSignupTab) {
            Spacer(Modifier.height(12.dp))
            OutlinedTextField(
                value = state.confirmPassword,
                onValueChange = { onAction(AuthAction.ConfirmPasswordChanged(it)) },
                label = { Text(AppStrings.t(language, "auth.field.confirm_password")) },
                isError = state.confirmError != null,
                supportingText = state.confirmError?.let { { Text(it) } },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                colors = tvAuthFieldColors()
            )
        }

        state.globalError?.let {
            Text(it, color = FluxaColors.errorRed, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
        }

        Spacer(Modifier.height(16.dp))

        TvAuthSubmitButton(
            label = if (state.allowSignup && state.isSignupTab) AppStrings.t(language, "auth.create_account") else AppStrings.t(language, "auth.log_in"),
            isSubmitting = state.isSubmitting,
            onClick = { onAction(AuthAction.Submit) }
        )

        if (state.showProviderActions) {
            Spacer(Modifier.height(12.dp))
            var skipFocused by remember { mutableStateOf(false) }
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(10.dp))
                    .border(width = if (skipFocused) 2.dp else 0.dp, color = Color.White, shape = RoundedCornerShape(10.dp))
                    .onFocusChanged { skipFocused = it.isFocused }
                    .clickable { onAction(AuthAction.ContinueWithoutAccount) }
                    .padding(vertical = 12.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(AppStrings.t(language, "auth.continue_without_account"), color = Color.White.copy(alpha = 0.5f))
            }
        }
    }
}

@Composable
private fun TvNuvioCredentialsStage(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 40.dp)) {
        TvAuthBackButton(onClick = { onAction(AuthAction.BackToRoot) })
        Spacer(Modifier.height(16.dp))
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            nuvioIcon()
            Text(AppStrings.t(language, "auth.nuvio.title"), color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
        }
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(
            value = state.email,
            onValueChange = { onAction(AuthAction.EmailChanged(it)) },
            label = { Text(AppStrings.t(language, "auth.field.email")) },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = tvAuthFieldColors()
        )
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            value = state.password,
            onValueChange = { onAction(AuthAction.PasswordChanged(it)) },
            label = { Text(AppStrings.t(language, "auth.field.password")) },
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = tvAuthFieldColors()
        )
        state.globalError?.let {
            Text(it, color = FluxaColors.errorRed, modifier = Modifier.padding(top = 8.dp))
        }
        Spacer(Modifier.height(20.dp))
        TvAuthSubmitButton(
            label = AppStrings.t(language, "auth.nuvio.sign_in"),
            isSubmitting = state.isSubmitting,
            onClick = { onAction(AuthAction.Submit) }
        )
    }
}

@Composable
private fun TvNuvioImportingStage(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit
) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                if (state.importDone) AppStrings.t(language, "auth.nuvio.import.done") else AppStrings.t(language, "auth.nuvio.import.title"),
                color = Color.White,
                fontWeight = FontWeight.SemiBold
            )
            Spacer(Modifier.height(20.dp))
            TV_IMPORT_STEP_ORDER.forEach { (step, key) ->
                val complete = step in state.importSteps
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.padding(vertical = 6.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .size(18.dp)
                            .background(
                                if (complete) Color.White.copy(alpha = 0.9f) else Color.White.copy(alpha = 0.08f),
                                CircleShape
                            )
                    )
                    Text(AppStrings.t(language, key), color = Color.White.copy(alpha = if (complete) 0.85f else 0.3f))
                }
            }
            if (state.importDone) {
                Spacer(Modifier.height(24.dp))
                val focusRequester = remember { FocusRequester() }
                LaunchedEffect(Unit) { focusRequester.requestFocus() }
                TvAuthSubmitButton(
                    label = AppStrings.t(language, "common.continue"),
                    isSubmitting = false,
                    onClick = { onAction(AuthAction.ContinueAfterImport) },
                    focusRequester = focusRequester
                )
            }
        }
    }
}

private val TV_IMPORT_STEP_ORDER = listOf(
    AuthImportStep.PROFILE to "auth.nuvio.import.profile",
    AuthImportStep.ADDONS to "auth.nuvio.import.addons",
    AuthImportStep.LIBRARY to "auth.nuvio.import.library",
    AuthImportStep.PROGRESS to "auth.nuvio.import.progress",
    AuthImportStep.HISTORY to "auth.nuvio.import.history",
    AuthImportStep.COLLECTIONS to "auth.nuvio.import.collections"
)

@Composable
private fun tvAuthFieldColors() = OutlinedTextFieldDefaults.colors(
    focusedTextColor = Color.White,
    unfocusedTextColor = Color.White,
    focusedBorderColor = Color.White,
    unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
    focusedLabelColor = Color.White.copy(alpha = 0.9f),
    unfocusedLabelColor = Color.White.copy(alpha = 0.4f),
    cursorColor = Color.White
)
