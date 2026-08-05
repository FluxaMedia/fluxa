package com.fluxa.app.desktop.addonstore

import com.fluxa.app.desktop.home.CINEMETA_TRANSPORT_URL
import java.util.prefs.Preferences

class DesktopAddonRegistry {
    private val prefs: Preferences = Preferences.userRoot().node("com/fluxa/app/desktop/addons")

    fun installedUrls(): List<String> {
        val raw = prefs.get(K.urls, null)
        if (raw == null) {
            prefs.put(K.urls, CINEMETA_TRANSPORT_URL)
            return listOf(CINEMETA_TRANSPORT_URL)
        }
        return raw.split(URL_DELIMITER).filter { it.isNotBlank() }
    }

    fun disabledUrls(): Set<String> =
        prefs.get(K.disabled, "").split(URL_DELIMITER).filter { it.isNotBlank() }.toSet()

    fun enabledUrls(): List<String> {
        val disabled = disabledUrls()
        return installedUrls().filterNot { it in disabled }
    }

    fun addUrl(url: String) {
        val current = installedUrls()
        if (url !in current) {
            prefs.put(K.urls, (current + url).joinToString(URL_DELIMITER))
        }
    }

    fun removeUrl(url: String) {
        prefs.put(K.urls, installedUrls().filterNot { it == url }.joinToString(URL_DELIMITER))
        prefs.put(K.disabled, disabledUrls().minus(url).joinToString(URL_DELIMITER))
    }

    fun setEnabled(url: String, enabled: Boolean) {
        val disabled = disabledUrls().toMutableSet()
        if (enabled) disabled.remove(url) else disabled.add(url)
        prefs.put(K.disabled, disabled.joinToString(URL_DELIMITER))
    }

    fun moveUrl(url: String, direction: Int) {
        val current = installedUrls().toMutableList()
        val index = current.indexOf(url)
        val target = index + direction
        if (index < 0 || target !in current.indices) return
        val tmp = current[target]
        current[target] = current[index]
        current[index] = tmp
        prefs.put(K.urls, current.joinToString(URL_DELIMITER))
    }

    private object K {
        const val urls = "installedUrls"
        const val disabled = "disabledUrls"
    }

    private companion object {
        const val URL_DELIMITER = "\n"
    }
}
