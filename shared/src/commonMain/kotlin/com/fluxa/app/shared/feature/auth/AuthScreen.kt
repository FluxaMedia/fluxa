package com.fluxa.app.shared.feature.auth

import androidx.compose.foundation.background
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
fun AuthScreen(
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

    Box(modifier = modifier.fillMaxSize().background(FluxaColors.backgroundNearBlack)) {
        when (state.stage) {
            AuthStage.Credentials -> CredentialsStage(state, language, onAction, nuvioIcon, stremioIcon)
            AuthStage.Nuvio -> NuvioCredentialsStage(state, language, onAction, nuvioIcon)
            AuthStage.NuvioImporting -> NuvioImportingStage(state, language, onAction)
        }
    }
}

@Composable
private fun AuthProviderButton(
    label: String,
    icon: @Composable () -> Unit,
    containerColor: Color,
    contentColor: Color,
    onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(56.dp),
        shape = RoundedCornerShape(14.dp),
        colors = ButtonDefaults.buttonColors(containerColor = containerColor, contentColor = contentColor)
    ) {
        icon()
        Spacer(Modifier.width(12.dp))
        Text(label, fontWeight = FontWeight.Bold, fontSize = 15.sp)
    }
}

@Composable
private fun AuthBackButton(onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(40.dp)
            .clip(CircleShape)
            .background(Color.White.copy(alpha = 0.05f))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null, tint = Color.White, modifier = Modifier.size(20.dp))
    }
}

@Composable
private fun CredentialsStage(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit,
    stremioIcon: @Composable () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp)
    ) {
        Spacer(Modifier.height(16.dp))
        val isRoot = state.showProviderActions
        AuthBackButton(onClick = { onAction(if (isRoot) AuthAction.BackRequested else AuthAction.BackToRoot) })
        Spacer(Modifier.height(24.dp))

        if (state.showProviderActions) {
            AuthProviderButton(
                label = AppStrings.t(language, "auth.continue_with_nuvio"),
                icon = nuvioIcon,
                containerColor = Color.White,
                contentColor = Color.Black,
                onClick = { onAction(AuthAction.ContinueWithNuvio) }
            )
            Spacer(Modifier.height(12.dp))
            AuthProviderButton(
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
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color.White.copy(alpha = 0.05f))
                    .padding(4.dp)
            ) {
                listOf(false to AppStrings.t(language, "auth.log_in"), true to AppStrings.t(language, "auth.sign_up")).forEach { (signup, label) ->
                    val selected = state.isSignupTab == signup
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(10.dp))
                            .background(if (selected) Color.White.copy(alpha = 0.12f) else Color.Transparent)
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
            colors = authFieldColors()
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
            colors = authFieldColors()
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
                colors = authFieldColors()
            )
        }

        state.globalError?.let {
            Text(it, color = FluxaColors.errorRed, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
        }

        Spacer(Modifier.height(16.dp))

        Button(
            onClick = { onAction(AuthAction.Submit) },
            enabled = !state.isSubmitting,
            modifier = Modifier.fillMaxWidth().height(52.dp),
            shape = RoundedCornerShape(14.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
        ) {
            if (state.isSubmitting) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.Black, strokeWidth = 2.dp)
            } else {
                Text(
                    if (state.allowSignup && state.isSignupTab) AppStrings.t(language, "auth.create_account") else AppStrings.t(language, "auth.log_in"),
                    fontWeight = FontWeight.SemiBold
                )
            }
        }

        if (state.showProviderActions) {
            Spacer(Modifier.height(12.dp))
            TextButton(onClick = { onAction(AuthAction.ContinueWithoutAccount) }, modifier = Modifier.fillMaxWidth()) {
                Text(AppStrings.t(language, "auth.continue_without_account"), color = Color.White.copy(alpha = 0.5f))
            }
        }
        Spacer(Modifier.height(24.dp))
    }
}

@Composable
private fun NuvioCredentialsStage(
    state: AuthUiState,
    language: String?,
    onAction: (AuthAction) -> Unit,
    nuvioIcon: @Composable () -> Unit
) {
    Column(modifier = Modifier.fillMaxSize().padding(horizontal = 24.dp)) {
        Spacer(Modifier.height(16.dp))
        AuthBackButton(onClick = { onAction(AuthAction.BackToRoot) })
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
            colors = authFieldColors()
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
            colors = authFieldColors()
        )
        state.globalError?.let {
            Text(it, color = FluxaColors.errorRed, modifier = Modifier.padding(top = 8.dp))
        }
        Spacer(Modifier.height(20.dp))
        Button(
            onClick = { onAction(AuthAction.Submit) },
            enabled = !state.isSubmitting,
            modifier = Modifier.fillMaxWidth().height(52.dp),
            shape = RoundedCornerShape(14.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
        ) {
            if (state.isSubmitting) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.Black, strokeWidth = 2.dp)
            } else {
                Text(AppStrings.t(language, "auth.nuvio.sign_in"), fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

private val IMPORT_STEP_ORDER = listOf(
    AuthImportStep.PROFILE to "auth.nuvio.import.profile",
    AuthImportStep.ADDONS to "auth.nuvio.import.addons",
    AuthImportStep.LIBRARY to "auth.nuvio.import.library",
    AuthImportStep.PROGRESS to "auth.nuvio.import.progress",
    AuthImportStep.HISTORY to "auth.nuvio.import.history",
    AuthImportStep.COLLECTIONS to "auth.nuvio.import.collections"
)

@Composable
private fun NuvioImportingStage(
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
            IMPORT_STEP_ORDER.forEach { (step, key) ->
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
                Button(
                    onClick = { onAction(AuthAction.ContinueAfterImport) },
                    modifier = Modifier.fillMaxWidth().height(52.dp),
                    shape = RoundedCornerShape(14.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
                ) {
                    Text(AppStrings.t(language, "common.continue"))
                }
            }
        }
    }
}

@Composable
private fun authFieldColors() = OutlinedTextFieldDefaults.colors(
    focusedTextColor = Color.White,
    unfocusedTextColor = Color.White,
    focusedBorderColor = Color.White.copy(alpha = 0.4f),
    unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
    focusedLabelColor = Color.White.copy(alpha = 0.7f),
    unfocusedLabelColor = Color.White.copy(alpha = 0.4f),
    cursorColor = Color.White
)
