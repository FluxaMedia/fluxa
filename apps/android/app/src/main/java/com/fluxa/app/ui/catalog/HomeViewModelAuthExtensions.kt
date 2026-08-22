package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.TraktDeviceCodeResponse
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

internal suspend fun <T> HomeViewModel.fromStateList(value: Any?, type: java.lang.reflect.Type): List<T> {
    if (value == null) return emptyList()
    return withContext(Dispatchers.Default) {
        runCatching { gson.fromJson<List<T>>(gson.toJsonTree(value), type) }.getOrDefault(emptyList())
    }
}

internal suspend fun <T> HomeViewModel.fromStateObject(value: Any?, clazz: Class<T>): T? {
    if (value == null) return null
    return withContext(Dispatchers.Default) {
        runCatching { gson.fromJson(gson.toJsonTree(value), clazz) }
            .onFailure { android.util.Log.e("HomeViewModel", "fromStateObject failed for ${clazz.simpleName}: $value", it) }
            .getOrNull()
    }
}

internal fun HomeViewModel.exchangeTraktCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) =
    authCoordinator.exchangeCode("trakt", code, null, onProfileUpdated, onComplete)

internal fun HomeViewModel.startTraktDeviceAuthorization(onCodeReady: (TraktDeviceCodeResponse) -> Unit, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean, String?) -> Unit) =
    authCoordinator.startTraktDeviceAuthorization(onCodeReady, onProfileUpdated, onComplete)

internal fun HomeViewModel.acceptAnilistToken(accessToken: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) =
    authCoordinator.exchangeCode("anilist", accessToken, null, onProfileUpdated, onComplete)
