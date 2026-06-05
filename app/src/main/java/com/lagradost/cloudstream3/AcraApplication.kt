package com.lagradost.cloudstream3

import android.annotation.SuppressLint
import android.app.Activity
import android.app.Application
import android.content.Context
import java.lang.ref.WeakReference

class AcraApplication : Application() {
    companion object {
        @SuppressLint("StaticFieldLeak")
        var context: Context? = null
            private set

        private var activityRef: WeakReference<Activity>? = null

        fun init(app: Application) {
            context = app.applicationContext
            CloudStreamApp.context = app.applicationContext
        }

        fun getActivity(): Activity? = activityRef?.get()

        fun setActivity(activity: Activity?) {
            activityRef = activity?.let(::WeakReference)
        }
    }
}
