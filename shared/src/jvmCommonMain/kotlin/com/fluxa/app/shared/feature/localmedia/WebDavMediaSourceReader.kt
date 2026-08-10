package com.fluxa.app.shared.feature.localmedia

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Credentials
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import org.w3c.dom.Element
import java.io.FilterInputStream
import java.io.InputStream
import java.net.URI
import java.net.URLDecoder
import java.nio.charset.StandardCharsets
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.TimeZone
import javax.xml.parsers.DocumentBuilderFactory

class WebDavMediaSourceReader(
    private val client: OkHttpClient = OkHttpClient.Builder().followRedirects(true).build(),
) : LocalMediaSourceReader {
    override fun supports(type: LocalMediaSourceType): Boolean = type == LocalMediaSourceType.WebDav

    override suspend fun listFiles(source: LocalMediaSourceConfig): List<LocalMediaFileCandidate> = withContext(Dispatchers.IO) {
        val root = normalizeRoot(source.location)
        val rootUri = runCatching { URI(root) }.getOrElse { error("Invalid WebDAV URL") }
        require(rootUri.scheme.equals("http", true) || rootUri.scheme.equals("https", true)) {
            "WebDAV URL must use http:// or https://"
        }
        val out = ArrayList<LocalMediaFileCandidate>()
        val seen = HashSet<String>()
        walk(source, rootUri, rootUri, out, seen, 0)
        out
    }

    private fun walk(
        source: LocalMediaSourceConfig,
        root: URI,
        directory: URI,
        out: MutableList<LocalMediaFileCandidate>,
        seen: MutableSet<String>,
        depth: Int,
    ) {
        if (depth > 24 || out.size >= 50_000 || !seen.add(directory.normalize().toString())) return
        val request = requestBuilder(source, directory.toString())
            .method(
                "PROPFIND",
                """<?xml version="1.0" encoding="utf-8"?><propfind xmlns="DAV:"><prop><resourcetype/><getcontentlength/><getlastmodified/><displayname/></prop></propfind>"""
                    .toRequestBody("application/xml; charset=utf-8".toMediaType())
            )
            .header("Depth", "1")
            .build()
        client.newCall(request).execute().use { response ->
            check(response.isSuccessful) { "WebDAV PROPFIND failed with HTTP ${response.code}" }
            val bytes = response.body?.bytes() ?: return
            val document = runCatching {
                secureDocumentBuilderFactory().newDocumentBuilder().parse(bytes.inputStream())
            }.getOrNull() ?: return
            val responses = document.getElementsByTagNameNS("DAV:", "response")
            for (i in 0 until responses.length) {
                val element = responses.item(i) as? Element ?: continue
                val href = element.getElementsByTagNameNS("DAV:", "href").item(0)?.textContent?.trim().orEmpty()
                if (href.isBlank()) continue
                val resolved = directory.resolve(href).normalize()
                if (resolved == directory.normalize()) continue
                val resourceType = element.getElementsByTagNameNS("DAV:", "resourcetype").item(0) as? Element
                val isCollection = resourceType?.getElementsByTagNameNS("DAV:", "collection")?.length?.let { it > 0 } == true
                val displayName = element.getElementsByTagNameNS("DAV:", "displayname").item(0)?.textContent
                    ?.takeIf { it.isNotBlank() }
                    ?: URLDecoder.decode(resolved.path.substringAfterLast('/').ifBlank { resolved.path.trim('/').substringAfterLast('/') }, "UTF-8")
                if (isCollection) {
                    walk(source, root, ensureDirectoryUri(resolved), out, seen, depth + 1)
                } else if (LocalMediaFilenameParser.isVideoFile(displayName)) {
                    val relative = resolved.path.removePrefix(root.path).trim('/')
                    val parentHints = relative.split('/').dropLast(1).asReversed().take(4)
                    val length = element.getElementsByTagNameNS("DAV:", "getcontentlength").item(0)?.textContent?.trim()?.toLongOrNull() ?: 0L
                    val modified = parseHttpDate(element.getElementsByTagNameNS("DAV:", "getlastmodified").item(0)?.textContent)
                    out += LocalMediaFileCandidate(
                        locator = resolved.toString(),
                        displayName = displayName,
                        parentHints = parentHints,
                        sizeBytes = length,
                        modifiedAtMs = modified,
                    )
                }
            }
        }
    }

    override fun open(source: LocalMediaSourceConfig, locator: String, offset: Long): LocalMediaOpenedStream {
        val request = requestBuilder(source, locator)
            .apply { if (offset > 0L) header("Range", "bytes=$offset-") }
            .get()
            .build()
        val response = client.newCall(request).execute()
        if (!response.isSuccessful && response.code != 206) {
            response.close()
            error("WebDAV GET failed with HTTP ${response.code}")
        }
        val body = response.body ?: run {
            response.close()
            error("WebDAV response has no body")
        }
        val total = contentRangeTotal(response) ?: body.contentLength().let { length ->
            if (length >= 0L) length + offset else 0L
        }
        val input = object : FilterInputStream(body.byteStream()) {
            override fun close() {
                try { super.close() } finally { response.close() }
            }
        }
        return LocalMediaOpenedStream(input, total, response.header("Content-Type") ?: localMediaContentType(locator))
    }


    private fun secureDocumentBuilderFactory(): DocumentBuilderFactory = DocumentBuilderFactory.newInstance().apply {
        isNamespaceAware = true
        isXIncludeAware = false
        isExpandEntityReferences = false
        // WebDAV XML is remote/untrusted input. Disable DTDs and external entities to
        // avoid XXE/file disclosure even when a NAS returns a malicious response.
        runCatching { setFeature("http://apache.org/xml/features/disallow-doctype-decl", true) }
        runCatching { setFeature("http://xml.org/sax/features/external-general-entities", false) }
        runCatching { setFeature("http://xml.org/sax/features/external-parameter-entities", false) }
        runCatching { setFeature("http://apache.org/xml/features/nonvalidating/load-external-dtd", false) }
    }

    private fun requestBuilder(source: LocalMediaSourceConfig, url: String): Request.Builder {
        val builder = Request.Builder().url(url)
        if (!source.username.isNullOrBlank()) {
            builder.header("Authorization", Credentials.basic(source.username.orEmpty(), source.password.orEmpty()))
        }
        return builder
    }

    private fun contentRangeTotal(response: Response): Long? = response.header("Content-Range")
        ?.substringAfter('/')
        ?.takeIf { it != "*" }
        ?.toLongOrNull()

    private fun normalizeRoot(value: String): String = if (value.endsWith('/')) value else "$value/"
    private fun ensureDirectoryUri(uri: URI): URI = if (uri.toString().endsWith('/')) uri else URI("${uri}/")

    private fun parseHttpDate(value: String?): Long {
        if (value.isNullOrBlank()) return 0L
        val formatter = SimpleDateFormat("EEE, dd MMM yyyy HH:mm:ss zzz", Locale.US).apply { timeZone = TimeZone.getTimeZone("GMT") }
        return runCatching { formatter.parse(value)?.time ?: 0L }.getOrDefault(0L)
    }
}
