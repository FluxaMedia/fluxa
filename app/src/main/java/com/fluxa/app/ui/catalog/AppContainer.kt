package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.remote.StremioService
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.PluginRepositoryManager

object AppContainer {
    lateinit var profileManager: ProfileManager
        private set
    lateinit var pluginManager: PluginManager
        private set
    lateinit var pluginRepositoryManager: PluginRepositoryManager
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
        pluginRepositoryManager: PluginRepositoryManager,
        repository: StremioRepository,
        authService: StremioService,
        nuvioImportCoordinator: NuvioAccountImportCoordinator
    ) {
        this.profileManager = profileManager
        this.pluginManager = pluginManager
        this.pluginRepositoryManager = pluginRepositoryManager
        this.repository = repository
        this.authService = authService
        this.nuvioImportCoordinator = nuvioImportCoordinator
    }
}
