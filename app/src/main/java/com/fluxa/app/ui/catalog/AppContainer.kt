package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.plugins.PluginManager

object AppContainer {
    lateinit var profileManager: ProfileManager
        private set
    lateinit var pluginManager: PluginManager
        private set
    lateinit var repository: StremioRepository
        private set
    lateinit var authService: StremioService
        private set
    lateinit var nuvioImportCoordinator: NuvioAccountImportCoordinator
        private set

    fun initialize(
        profileManager: ProfileManager,
        pluginManager: PluginManager,
        repository: StremioRepository,
        authService: StremioService,
        nuvioImportCoordinator: NuvioAccountImportCoordinator
    ) {
        this.profileManager = profileManager
        this.pluginManager = pluginManager
        this.repository = repository
        this.authService = authService
        this.nuvioImportCoordinator = nuvioImportCoordinator
    }
}
