package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.common.AppStrings
import java.io.InputStream
import java.net.DatagramSocket
import java.net.ServerSocket
import java.net.URLDecoder
import java.nio.charset.StandardCharsets
import kotlin.concurrent.thread

class IntegrationServer(
    private val port: Int = 8585,
    private val authToken: String,
    private val appName: String,
    private val onCredentialsReceived: (String, String) -> Unit,
    private val onNuvioCredentialsReceived: ((String, String) -> Unit)? = null
) {
    private var serverSocket: ServerSocket? = null
    @Volatile private var running = false

    fun start() {
        running = true
        val socket = ServerSocket(port)
        serverSocket = socket
        thread(name = "IntegrationServer", isDaemon = true) {
            while (running) {
                try {
                    val client = socket.accept()
                    thread(isDaemon = true) { handle(client) }
                } catch (_: Exception) {
                    break
                }
            }
        }
    }

    fun stop() {
        running = false
        serverSocket?.close()
        serverSocket = null
    }

    private fun handle(socket: java.net.Socket) {
        try {
            socket.use { s ->
                val stream = s.inputStream
                val requestLine = readLine(stream) ?: return
                val parts = requestLine.split(" ")
                val method = parts.getOrElse(0) { "GET" }
                val rawPath = parts.getOrElse(1) { "/" }
                val qMark = rawPath.indexOf('?')
                val path = if (qMark >= 0) rawPath.substring(0, qMark) else rawPath
                val queryString = if (qMark >= 0) rawPath.substring(qMark + 1) else ""

                Log.i("LocalServer", "Incoming Request: $method $rawPath")

                val headers = mutableMapOf<String, String>()
                while (true) {
                    val line = readLine(stream) ?: break
                    if (line.isEmpty()) break
                    val colon = line.indexOf(':')
                    if (colon > 0) {
                        headers[line.substring(0, colon).trim().lowercase()] = line.substring(colon + 1).trim()
                    }
                }

                val contentLength = headers["content-length"]?.toIntOrNull() ?: 0
                val bodyBytes = ByteArray(contentLength)
                if (contentLength > 0) {
                    var offset = 0
                    while (offset < contentLength) {
                        val read = stream.read(bodyBytes, offset, contentLength - offset)
                        if (read < 0) break
                        offset += read
                    }
                }

                val queryParams = decodeForm(queryString)
                val bodyParams = if (method == "POST") decodeForm(String(bodyBytes, StandardCharsets.UTF_8)) else emptyMap()

                if (queryParams["token"] != authToken) {
                    respond(s, 403, "text/plain", "403 Forbidden")
                    return
                }

                when {
                    method == "GET" -> respond(s, 200, "text/html; charset=UTF-8", getHtmlForm())
                    method == "POST" && path == "/login" -> {
                        val email = bodyParams["email"]
                        val password = bodyParams["password"]
                        val provider = bodyParams["provider"] ?: "stremio"
                        if (!email.isNullOrEmpty() && !password.isNullOrEmpty()) {
                            if (provider == "nuvio" && onNuvioCredentialsReceived != null) {
                                onNuvioCredentialsReceived.invoke(email, password)
                            } else {
                                onCredentialsReceived(email, password)
                            }
                            respond(s, 200, "text/html; charset=UTF-8", getSuccessHtml())
                        } else {
                            respond(s, 400, "text/plain", AppStrings.t("en", "integration.missing_credentials"))
                        }
                    }
                    else -> respond(s, 404, "text/plain", "404 Not Found")
                }
            }
        } catch (e: Exception) {
            Log.e("LocalServer", "Server Error", e)
        }
    }

    private fun readLine(stream: InputStream): String? {
        val buf = StringBuilder()
        var prev = -1
        while (true) {
            val b = stream.read()
            if (b < 0) return if (buf.isEmpty()) null else buf.toString()
            if (b == '\n'.code) {
                if (prev == '\r'.code && buf.isNotEmpty()) buf.deleteCharAt(buf.lastIndex)
                return buf.toString()
            }
            buf.append(b.toChar())
            prev = b
        }
    }

    private fun decodeForm(input: String): Map<String, String> {
        if (input.isBlank()) return emptyMap()
        return input.split("&").associate { pair ->
            val idx = pair.indexOf('=')
            val key = if (idx >= 0) pair.substring(0, idx) else pair
            val value = if (idx >= 0) pair.substring(idx + 1) else ""
            URLDecoder.decode(key, "UTF-8") to URLDecoder.decode(value, "UTF-8")
        }
    }

    private fun respond(socket: java.net.Socket, code: Int, contentType: String, body: String) {
        val bodyBytes = body.toByteArray(StandardCharsets.UTF_8)
        val statusText = when (code) {
            200 -> "OK"; 400 -> "Bad Request"; 403 -> "Forbidden"
            404 -> "Not Found"; else -> "Error"
        }
        val header = "HTTP/1.1 $code $statusText\r\nContent-Type: $contentType\r\nContent-Length: ${bodyBytes.size}\r\nConnection: close\r\n\r\n"
        socket.outputStream.write(header.toByteArray(StandardCharsets.US_ASCII))
        socket.outputStream.write(bodyBytes)
        socket.outputStream.flush()
    }

    private fun getHtmlForm(): String {
        val safeAppName = appName.escapeHtml()
        val nuvioTab = if (onNuvioCredentialsReceived != null) {
            """
                <div class="tabs">
                    <button type="button" class="tab active" id="tabStremio" onclick="showTab('stremio')">${AppStrings.t("en", "integration.tab_stremio")}</button>
                    <button type="button" class="tab" id="tabNuvio" onclick="showTab('nuvio')">${AppStrings.t("en", "integration.tab_nuvio")}</button>
                </div>
            """.trimIndent()
        } else ""
        val nuvioForm = if (onNuvioCredentialsReceived != null) {
            """
                <form id="formNuvio" action="/login?token=$authToken" method="POST" style="display:none">
                    <input type="hidden" name="provider" value="nuvio">
                    <input type="email" name="email" placeholder="${AppStrings.t("en", "integration.nuvio_email")}" required>
                    <input type="password" name="password" placeholder="${AppStrings.t("en", "integration.password")}" required>
                    <button type="submit">${AppStrings.t("en", "integration.sign_in")}</button>
                </form>
            """.trimIndent()
        } else ""
        val script = if (onNuvioCredentialsReceived != null) {
            """
                <script>
                function showTab(name) {
                    document.getElementById('formStremio').style.display = name === 'stremio' ? 'block' : 'none';
                    document.getElementById('formNuvio').style.display = name === 'nuvio' ? 'block' : 'none';
                    document.getElementById('tabStremio').classList.toggle('active', name === 'stremio');
                    document.getElementById('tabNuvio').classList.toggle('active', name === 'nuvio');
                }
                </script>
            """.trimIndent()
        } else ""
        return """
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
                <title>${AppStrings.format("en", "integration.login_page_title", safeAppName)}</title>
                <style>
                    body { font-family: -apple-system, sans-serif; background-color: #0f0f0f; color: white; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; padding: 20px; box-sizing: border-box; }
                    .card { background: #1a1a1a; padding: 2rem; border-radius: 1rem; box-shadow: 0 10px 30px rgba(0,0,0,0.5); width: 100%; max-width: 400px; text-align: center; border: 1px solid #333; }
                    .logo { color: #ffffff; font-size: 2.5rem; font-weight: 900; margin-bottom: 1rem; }
                    input { width: 100%; padding: 12px; margin: 10px 0; border: 1px solid #444; border-radius: 8px; background: #262626; color: white; font-size: 1rem; box-sizing: border-box; }
                    button { width: 100%; padding: 14px; margin-top: 15px; border: none; border-radius: 8px; background: #ffffff; color: black; font-size: 1.1rem; font-weight: bold; cursor: pointer; }
                    p { color: #888; font-size: 0.8rem; margin-top: 1.5rem; }
                    .tabs { display: flex; gap: 8px; margin-bottom: 1rem; }
                    .tab { flex: 1; padding: 10px; border: 1px solid #444; border-radius: 8px; background: #262626; color: #aaa; font-size: 0.9rem; cursor: pointer; margin-top: 0; }
                    .tab.active { background: #ffffff; color: black; font-weight: bold; }
                </style>
            </head>
            <body>
                <div class="card">
                    <div class="logo">${safeAppName.uppercase()}</div>
                    $nuvioTab
                    <form id="formStremio" action="/login?token=$authToken" method="POST">
                        <input type="hidden" name="provider" value="stremio">
                        <input type="email" name="email" placeholder="${AppStrings.t("en", "integration.stremio_email")}" required>
                        <input type="password" name="password" placeholder="${AppStrings.t("en", "integration.password")}" required>
                        <button type="submit">${AppStrings.t("en", "integration.sign_in")}</button>
                    </form>
                    $nuvioForm
                    <p>${AppStrings.t("en", "integration.credentials_notice")}</p>
                </div>
                $script
            </body>
            </html>
        """.trimIndent()
    }

    private fun getSuccessHtml(): String {
        return """
            <html>
            <body style="background: #0f0f0f; color: white; font-family: sans-serif; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; text-align: center;">
                <h1 style="color: #4CAF50;">${AppStrings.t("en", "integration.success_title")}</h1>
                <p>${AppStrings.t("en", "integration.success_message")}</p>
            </body>
            </html>
        """.trimIndent()
    }

    private fun String.escapeHtml(): String {
        return replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;")
    }

    companion object {
        suspend fun getLocalIpAddress(): String? = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            try {
                val socket = DatagramSocket()
                socket.connect(java.net.InetAddress.getByName("8.8.8.8"), 80)
                val ip = socket.localAddress.hostAddress
                socket.close()
                ip ?: "10.0.2.15"
            } catch (e: Exception) {
                Log.e("LocalServer", "IP Error", e)
                "10.0.2.15"
            }
        }
    }
}
