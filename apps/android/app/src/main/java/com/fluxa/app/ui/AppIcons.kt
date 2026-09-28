package com.fluxa.app.ui

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager

object AppIcons {
    private const val PREFIX = "com.fluxa.app.ui.icon."

    fun apply(context: Context, id: String) {
        val pm = context.packageManager
        val aliases = pm.getPackageInfo(
            context.packageName,
            PackageManager.GET_ACTIVITIES or PackageManager.MATCH_DISABLED_COMPONENTS,
        ).activities.orEmpty().map { it.name }.filter { it.startsWith(PREFIX) }
        val target = aliases.firstOrNull { it == PREFIX + id } ?: return
        fun set(name: String, state: Int) {
            val component = ComponentName(context.packageName, name)
            if (pm.getComponentEnabledSetting(component) != state) {
                pm.setComponentEnabledSetting(component, state, PackageManager.DONT_KILL_APP)
            }
        }
        set(target, PackageManager.COMPONENT_ENABLED_STATE_ENABLED)
        aliases.filter { it != target }.forEach { set(it, PackageManager.COMPONENT_ENABLED_STATE_DISABLED) }
    }
}
