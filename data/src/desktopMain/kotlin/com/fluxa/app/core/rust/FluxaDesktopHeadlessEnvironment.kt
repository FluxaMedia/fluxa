package com.fluxa.app.core.rust

import com.fluxa.app.core.rust.effects.fetchAddonCatalogPage
import com.fluxa.app.data.repository.AddonRepository

class FluxaDesktopHeadlessEnvironment(
    private val addonRepository: AddonRepository
) : HeadlessPlatformEnvironment {
    override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        return when (effect.type) {
            "fetchCatalogPage", "fetchDiscoverPage" -> fetchAddonCatalogPage(effect, addonRepository)
            else -> HeadlessEffectCompletion(
                effectId = effect.id,
                status = "error",
                error = mapOf("code" to "unsupported_effect")
            )
        }
    }
}
