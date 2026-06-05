package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.repository.StremioRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull

internal class HomeAddonCoordinator(
    private val repository: StremioRepository,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val setUserAddons: (List<AddonDescriptor>) -> Unit
) {
    fun refreshInstalledAddons(forceRefresh: Boolean = true) {
        val profile = activeProfile() ?: return
        scope.launch(Dispatchers.IO) {
            val fetched = withTimeoutOrNull(10_000L) {
                repository.getUserAddons(
                    authKey = profile.authKey,
                    localAddons = profile.safeLocalAddons,
                    forceRefresh = forceRefresh
                )
            } ?: return@launch
            setUserAddons(fetched)
        }
    }
}
