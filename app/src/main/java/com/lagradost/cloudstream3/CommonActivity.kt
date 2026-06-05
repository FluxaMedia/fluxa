package com.lagradost.cloudstream3

import android.app.Activity
import android.os.Handler
import android.os.Looper
import android.widget.Toast
import java.lang.ref.WeakReference

object CommonActivity {
    private var activityRef: WeakReference<Activity>? = null

    var activity: Activity?
        get() = activityRef?.get()
        set(value) {
            activityRef = value?.let(::WeakReference)
            AcraApplication.setActivity(value)
        }

    // ── showToast overloads that plugins commonly call ────────────────────────

    @JvmStatic
    fun showToast(message: String?, duration: Int? = null) {
        val ctx = activity ?: AcraApplication.context ?: return
        val text = message ?: return
        val d = duration ?: Toast.LENGTH_SHORT
        Handler(Looper.getMainLooper()).post {
            Toast.makeText(ctx, text, d).show()
        }
    }

    @JvmStatic
    fun showToast(act: Activity?, message: String?, duration: Int? = null) {
        val ctx = act ?: activity ?: AcraApplication.context ?: return
        val text = message ?: return
        val d = duration ?: Toast.LENGTH_SHORT
        Handler(Looper.getMainLooper()).post {
            Toast.makeText(ctx, text, d).show()
        }
    }
}
