package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.NuvioSession
import com.fluxa.app.data.repository.NuvioImportStep
import com.fluxa.app.R

import androidx.compose.foundation.Image
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
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private enum class MobileNuvioView { Credentials, Importing }

@Composable
internal fun MobileNuvioLoginView(
    onBack: () -> Unit,
    onImported: (UserProfile) -> Unit
) {
    val lang = "en"
    val coordinator = remember { AppContainer.nuvioImportCoordinator }

    var view by remember { mutableStateOf(MobileNuvioView.Credentials) }
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var showPassword by remember { mutableStateOf(false) }
    var submitting by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var authenticatedSession by remember { mutableStateOf<NuvioSession?>(null) }
    var importedProfile by remember { mutableStateOf<UserProfile?>(null) }

    Box(modifier = Modifier.fillMaxSize().background(FluxaColors.backgroundNearBlack)) {
        Column(modifier = Modifier.fillMaxSize().statusBarsPadding().padding(horizontal = 24.dp)) {
            IconButton(onClick = onBack, modifier = Modifier.padding(top = 8.dp)) {
                Icon(FluxaIcons.ArrowBack, contentDescription = AppStrings.t(lang, "common.back"), tint = Color.White)
            }

            Spacer(Modifier.height(16.dp))

            when (view) {
                MobileNuvioView.Credentials -> {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Image(
                            painter = painterResource(id = R.drawable.ic_nuvio),
                            contentDescription = null,
                            modifier = Modifier.size(28.dp)
                        )
                        Text(AppStrings.t(lang, "auth.nuvio.title"), color = Color.White, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                    }

                    Spacer(Modifier.height(24.dp))

                    OutlinedTextField(
                        value = email,
                        onValueChange = { email = it },
                        label = { Text(AppStrings.t(lang, "auth.field.email")) },
                        placeholder = { Text(AppStrings.t(lang, "auth.placeholder.email")) },
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
                        placeholder = { Text(AppStrings.t(lang, "auth.placeholder.password_login")) },
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

                    error?.let {
                        Text(it, color = FluxaColors.errorRed, modifier = Modifier.padding(top = 8.dp))
                    }

                    Spacer(Modifier.height(20.dp))

                    Button(
                        onClick = {
                            if (email.isBlank() || password.isBlank()) {
                                error = AppStrings.t(lang, "auth.error.fill_required")
                                return@Button
                            }
                            error = null
                            submitting = true
                        },
                        enabled = !submitting,
                        modifier = Modifier.fillMaxWidth().height(52.dp),
                        shape = RoundedCornerShape(14.dp),
                        colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
                    ) {
                        if (submitting) {
                            CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.Black, strokeWidth = 2.dp)
                        } else {
                            Text(AppStrings.t(lang, "auth.nuvio.sign_in"), fontWeight = FontWeight.SemiBold)
                        }
                    }

                    if (submitting) {
                        LaunchedEffect(Unit) {
                            val result = coordinator.signIn(email.trim(), password)
                            result.fold(
                                onSuccess = { session ->
                                    authenticatedSession = session
                                    view = MobileNuvioView.Importing
                                },
                                onFailure = {
                                    error = AppStrings.t(lang, "auth.error.invalid_credentials")
                                    submitting = false
                                }
                            )
                        }
                    }
                }

                MobileNuvioView.Importing -> {
                    val steps = remember {
                        mutableStateOf(setOf<NuvioImportStep>())
                    }
                    var done by remember { mutableStateOf(false) }

                    LaunchedEffect(Unit) {
                        try {
                            val session = authenticatedSession ?: coordinator.signIn(email.trim(), password).getOrNull() ?: run {
                                error = AppStrings.t(lang, "auth.error.invalid_credentials")
                                submitting = false
                                view = MobileNuvioView.Credentials
                                return@LaunchedEffect
                            }
                            val baseProfile = UserProfile(
                                id = java.util.UUID.randomUUID().toString(),
                                email = session.user?.email ?: email,
                                authKey = ""
                            )
                            val result = coordinator.import(baseProfile, session) { step ->
                                steps.value = steps.value + step
                            }
                            importedProfile = result
                            done = true
                        } catch (_: Exception) {
                            error = AppStrings.t(lang, "auth.error.network")
                            submitting = false
                            view = MobileNuvioView.Credentials
                        }
                    }

                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                if (done) AppStrings.t(lang, "auth.nuvio.import.done") else AppStrings.t(lang, "auth.nuvio.import.title"),
                                color = Color.White,
                                fontWeight = FontWeight.SemiBold
                            )
                            Spacer(Modifier.height(20.dp))
                            listOf(
                                NuvioImportStep.PROFILE to AppStrings.t(lang, "auth.nuvio.import.profile"),
                                NuvioImportStep.ADDONS to AppStrings.t(lang, "auth.nuvio.import.addons"),
                                NuvioImportStep.LIBRARY to AppStrings.t(lang, "auth.nuvio.import.library"),
                                NuvioImportStep.PROGRESS to AppStrings.t(lang, "auth.nuvio.import.progress"),
                                NuvioImportStep.HISTORY to AppStrings.t(lang, "auth.nuvio.import.history"),
                                NuvioImportStep.COLLECTIONS to AppStrings.t(lang, "auth.nuvio.import.collections")
                            ).forEach { (step, label) ->
                                val complete = steps.value.contains(step)
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
                                    Text(label, color = Color.White.copy(alpha = if (complete) 0.85f else 0.3f))
                                }
                            }

                            if (done) {
                                Spacer(Modifier.height(24.dp))
                                Button(
                                    onClick = { importedProfile?.let(onImported) },
                                    modifier = Modifier.fillMaxWidth().height(52.dp),
                                    shape = RoundedCornerShape(14.dp),
                                    colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black)
                                ) {
                                    Text(AppStrings.t(lang, "common.continue"))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
