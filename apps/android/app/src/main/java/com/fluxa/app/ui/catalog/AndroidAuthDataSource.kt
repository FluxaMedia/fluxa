package com.fluxa.app.ui.catalog

import android.util.Patterns
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.DeviceAuthAdapter
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.auth.JvmAuthDataSource
import java.util.UUID

/** Android wiring for the shared JVM authentication state machine. */
class AndroidAuthDataSource(
    nuvioCoordinator: NuvioAccountImportCoordinator,
    profileManager: ProfileManager,
    language: () -> String,
    onAuthenticated: (UserProfile) -> Unit,
    deviceAuthAdapters: Map<String, DeviceAuthAdapter> = emptyMap(),
    deviceAuthEnabled: Boolean = false,
) : AuthDataSource by JvmAuthDataSource(
    nuvioCoordinator = nuvioCoordinator,
    profileManager = profileManager,
    language = language,
    idGenerator = { UUID.randomUUID().toString() },
    emailValidator = { Patterns.EMAIL_ADDRESS.matcher(it).matches() },
    onAuthenticated = onAuthenticated,
    deviceAuthAdapters = deviceAuthAdapters,
    deviceAuthEnabled = deviceAuthEnabled,
)
