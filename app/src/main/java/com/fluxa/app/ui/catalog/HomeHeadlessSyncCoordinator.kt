package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.Meta
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

internal class HomeHeadlessSyncCoordinator(
    private val scope: CoroutineScope,
    private val gson: Gson,
    private val dispatch: suspend (Any) -> NativeHeadlessEngineResult,
    private val activeProfile: () -> UserProfile?,
    private val setActiveProfile: (UserProfile) -> Unit,
    private val setWatchlist: (List<Meta>) -> Unit,
    private val setContinueWatching: (List<Meta>) -> Unit,
    private val setExternalContinueWatching: (List<Meta>) -> Unit,
    private val setLiked: (List<Meta>) -> Unit,
    private val refreshDynamicRows: () -> Unit
) {
    private val metaListType = object : TypeToken<List<Meta>>() {}.type

    fun loadLibrary(profile: UserProfile?) {
        scope.launch {
            val result = dispatch(mapOf("type" to "libraryHydrateRequested", "profileId" to profile?.id))
            val library = result.state["library"] as? Map<*, *> ?: return@launch
            setWatchlist(decodeList(library["watchlist"]))
            setContinueWatching(decodeList(library["continueWatching"]))
            setLiked(decodeList(library["liked"]))
            refreshDynamicRows()
        }
    }

    fun syncTrakt(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) {
        syncIntegration("trakt", profile, onProfileUpdated, onComplete)
    }

    fun syncStremio(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) {
        syncIntegration("stremio", profile, onProfileUpdated, onComplete)
    }

    fun syncNuvio(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
        onSynced: (UserProfile) -> Unit
    ) {
        syncIntegration("nuvio", profile, onProfileUpdated, onComplete, onSynced)
    }

    private fun syncIntegration(
        provider: String,
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
        onSynced: ((UserProfile) -> Unit)? = null
    ) {
        scope.launch {
            val result = dispatch(
                mapOf(
                    "type" to "externalIntegrationSyncRequested",
                    "provider" to provider,
                    "profile" to profile,
                    "language" to profile.safeLanguage
                )
            )
            val sync = result.state["sync"] as? Map<*, *>
            val snapshot = sync?.get("snapshot") as? Map<*, *>
            val syncResult = decodeProfile(snapshot?.get("profile") ?: (result.state["profile"] as? Map<*, *>)?.get("active"))
            val updated = syncResult?.let { mergeSyncedProfile(gson, base = profile, updated = it, current = activeProfile()) }
            if (updated != null) {
                setActiveProfile(updated)
                setExternalContinueWatching(decodeList((result.state["home"] as? Map<*, *>)?.get("externalContinueWatching")))
                onProfileUpdated(updated)
                refreshDynamicRows()
                onSynced?.invoke(updated)
            }
            onComplete(sync?.get("error") == null && updated != null)
        }
    }

    private suspend fun decodeProfile(value: Any?): UserProfile? = withContext(Dispatchers.Default) {
        if (value == null) null else runCatching { gson.fromJson(gson.toJsonTree(value), UserProfile::class.java) }.getOrNull()
    }

    private suspend fun decodeList(value: Any?): List<Meta> = withContext(Dispatchers.Default) {
        if (value == null) emptyList() else runCatching { gson.fromJson<List<Meta>>(gson.toJsonTree(value), metaListType) }.getOrDefault(emptyList())
    }
}
