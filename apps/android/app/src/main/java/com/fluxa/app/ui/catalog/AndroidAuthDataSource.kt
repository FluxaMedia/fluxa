package com.fluxa.app.ui.catalog

import android.util.Patterns
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.plugins.PluginRepositoryManager
import com.fluxa.app.shared.feature.auth.AuthDataSource
import com.fluxa.app.shared.feature.auth.JvmAuthDataSource
import java.util.UUID

/** Android wiring for the shared JVM authentication state machine. */
class AndroidAuthDataSource(
    authService: StremioService,
    nuvioCoordinator: NuvioAccountImportCoordinator,
    pluginRepositoryManager: PluginRepositoryManager,
    profileManager: ProfileManager,
    language: () -> String,
    onAuthenticated: (UserProfile) -> Unit,
) : AuthDataSource by JvmAuthDataSource(
    authService = authService,
    nuvioCoordinator = nuvioCoordinator,
    profileManager = profileManager,
    language = language,
    idGenerator = { UUID.randomUUID().toString() },
    emailValidator = { Patterns.EMAIL_ADDRESS.matcher(it).matches() },
    onPluginsImported = pluginRepositoryManager::syncNuvioPlugins,
    onAuthenticated = onAuthenticated,
)
