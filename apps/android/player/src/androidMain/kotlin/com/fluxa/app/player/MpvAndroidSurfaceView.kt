package com.fluxa.app.player

import android.content.Context
import android.util.AttributeSet
import android.util.Log
import android.view.SurfaceHolder
import android.view.SurfaceView

class MpvAndroidSurfaceView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : SurfaceView(context, attrs), SurfaceHolder.Callback {
    private var player: MpvEmbeddedPlayer? = null

    init {
        holder.addCallback(this)
    }

    fun bind(player: MpvEmbeddedPlayer?) {
        if (this.player === player) return
        this.player?.onSurfaceDestroyedByView()
        this.player = player
        val s = holder.surface
        if (s != null && s.isValid) {
            player?.onSurfaceCreatedByView(s, width, height)
        }
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        Log.d(TAG, "surface created ${width}x${height}")
        player?.onSurfaceCreatedByView(holder.surface, width, height)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        Log.d(TAG, "surface changed ${width}x${height}")
        player?.onSurfaceChangedByView(holder.surface, width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        Log.d(TAG, "surface destroyed")
        player?.onSurfaceDestroyedByView()
    }

    override fun onDetachedFromWindow() {
        player?.onSurfaceDestroyedByView()
        super.onDetachedFromWindow()
    }

    private companion object {
        const val TAG = "MpvAndroidSurfaceView"
    }
}
