package com.fluxa.app.shared.feature.localmedia

import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.net.URLDecoder
import java.nio.charset.StandardCharsets
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

internal class LocalMediaHttpGateway(
    private val opener: (String, Long) -> LocalMediaOpenedStream?,
) : AutoCloseable {
    private val running = AtomicBoolean(true)
    private val server = ServerSocket(0, 32, InetAddress.getByName("127.0.0.1"))
    private val executor = Executors.newCachedThreadPool { runnable ->
        Thread(runnable, "fluxa-local-media-http").apply { isDaemon = true }
    }
    val port: Int get() = server.localPort

    init {
        executor.execute {
            while (running.get()) {
                val socket = runCatching { server.accept() }.getOrNull() ?: break
                executor.execute { handle(socket) }
            }
        }
    }

    fun urlFor(fileId: String): String = "http://127.0.0.1:$port/media/$fileId"

    private fun handle(socket: Socket) {
        socket.use { client ->
            client.soTimeout = 30_000
            val input = BufferedInputStream(client.getInputStream())
            val output = BufferedOutputStream(client.getOutputStream())
            val requestLine = readAsciiLine(input) ?: return
            val parts = requestLine.split(' ')
            if (parts.size < 2) return
            val method = parts[0].uppercase()
            if (method != "GET" && method != "HEAD") {
                writeError(output, 405, "Method Not Allowed")
                return
            }
            val path = parts[1].substringBefore('?')
            val fileId = URLDecoder.decode(path.substringAfter("/media/", ""), "UTF-8")
            if (fileId.isBlank()) {
                writeError(output, 404, "Not Found")
                return
            }
            var rangeHeader: String? = null
            while (true) {
                val line = readAsciiLine(input) ?: break
                if (line.isEmpty()) break
                val separator = line.indexOf(':')
                if (separator > 0 && line.substring(0, separator).trim().equals("Range", ignoreCase = true)) {
                    rangeHeader = line.substring(separator + 1).trim()
                }
            }
            val requested = parseRange(rangeHeader)
            val start = requested?.first ?: 0L
            val opened = runCatching { opener(fileId, start) }.getOrNull()
            if (opened == null) {
                writeError(output, 404, "Not Found")
                return
            }
            opened.input.use { media ->
                val total = opened.totalLength.coerceAtLeast(0L)
                if (total > 0L && start >= total) {
                    output.write("HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */$total\r\nConnection: close\r\n\r\n".toByteArray())
                    output.flush()
                    return
                }
                val requestedEnd = requested?.second
                val end = when {
                    total <= 0L -> requestedEnd
                    requestedEnd == null -> total - 1
                    else -> requestedEnd.coerceAtMost(total - 1)
                }
                val length = when {
                    total <= 0L -> -1L
                    end == null -> total - start
                    else -> (end - start + 1).coerceAtLeast(0L)
                }
                val partial = rangeHeader != null && total > 0L
                output.write(buildString {
                    append(if (partial) "HTTP/1.1 206 Partial Content\r\n" else "HTTP/1.1 200 OK\r\n")
                    append("Content-Type: ${opened.contentType}\r\n")
                    append("Accept-Ranges: bytes\r\n")
                    if (partial && end != null) append("Content-Range: bytes $start-$end/$total\r\n")
                    if (length >= 0L) append("Content-Length: $length\r\n")
                    append("Cache-Control: no-store\r\n")
                    append("Connection: close\r\n\r\n")
                }.toByteArray(StandardCharsets.US_ASCII))
                output.flush()
                if (method == "HEAD") return
                copyLimited(media, output, length)
                output.flush()
            }
        }
    }

    private fun copyLimited(input: java.io.InputStream, output: java.io.OutputStream, length: Long) {
        val buffer = ByteArray(128 * 1024)
        var remaining = length
        while (running.get() && (remaining != 0L)) {
            val max = if (remaining < 0L) buffer.size else minOf(buffer.size.toLong(), remaining).toInt()
            val read = input.read(buffer, 0, max)
            if (read <= 0) break
            output.write(buffer, 0, read)
            if (remaining > 0L) remaining -= read
        }
    }

    private fun parseRange(value: String?): Pair<Long, Long?>? {
        val raw = value?.trim()?.takeIf { it.startsWith("bytes=", ignoreCase = true) }?.substringAfter('=') ?: return null
        val first = raw.substringBefore(',').trim()
        val dash = first.indexOf('-')
        if (dash <= 0) return null
        val start = first.substring(0, dash).toLongOrNull()?.coerceAtLeast(0L) ?: return null
        val end = first.substring(dash + 1).toLongOrNull()?.takeIf { it >= start }
        return start to end
    }

    private fun readAsciiLine(input: java.io.InputStream): String? {
        val bytes = java.io.ByteArrayOutputStream()
        while (bytes.size() < 16_384) {
            val b = input.read()
            if (b < 0) return if (bytes.size() == 0) null else bytes.toString(StandardCharsets.US_ASCII)
            if (b == '\n'.code) break
            if (b != '\r'.code) bytes.write(b)
        }
        return bytes.toString(StandardCharsets.US_ASCII)
    }

    private fun writeError(output: java.io.OutputStream, code: Int, text: String) {
        val body = "$code $text"
        output.write("HTTP/1.1 $code $text\r\nContent-Type: text/plain\r\nContent-Length: ${body.length}\r\nConnection: close\r\n\r\n$body".toByteArray(StandardCharsets.US_ASCII))
        output.flush()
    }

    override fun close() {
        if (!running.compareAndSet(true, false)) return
        runCatching { server.close() }
        executor.shutdownNow()
    }
}
