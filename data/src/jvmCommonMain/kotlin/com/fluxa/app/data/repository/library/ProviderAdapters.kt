package com.fluxa.app.data.repository.library

import javax.inject.Inject

class ProviderAdapters @Inject constructor(
    val trakt: TraktProviderAdapter,
    val simkl: SimklProviderAdapter,
    val anilist: AnilistProviderAdapter,
    val nuvio: NuvioProviderAdapter,
    val stremio: StremioProviderAdapter
) {
    val all: List<ProviderAdapter> = listOf(trakt, simkl, anilist, nuvio, stremio)

    fun byId(id: String?): ProviderAdapter? = all.firstOrNull { it.id == id }
}
