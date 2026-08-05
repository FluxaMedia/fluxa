package com.fluxa.app.core.rust.effects

import com.fluxa.app.core.rust.HeadlessEffectCompletion
import com.fluxa.app.core.rust.NativeHeadlessEffect
import com.fluxa.app.core.rust.number
import com.fluxa.app.core.rust.string
import com.fluxa.app.core.rust.stringOrNull
import com.fluxa.app.data.repository.AddonRepository

suspend fun fetchAddonCatalogPage(
    effect: NativeHeadlessEffect,
    addonRepository: AddonRepository
): HeadlessEffectCompletion {
    val payload = effect.payload
    val items = addonRepository.getAddonCatalog(
        transportUrl = payload.string("transportUrl"),
        type = payload.string("contentType"),
        id = payload.string("catalogId"),
        skip = payload.number("skip")?.toInt() ?: 0,
        genre = payload.stringOrNull("genre"),
        search = payload.stringOrNull("search")
    )
    return HeadlessEffectCompletion(effectId = effect.id, status = "ok", value = mapOf("items" to items))
}
