package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.data.remote.NuvioAddon
import com.google.gson.Gson
import com.google.gson.JsonObject

object NuvioImportPolicy {
    private val gson = Gson()

    fun addonState(addons: List<NuvioAddon>): NuvioAddonState {
        val args = JsonObject().apply { add("addons", gson.toJsonTree(addons)) }
        val value = FluxaCoreUniFfi.coreInvokeValue("nuvioAddonState", args.toString())
        return gson.fromJson(value, NuvioAddonState::class.java)
    }
}
