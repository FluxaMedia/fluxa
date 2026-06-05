package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.domain.discovery.StreamDiscoveryRequest
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

internal class HomeDirectPlaybackPrefetcher(
    private val repository: StremioRepository,
    private val streamDiscovery: StreamDiscoveryUseCase,
    private val scope: CoroutineScope,
    private val activeProfile: () -> UserProfile?,
    private val userAddons: () -> List<AddonDescriptor>
) {
    fun prefetch(meta: Meta, detail: MetaDetail?) {
        scope.launch(Dispatchers.IO) {
            runCatching {
                val profile = activeProfile()
                val language = profile?.safeLanguage ?: "en"
                val addons = userAddons().ifEmpty {
                    repository.getUserAddons(profile?.authKey ?: "", profile?.safeLocalAddons)
                }
                if (addons.isEmpty()) return@runCatching
                val plan = FluxaCoreNative.directPlaybackPlan(meta, detail, ReleaseDateUtils.todayIso())
                val lookupId = plan.lookupId.ifBlank { detail?.id ?: meta.id }
                if (streamDiscovery.peek(meta.type, lookupId, language).isNullOrEmpty()) {
                    streamDiscovery.prefetch(
                        StreamDiscoveryRequest(
                            addons = addons,
                            type = meta.type,
                            id = lookupId,
                            language = language,
                            preferFastStart = true
                        )
                    )
                }
            }
        }
    }
}
