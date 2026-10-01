package com.fluxa.app.ui

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

class TvLauncherReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val pending = goAsync()
        Thread {
            TvLauncher.initialize(context.applicationContext)
            pending.finish()
        }.start()
    }
}
