package com.fluxa.app.data.apple

import com.fluxa.app.data.repository.StremioCatalogParser
import com.fluxa.app.data.repository.StremioStreamParser
import com.fluxa.app.domain.discovery.StremioResourceUrlBuilder
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

data class AppleAddonCatalogExtraSnapshot(
    val name: String? = null,
    val isRequired: Boolean? = null,
    val options: List<String>? = null
)

data class AppleAddonCatalogSnapshot(
    val type: String? = null,
    val id: String? = null,
    val name: String? = null,
    val genres: List<String>? = null,
    val extra: List<AppleAddonCatalogExtraSnapshot>? = null,
    val extraSupported: List<String>? = null,
    val supportsInitialLoad: Boolean = false,
    val supportsSearch: Boolean = false,
    val hasRequiredExtraExceptGenre: Boolean = false
)

data class AppleAddonManifestSnapshot(
    val id: String,
    val name: String,
    val description: String?,
    val logo: String?,
    val version: String?,
    val configurable: Boolean,
    val supportsCatalog: Boolean,
    val catalogs: List<AppleAddonCatalogSnapshot>? = null
)

data class AppleCatalogItemDataSnapshot(
    val id: String,
    val type: String,
    val title: String,
    val subtitle: String,
    val artworkUrl: String?,
    val logoUrl: String?,
    val backgroundUrl: String?,
    val description: String?
)

data class AppleDirectStreamDataSnapshot(
    val title: String?,
    val playableUrl: String?,
    val requestHeaders: Map<String, String>
)

object AppleStremioBridge {
    private val json = Json { ignoreUnknownKeys = true }

    fun normalizeManifestUrl(rawUrl: String): String = StremioResourceUrlBuilder.normalizeManifestUrl(rawUrl)

    fun resourceUrl(transportUrl: String, resource: String, contentType: String, id: String): String =
        StremioResourceUrlBuilder.resourceUrl(transportUrl, resource, contentType, id)

    fun catalogUrl(
        transportUrl: String,
        contentType: String,
        catalogId: String,
        extraName: String?,
        extraValue: String?
    ): String = StremioResourceUrlBuilder.catalogUrl(transportUrl, contentType, catalogId, extraName, extraValue)

    fun parseManifest(body: String): AppleAddonManifestSnapshot? = runCatching {
        val root = json.parseToJsonElement(body).jsonObject
        val resources = root["resources"] as? JsonArray
        val supportsCatalog = resources.orEmpty().any { resource ->
            when (resource) {
                is JsonPrimitive -> resource.contentOrNull == "catalog"
                is JsonObject -> resource["name"]?.jsonPrimitive?.contentOrNull == "catalog"
                else -> false
            }
        }
        val id = root["id"]?.jsonPrimitive?.contentOrNull.orEmpty()
        val name = root["name"]?.jsonPrimitive?.contentOrNull?.takeIf { it.isNotBlank() } ?: id
        AppleAddonManifestSnapshot(
            id = id,
            name = name,
            description = root["description"]?.jsonPrimitive?.contentOrNull,
            logo = root["logo"]?.jsonPrimitive?.contentOrNull,
            version = root["version"]?.jsonPrimitive?.contentOrNull,
            configurable = (root["behaviorHints"] as? JsonObject)?.get("configurable")?.jsonPrimitive?.booleanOrNull ?: false,
            supportsCatalog = supportsCatalog,
            catalogs = (root["catalogs"] as? JsonArray)?.mapNotNull(::parseCatalog)
        )
    }.getOrNull()

    fun parseCatalogItems(body: String, fallbackType: String): List<AppleCatalogItemDataSnapshot>? =
        StremioCatalogParser.parse(body)?.map { item ->
            AppleCatalogItemDataSnapshot(
                id = item.id,
                type = item.type.ifBlank { fallbackType },
                title = item.name,
                subtitle = item.releaseInfo.orEmpty(),
                artworkUrl = item.poster,
                logoUrl = item.logo,
                backgroundUrl = item.background,
                description = item.description
            )
        }

    fun parseDirectStreams(body: String): List<AppleDirectStreamDataSnapshot>? =
        StremioStreamParser.parse(body)?.map { stream ->
            AppleDirectStreamDataSnapshot(stream.title, stream.url, stream.requestHeaders)
        }

    private fun parseCatalog(element: JsonElement): AppleAddonCatalogSnapshot? {
        val catalog = element as? JsonObject ?: return null
        val extras = (catalog["extra"] as? JsonArray)?.mapNotNull { extraElement ->
            val extra = extraElement as? JsonObject ?: return@mapNotNull null
            AppleAddonCatalogExtraSnapshot(
                name = extra["name"]?.jsonPrimitive?.contentOrNull,
                isRequired = extra["isRequired"]?.jsonPrimitive?.booleanOrNull,
                options = extra["options"]?.stringList()
            )
        }
        val extraSupported = catalog["extraSupported"]?.stringList()
        return AppleAddonCatalogSnapshot(
            type = catalog["type"]?.jsonPrimitive?.contentOrNull,
            id = catalog["id"]?.jsonPrimitive?.contentOrNull,
            name = catalog["name"]?.jsonPrimitive?.contentOrNull,
            genres = catalog["genres"]?.stringList(),
            extra = extras,
            extraSupported = extraSupported,
            supportsInitialLoad = extras.orEmpty().none { it.isRequired == true },
            supportsSearch = extraSupported.orEmpty().any { it.equals("search", true) } ||
                extras.orEmpty().any { it.name.equals("search", true) },
            hasRequiredExtraExceptGenre = extras.orEmpty().any {
                it.isRequired == true && !it.name.equals("genre", true)
            }
        )
    }

    private fun JsonElement.stringList(): List<String>? =
        (this as? JsonArray)?.mapNotNull { it.jsonPrimitive.contentOrNull }
}
