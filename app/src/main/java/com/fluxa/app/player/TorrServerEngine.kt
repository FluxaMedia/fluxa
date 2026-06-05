package com.fluxa.app.player

import android.content.Context
import android.util.Log
import com.fluxa.app.core.rust.FluxaStreamingNative
import java.io.IOException
import java.net.ServerSocket
import kotlinx.coroutines.*

class TorrServerEngine(private val context: Context) {
    private val port = 8090
    private var watcherJob: Job? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    @Volatile private var running = false

    fun start() {
        if (isRunning()) return

        ensurePortFree()
        // Clear stale cache so rqbit doesn't spend time hash-checking old pieces on startup
        val dataDir = java.io.File(context.filesDir, "rust_torrent_cache").apply {
            deleteRecursively()
            mkdirs()
        }

        try {
            val result = FluxaStreamingNative.startTorrentServer(dataDir.absolutePath, port)
            running = result.isNotBlank()
            Log.i("TorrServer", "Rust torrent engine started on port $port")
            startWatcher()
        } catch (e: IOException) {
            Log.e("TorrServer", "Failed to start engine", e)
            running = false
        } catch (e: Exception) {
            Log.e("TorrServer", "Failed to start Rust torrent engine", e)
            running = false
        }
    }
    
    private fun startWatcher() {
        watcherJob?.cancel()
        watcherJob = scope.launch {
            while (isActive) {
                delay(3000)
                if (!isRunning()) {
                    Log.w("TorrServer", "Rust torrent engine stopped. Restarting...")
                    running = false
                    start()
                }
            }
        }
    }

    fun stop() {
        watcherJob?.cancel()
        Log.i("TorrServer", "Stopping Rust torrent engine...")
        try {
            FluxaStreamingNative.stopTorrentServer()
            running = false
        } catch (e: Exception) {
            Log.e("TorrServer", "Rust torrent engine stop failed", e)
        }
        
        scope.launch {
            delay(500)
            ensurePortFree()
        }
    }

    fun isRunning(): Boolean {
        return running
    }

    private fun ensurePortFree() {
        try {
            ServerSocket(port).use { /* Port is free */ }
        } catch (e: IOException) {
            Log.w("TorrServer", "Port $port is busy. Engine might be already running or zombie process exists.")
            // Instead of lsof, we just hope destroyForcibly worked or the new process can take over
        }
    }
}
