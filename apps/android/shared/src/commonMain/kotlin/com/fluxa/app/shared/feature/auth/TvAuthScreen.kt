package com.fluxa.app.shared.feature.auth

import com.fluxa.app.ui.catalog.FluxaIcons
import com.fluxa.app.ui.catalog.FluxaUiTokens

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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors
import com.fluxa.app.ui.catalog.LocalAccentColor

@Composable
fun TvAuthScreen(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit,
    modifier: Modifier = Modifier
) {
    LaunchedEffect(state.isAuthenticated) {
        if (state.isAuthenticated) onAction(AuthAction.Completed)
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(FluxaColors.backgroundNearBlack)
            .padding(horizontal = 48.dp, vertical = 24.dp),
        contentAlignment = Alignment.Center
    ) {
        Box(modifier = Modifier.widthIn(max = 600.dp).fillMaxWidth()) {
            when (state.stage) {
                AuthStage.Credentials -> TvCredentialsStage(state, language, onAction, nuvioIcon)
                AuthStage.Nuvio -> TvNuvioCredentialsStage(state, language, onAction, nuvioIcon)
                AuthStage.NuvioImporting -> TvNuvioImportingStage(state, language, onAction)
                AuthStage.DeviceQr -> DeviceQrStage(state, language, onAction)
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
            .border(width = 0.dp, color = Color.Transparent, shape = shape)
    ) {
        content()
    }
}

@Composable
private fun TvAuthDivider(language: String?) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Box(Modifier.weight(1f).height(1.dp).background(FluxaUiTokens.colorBorder))
        Text(
            text = AppStrings.t(language, "auth.or"),
            color = FluxaUiTokens.colorTextMuted,
            fontSize = FluxaUiTokens.typographyAuthCaption,
            fontWeight = FontWeight.SemiBold
        )
        Box(Modifier.weight(1f).height(1.dp).background(FluxaUiTokens.colorBorder))
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
                .height(FluxaUiTokens.authProviderHeight)
                .clip(RoundedCornerShape(FluxaUiTokens.shapeControl))
                .background(if (focused) Color.White else FluxaColors.surfaceRaised)
                .clickable(onClick = onClick)
                .padding(horizontal = 24.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            icon()
            Spacer(Modifier.width(12.dp))
            Text(
                label,
                color = if (focused) Color.Black else contentColor,
                fontWeight = FontWeight.SemiBold,
                fontSize = FluxaUiTokens.typographyAuthControl,
            )
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
                .height(FluxaUiTokens.authSubmitHeight)
                .background(if (focused) LocalAccentColor.current else FluxaColors.surfaceRaised)
                .clickable(enabled = !isSubmitting, onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            if (isSubmitting) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.White, strokeWidth = 2.dp)
            } else {
                Text(label, color = Color.White, fontWeight = FontWeight.SemiBold, fontSize = FluxaUiTokens.typographyAuthControl)
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
                .background(if (focused) Color.White else Color.White.copy(alpha = 0.05f), CircleShape)
                .clickable(onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            Icon(FluxaIcons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
        }
    }
}

@Composable
private fun TvCredentialsStage(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit,
) {
    var passwordVisible by remember { mutableStateOf(false) }
    var confirmPasswordVisible by remember { mutableStateOf(false) }
    val primaryFocusRequester = remember { FocusRequester() }
    LaunchedEffect(state.showProviderActions) {
        primaryFocusRequester.requestFocus()
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = FluxaUiTokens.spacingScreen, vertical = 28.dp)
    ) {
        if (state.showProviderActions) {
            Spacer(Modifier.height(40.dp))
        } else {
            TvAuthBackButton(onClick = { onAction(AuthAction.BackToRoot) })
            Spacer(Modifier.height(32.dp))
        }

        if (state.showProviderActions) {
            Text(
                text = AppStrings.t(language, "auth.welcome_back"),
                color = Color.White,
                fontSize = FluxaUiTokens.typographyAuthTitle,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.fillMaxWidth(),
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                text = AppStrings.t(language, "auth.choose_login_method"),
                color = Color.White.copy(alpha = 0.58f),
                fontSize = FluxaUiTokens.typographyAuthSubtitle,
                modifier = Modifier.fillMaxWidth(),
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(FluxaUiTokens.spacingAuthSection))
        } else {
            Spacer(Modifier.height(24.dp))
        }

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(0.dp)
        ) {
        OutlinedTextField(
            value = state.email,
            onValueChange = { onAction(AuthAction.EmailChanged(it)) },
            label = { Text(AppStrings.t(language, "auth.field.email")) },
            isError = state.emailError != null,
            supportingText = state.emailError?.let { { Text(it) } },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            singleLine = true,
            shape = RoundedCornerShape(FluxaUiTokens.shapeControl),
            modifier = Modifier.fillMaxWidth().heightIn(min = FluxaUiTokens.authFieldHeight),
            colors = tvAuthFieldColors()
        )
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            value = state.password,
            onValueChange = { onAction(AuthAction.PasswordChanged(it)) },
            label = { Text(AppStrings.t(language, "auth.field.password")) },
            isError = state.passwordError != null,
            supportingText = state.passwordError?.let { { Text(it) } },
            visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
            trailingIcon = {
                AuthPasswordVisibilityToggle(passwordVisible, language) { passwordVisible = !passwordVisible }
            },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            colors = tvAuthFieldColors()
        )
        if (state.allowSignup && state.isSignupTab) {
            Spacer(Modifier.height(FluxaUiTokens.spacingControl))
            OutlinedTextField(
                value = state.confirmPassword,
                onValueChange = { onAction(AuthAction.ConfirmPasswordChanged(it)) },
                label = { Text(AppStrings.t(language, "auth.field.confirm_password")) },
                isError = state.confirmError != null,
                supportingText = state.confirmError?.let { { Text(it) } },
                visualTransformation = if (confirmPasswordVisible) VisualTransformation.None else PasswordVisualTransformation(),
                trailingIcon = {
                    AuthPasswordVisibilityToggle(confirmPasswordVisible, language) { confirmPasswordVisible = !confirmPasswordVisible }
                },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                colors = tvAuthFieldColors()
            )
        }

        state.globalError?.let {
            Text(it, color = FluxaColors.errorRed, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
        }

        Spacer(Modifier.height(FluxaUiTokens.spacingAuthSection))

        TvAuthSubmitButton(
            label = if (state.allowSignup && state.isSignupTab) AppStrings.t(language, "auth.create_account") else AppStrings.t(language, "auth.log_in"),
            isSubmitting = state.isSubmitting,
            onClick = { onAction(AuthAction.Submit) }
        )

        if (state.allowSignup) {
            var switchFocused by remember { mutableStateOf(false) }
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .onFocusChanged { switchFocused = it.isFocused }
                    .border(
                        width = if (switchFocused) 2.dp else 0.dp,
                        color = Color.White,
                        shape = RoundedCornerShape(FluxaUiTokens.shapeControl)
                    )
                    .clickable { onAction(AuthAction.TabChanged(!state.isSignupTab)) }
                    .padding(vertical = 10.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    AppStrings.t(
                        language,
                        if (state.isSignupTab) "auth.already_have_account_sign_in" else "auth.no_account_sign_up"
                    ),
                    color = FluxaUiTokens.colorTextSecondary,
                    fontSize = FluxaUiTokens.typographyAuthCaption,
                    fontWeight = FontWeight.Medium
                )
            }
        }
        }

        if (state.showProviderActions && !state.isSignupTab) {
            Spacer(Modifier.height(FluxaUiTokens.spacingAuthSection))
            TvAuthDivider(language)
            Spacer(Modifier.height(FluxaUiTokens.spacingControl))
            TvAuthProviderButton(
                label = AppStrings.t(language, "auth.continue_with_nuvio"),
                icon = nuvioIcon,
                containerColor = FluxaUiTokens.colorSurfaceRaised,
                contentColor = Color.White,
                onClick = { onAction(AuthAction.ContinueWithNuvio) },
                focusRequester = primaryFocusRequester
            )
            Spacer(Modifier.height(FluxaUiTokens.spacingControl))
            var skipFocused by remember { mutableStateOf(false) }
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(FluxaUiTokens.shapeControl))
                    .border(width = if (skipFocused) 2.dp else 0.dp, color = Color.White, shape = RoundedCornerShape(FluxaUiTokens.shapeControl))
                    .onFocusChanged { skipFocused = it.isFocused }
                    .clickable { onAction(AuthAction.ContinueWithoutAccount) }
                    .padding(vertical = 10.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(AppStrings.t(language, "auth.continue_without_account"), color = FluxaUiTokens.colorTextMuted, fontSize = FluxaUiTokens.typographyAuthCaption)
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
    var passwordVisible by remember { mutableStateOf(false) }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = FluxaUiTokens.spacingScreen, vertical = 28.dp)
    ) {
        TvAuthBackButton(onClick = { onAction(AuthAction.BackToRoot) })
        Spacer(Modifier.height(24.dp))
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            nuvioIcon()
            Spacer(Modifier.height(12.dp))
            Text(
                AppStrings.t(language, "auth.nuvio.title"),
                color = Color.White,
                fontSize = FluxaUiTokens.typographyAuthTitle,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                AppStrings.t(language, "auth.nuvio.subtitle"),
                color = FluxaUiTokens.colorTextSecondary,
                fontSize = FluxaUiTokens.typographyAuthSubtitle,
                textAlign = TextAlign.Center,
            )
        }
        Spacer(Modifier.height(FluxaUiTokens.spacingAuthSection))
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(FluxaUiTokens.shapeAuthCard))
                .background(FluxaUiTokens.colorSurface.copy(alpha = 0.72f))
                .border(1.dp, FluxaUiTokens.colorBorder, RoundedCornerShape(FluxaUiTokens.shapeAuthCard))
                .padding(FluxaUiTokens.spacingAuthCardPadding)
        ) {
            OutlinedTextField(
                value = state.email,
                onValueChange = { onAction(AuthAction.EmailChanged(it)) },
                placeholder = { Text(AppStrings.t(language, "auth.field.email")) },
                leadingIcon = { Icon(FluxaIcons.Filled.Email, contentDescription = null, modifier = Modifier.size(20.dp)) },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                singleLine = true,
                shape = RoundedCornerShape(FluxaUiTokens.shapeControl),
                modifier = Modifier.fillMaxWidth().heightIn(min = FluxaUiTokens.authFieldHeight),
                colors = tvAuthFieldColors()
            )
            Spacer(Modifier.height(FluxaUiTokens.spacingControl))
            OutlinedTextField(
                value = state.password,
                onValueChange = { onAction(AuthAction.PasswordChanged(it)) },
                placeholder = { Text(AppStrings.t(language, "auth.field.password")) },
                leadingIcon = { Icon(FluxaIcons.Filled.Lock, contentDescription = null, modifier = Modifier.size(20.dp)) },
                visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
                trailingIcon = {
                    AuthPasswordVisibilityToggle(passwordVisible, language) { passwordVisible = !passwordVisible }
                },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                singleLine = true,
                shape = RoundedCornerShape(FluxaUiTokens.shapeControl),
                modifier = Modifier.fillMaxWidth().heightIn(min = FluxaUiTokens.authFieldHeight),
                colors = tvAuthFieldColors()
            )
            state.globalError?.let {
                Text(it, color = FluxaColors.errorRed, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
            }
            Spacer(Modifier.height(FluxaUiTokens.spacingAuthSection))
            TvAuthSubmitButton(
                label = AppStrings.t(language, "auth.nuvio.sign_in"),
                isSubmitting = state.isSubmitting,
                onClick = { onAction(AuthAction.Submit) }
            )
        }
        Spacer(Modifier.height(24.dp))
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
                val active = !complete && !state.importDone && (state.importSteps.size == TV_IMPORT_STEP_ORDER.indexOfFirst { it.first == step })
                Column(modifier = Modifier.padding(vertical = 6.dp)) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(18.dp)
                                .background(
                                    if (complete) Color.White.copy(alpha = 0.9f) else Color.White.copy(alpha = 0.08f),
                                    CircleShape
                                ),
                            contentAlignment = Alignment.Center
                        ) {
                            if (complete) {
                                Icon(
                                    imageVector = FluxaIcons.Filled.Check,
                                    contentDescription = null,
                                    tint = Color.Black,
                                    modifier = Modifier.size(12.dp)
                                )
                            }
                        }
                        Text(AppStrings.t(language, key), color = Color.White.copy(alpha = if (complete) 0.85f else 0.3f))
                    }
                    if (active && state.importItemTitle != null && state.importItemIndex != null && state.importItemTotal != null) {
                        Text(
                            AppStrings.format(
                                language,
                                "auth.nuvio.import.item_progress",
                                state.importItemIndex,
                                state.importItemTotal,
                                state.importItemTitle
                            ),
                            color = Color.White.copy(alpha = 0.5f),
                            fontSize = 12.sp,
                            modifier = Modifier.padding(start = 28.dp, top = 2.dp)
                        )
                    }
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
    focusedTextColor = FluxaUiTokens.colorTextPrimary,
    unfocusedTextColor = FluxaUiTokens.colorTextPrimary,
    focusedBorderColor = FluxaUiTokens.colorFocus,
    unfocusedBorderColor = FluxaUiTokens.colorBorderStrong,
    focusedLabelColor = FluxaUiTokens.colorTextPrimary.copy(alpha = 0.9f),
    unfocusedLabelColor = FluxaUiTokens.colorTextMuted,
    focusedContainerColor = FluxaUiTokens.colorSurfaceRaised.copy(alpha = 0.72f),
    unfocusedContainerColor = FluxaUiTokens.colorSurface.copy(alpha = 0.72f),
    focusedTrailingIconColor = FluxaUiTokens.colorTextSecondary,
    unfocusedTrailingIconColor = FluxaUiTokens.colorTextMuted,
    cursorColor = FluxaUiTokens.colorFocus
)
