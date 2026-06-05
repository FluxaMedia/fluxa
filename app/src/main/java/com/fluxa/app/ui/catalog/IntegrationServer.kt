package com.fluxa.app.ui.catalog

import android.util.Log
import fi.iki.elonen.NanoHTTPD
import java.net.DatagramSocket
import java.net.URLDecoder
import java.nio.charset.StandardCharsets

class IntegrationServer(
    private val port: Int = 8585,
    private val authToken: String,
    private val appName: String,
    private val onCredentialsReceived: (String, String) -> Unit
) : NanoHTTPD(port) {

    override fun serve(session: IHTTPSession): Response {
        val uri = session.uri ?: "/"
        val method = session.method
        Log.i("LocalServer", "Incoming Request: $method $uri")

        return try {
            if (!isAuthorized(session)) {
                newFixedLengthResponse(Response.Status.FORBIDDEN, "text/plain", "403 Forbidden")
            } else if (method == Method.GET) {
                newFixedLengthResponse(Response.Status.OK, "text/html; charset=UTF-8", getHtmlForm())
            } else if (method == Method.POST && uri == "/login") {
                val files = HashMap<String, String>()
                session.parseBody(files)
                val email = session.parameters["email"]?.firstOrNull()
                val password = session.parameters["password"]?.firstOrNull()

                if (!email.isNullOrEmpty() && !password.isNullOrEmpty()) {
                    onCredentialsReceived(email, password)
                    newFixedLengthResponse(Response.Status.OK, "text/html; charset=UTF-8", getSuccessHtml())
                } else {
                    newFixedLengthResponse(Response.Status.BAD_REQUEST, "text/plain", AppStrings.t("en", "integration.missing_credentials"))
                }
            } else {
                newFixedLengthResponse(Response.Status.NOT_FOUND, "text/plain", "404 Not Found")
            }
        } catch (e: Exception) {
            Log.e("LocalServer", "Server Error", e)
            newFixedLengthResponse(Response.Status.INTERNAL_ERROR, "text/plain", "Server Error: ${e.message}")
        }
    }

    private fun isAuthorized(session: IHTTPSession): Boolean {
        val tokenFromParams = session.parameters["token"]?.firstOrNull()
        if (tokenFromParams == authToken) return true
        val query = session.queryParameterString ?: return false
        return query.split("&").any { pair ->
            val parts = pair.split("=", limit = 2)
            parts.firstOrNull() == "token" &&
                parts.getOrNull(1)?.let { URLDecoder.decode(it, StandardCharsets.UTF_8.name()) } == authToken
        }
    }

    private fun getHtmlForm(): String {
        val safeAppName = appName.escapeHtml()
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
                </style>
            </head>
            <body>
                <div class="card">
                    <div class="logo">${safeAppName.uppercase()}</div>
                    <form action="/login?token=$authToken" method="POST">
                        <input type="email" name="email" placeholder="${AppStrings.t("en", "integration.stremio_email")}" required>
                        <input type="password" name="password" placeholder="${AppStrings.t("en", "integration.password")}" required>
                        <button type="submit">${AppStrings.t("en", "integration.sign_in")}</button>
                    </form>
                    <p>${AppStrings.t("en", "integration.credentials_notice")}</p>
                </div>
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
