package com.fluxa.app.data.remote

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

@Serializable
data class MetaRating(val source: String, val value: String?)

@Serializable
data class AppExtras(
    val seasonPosters: List<String?>? = null,
    val certification: String? = null,
    val certificationLocal: String? = null,
    val cast: List<CastMember>? = null
)

@Serializable
data class MetaLink(val name: String, val category: String, val url: String)

@Serializable
data class Video(
    val id: String,
    @SerialName("name") val name: String? = null,
    val season: Int? = null,
    @SerialName("number") val number: Int? = null,
    @SerialName("released") val released: String? = null,
    val thumbnail: String? = null,
    @SerialName("overview") val overview: String? = null,
    val rating: String? = null,
    val episodeRuntime: Int? = null
)

@Serializable(with = CastMemberSerializer::class)
data class CastMember(val name: String, val character: String?, val profilePath: String?)

@Serializable(with = DetailTrailerSerializer::class)
data class DetailTrailer(
    val id: String,
    val title: String,
    val type: String,
    val url: String,
    val thumbnail: String? = null,
    val source: String
)

@Serializable
data class MetaDetailResponse(val meta: MetaDetail? = null)

@Serializable(with = MetaDetailSerializer::class)
data class MetaDetail(
    val id: String,
    val type: String,
    val name: String,
    val genres: List<String>?,
    val poster: String?,
    val background: String?,
    val logo: String?,
    val description: String?,
    val releaseInfo: String?,
    val released: String? = null,
    val runtime: String?,
    val videos: List<Video>?,
    val trailers: List<DetailTrailer>? = null,
    val imdbRating: String? = null,
    val ageRating: String? = null,
    val ratings: List<MetaRating>? = null,
    val cast: List<CastMember>? = null,
    val director: List<String>? = null,
    val links: List<MetaLink>? = null,
    val status: String? = null,
    val seasonsCount: Int? = null,
    val platforms: List<String>? = null,
    val awards: String? = null,
    val originalLanguage: String? = null,
    val originalName: String? = null,
    val country: String? = null,
    val productionCompanies: List<String>? = null,
    val networks: List<String>? = null,
    val collectionName: String? = null,
    val collectionId: Int? = null,
    val collectionParts: List<Meta>? = null,
    val seasonPosters: Map<String, String>? = null,
    val appExtras: AppExtras? = null
)

@Serializable
data class Meta(
    val id: String,
    val name: String,
    val type: String,
    val poster: String?,
    val background: String? = null,
    val logo: String? = null,
    val description: String? = null,
    val imdbRating: String? = null,
    val ageRating: String? = null,
    val ratings: List<MetaRating>? = null,
    val genres: List<String>? = null,
    val releaseInfo: String? = null,
    val released: String? = null,
    val runtime: String? = null,
    val seasonsCount: Int? = null,
    val episodesCount: Int? = null,
    val cast: List<CastMember>? = null,
    val timeOffset: Long? = null,
    val duration: Long? = null,
    val lastVideoId: String? = null,
    val lastStreamIndex: Int? = null,
    val lastEpisodeName: String? = null,
    val lastStreamUrl: String? = null,
    val lastStreamTitle: String? = null,
    val lastBingeGroup: String? = null,
    val lastAudioLanguage: String? = null,
    val lastSubtitleLanguage: String? = null,
    val awards: String? = null,
    val rank: Int? = null,
    val reason: String? = null,
    val homeBadge: String? = null,
    val originalLanguage: String? = null,
    val originalName: String? = null,
    val continueWatchingPoster: String? = null,
    val continueWatchingBackground: String? = null,
    val focusGifUrl: String? = null,
    val coverEmoji: String? = null,
    val hideTitle: Boolean? = null,
    val focusGlowEnabled: Boolean? = null,
    @SerialName("videos") val videos: List<Video>? = null,
    val trailers: List<DetailTrailer>? = null,
    val seasonPosters: Map<String, String>? = null
)

private fun JsonElement?.safeString(): String? {
    val element = this?.takeUnless { it is JsonNull } ?: return null
    return (element as? JsonPrimitive)?.takeIf { it.isString || it.contentOrNull != null }?.content
}

private fun JsonElement?.asObjectOrNull(): JsonObject? {
    val element = this?.takeUnless { it is JsonNull } ?: return null
    return element as? JsonObject
}

private fun JsonObject.first(vararg keys: String): JsonElement? =
    keys.firstNotNullOfOrNull { key -> get(key)?.takeUnless { it is JsonNull } }

private fun JsonObject.text(vararg keys: String): String? =
    first(*keys)?.safeString()?.trim()?.takeIf { it.isNotBlank() }

private fun JsonObject.int(vararg keys: String): Int? {
    val value = first(*keys)?.safeString()?.trim()?.takeIf { it.isNotBlank() } ?: return null
    return value.toIntOrNull() ?: value.toDoubleOrNull()?.toInt()
}

private fun JsonObject.stringList(vararg keys: String): List<String>? {
    val value = first(*keys) ?: return null
    return when (value) {
        is JsonArray -> value
            .mapNotNull { it.safeString()?.trim()?.takeIf(String::isNotBlank) }
            .takeIf { it.isNotEmpty() }
        else -> value.safeString()
            ?.split(',')
            ?.mapNotNull { it.trim().takeIf(String::isNotBlank) }
            ?.takeIf { it.isNotEmpty() }
    }
}

private fun JsonObject.stringMap(vararg keys: String): Map<String, String>? {
    val value = first(*keys) ?: return null
    return when (value) {
        is JsonObject -> value.entries
            .mapNotNull { entry -> entry.value.safeString()?.takeIf(String::isNotBlank)?.let { entry.key to it } }
            .toMap()
            .takeIf { it.isNotEmpty() }
        is JsonArray -> value
            .mapIndexedNotNull { index, item -> item.safeString()?.takeIf(String::isNotBlank)?.let { index.toString() to it } }
            .toMap()
            .takeIf { it.isNotEmpty() }
        else -> null
    }
}

private fun JsonObject.objectList(vararg keys: String): List<JsonObject> {
    val value = first(*keys) ?: return emptyList()
    return when (value) {
        is JsonArray -> value.mapNotNull { it.asObjectOrNull() }
        is JsonObject -> listOf(value)
        else -> emptyList()
    }
}

private fun JsonObject.videoList(vararg keys: String): List<Video>? =
    objectList(*keys).mapNotNull { obj ->
        val id = obj.text("id") ?: return@mapNotNull null
        Video(
            id = id,
            name = obj.text("name", "title"),
            season = obj.int("season"),
            number = obj.int("number", "episode"),
            released = obj.text("released", "firstAired"),
            thumbnail = obj.text("thumbnail"),
            overview = obj.text("overview", "description"),
            rating = obj.text("rating"),
            episodeRuntime = obj.int("episodeRuntime", "runtime")
        )
    }.takeIf { it.isNotEmpty() }

private fun detailTrailerFromJson(json: JsonElement): DetailTrailer {
    val obj = json.asObjectOrNull() ?: buildJsonObject {
        put("source", json.safeString())
    }
    fun text(vararg keys: String): String? = keys.firstNotNullOfOrNull { key ->
        obj[key]?.safeString()?.trim()?.takeIf { it.isNotBlank() }
    }
    val rawSource = text("source")
    val youtubeId = text("ytId") ?: rawSource?.takeUnless { it.startsWith("http://") || it.startsWith("https://") }
    val url = text("externalUrl", "url")
        ?: youtubeId?.let { "https://www.youtube.com/watch?v=$it" }
        ?: rawSource.orEmpty()
    val trailerType = text("type") ?: "Trailer"
    return DetailTrailer(
        id = text("id") ?: youtubeId ?: url,
        title = text("name", "title", "description") ?: trailerType,
        type = trailerType,
        url = url,
        thumbnail = text("thumbnail") ?: youtubeId?.let { "https://i.ytimg.com/vi/$it/hqdefault.jpg" },
        source = text("provider") ?: "addon"
    )
}

private fun JsonObject.trailerList(): List<DetailTrailer>? {
    val value = first("trailers") ?: return null
    val items = when (value) {
        is JsonArray -> value.toList()
        else -> listOf(value)
    }
    return items.mapNotNull { runCatching { detailTrailerFromJson(it) }.getOrNull() }
        .takeIf { it.isNotEmpty() }
}

private fun JsonObject.castMemberName(): String {
    val explicitName = first("name", "fullName", "full_name", "actor", "person")
        ?.castNameValue()
    val firstName = text("firstName", "first_name", "first", "givenName", "given_name")
    val lastName = text("lastName", "last_name", "last", "surname", "familyName", "family_name")
    return when {
        !explicitName.isNullOrBlank() && !lastName.isNullOrBlank() &&
            !explicitName.contains(lastName, ignoreCase = true) -> "$explicitName $lastName"
        !explicitName.isNullOrBlank() -> explicitName
        !firstName.isNullOrBlank() || !lastName.isNullOrBlank() ->
            listOfNotNull(firstName, lastName).joinToString(" ")
        else -> ""
    }
}

private fun JsonElement.castNameValue(): String? {
    safeString()?.trim()?.takeIf { it.isNotBlank() }?.let { return it }
    val obj = asObjectOrNull() ?: return null
    val explicitName = obj.first("fullName", "full_name", "name")
        ?.safeString()
        ?.trim()
        ?.takeIf { it.isNotBlank() }
    val firstName = obj.text("firstName", "first_name", "first", "givenName", "given_name")
    val lastName = obj.text("lastName", "last_name", "last", "surname", "familyName", "family_name")
    return when {
        !explicitName.isNullOrBlank() && !lastName.isNullOrBlank() &&
            !explicitName.contains(lastName, ignoreCase = true) -> "$explicitName $lastName"
        !explicitName.isNullOrBlank() -> explicitName
        !firstName.isNullOrBlank() || !lastName.isNullOrBlank() ->
            listOfNotNull(firstName, lastName).joinToString(" ")
        else -> null
    }
}

private fun castMemberFromJson(json: JsonElement): CastMember {
    if (json is JsonPrimitive && json.isString) return CastMember(json.content, null, null)
    val obj = json.asObjectOrNull() ?: return CastMember("", null, null)
    return CastMember(
        name = obj.castMemberName(),
        character = obj.text("character", "role", "as"),
        profilePath = obj.text("profilePath", "profile_path", "photo", "profile", "image", "img")
    )
}

private fun JsonObject.castList(vararg keys: String): List<CastMember>? {
    val value = first(*keys) ?: return null
    val items = when (value) {
        is JsonArray -> value.toList()
        else -> listOf(value)
    }
    return items.mapNotNull { runCatching { castMemberFromJson(it) }.getOrNull() }
        .filter { it.name.isNotBlank() }
        .takeIf { it.isNotEmpty() }
}

private fun JsonObject.linkList(): List<MetaLink>? =
    objectList("links").mapNotNull { obj ->
        val name = obj.text("name") ?: return@mapNotNull null
        val category = obj.text("category") ?: return@mapNotNull null
        val url = obj.text("url") ?: return@mapNotNull null
        MetaLink(name, category, url)
    }.takeIf { it.isNotEmpty() }

private fun JsonObject.ratingList(): List<MetaRating>? =
    objectList("ratings").mapNotNull { obj ->
        val source = obj.text("source") ?: return@mapNotNull null
        MetaRating(source = source, value = obj.first("value")?.safeString())
    }.takeIf { it.isNotEmpty() }

private fun JsonObject.appExtras(): AppExtras? {
    val obj = first("app_extras", "appExtras").asObjectOrNull() ?: return null
    return AppExtras(
        seasonPosters = obj.first("seasonPosters")?.let { value ->
            when (value) {
                is JsonArray -> value.map { it.safeString() }
                else -> listOf(value.safeString())
            }
        },
        certification = obj.text("certification"),
        certificationLocal = obj.text("certificationLocal", "certification_local"),
        cast = obj.castList("cast")
    )
}

private fun metaDetailFromJson(json: JsonElement): MetaDetail {
    val obj = json.asObjectOrNull() ?: JsonObject(emptyMap())
    return MetaDetail(
        id = obj.text("id").orEmpty(),
        type = obj.text("type").orEmpty(),
        name = obj.text("name").orEmpty(),
        genres = obj.stringList("genres"),
        poster = obj.text("poster"),
        background = obj.text("background"),
        logo = obj.text("logo"),
        description = obj.text("description"),
        releaseInfo = obj.text("releaseInfo", "year"),
        released = obj.text("released"),
        runtime = obj.text("runtime"),
        videos = obj.videoList("videos", "episodes"),
        trailers = obj.trailerList(),
        imdbRating = obj.text("imdbRating", "imdb_rating"),
        ageRating = obj.text("ageRating", "age_rating"),
        ratings = obj.ratingList(),
        cast = obj.castList("cast"),
        director = obj.stringList("director"),
        links = obj.linkList(),
        status = obj.text("status"),
        seasonsCount = obj.int("seasonsCount", "seasons_count"),
        platforms = obj.stringList("platforms"),
        awards = obj.text("awards"),
        originalLanguage = obj.text("originalLanguage", "original_language"),
        originalName = obj.text("originalName", "original_name"),
        country = obj.text("country"),
        productionCompanies = obj.stringList("productionCompanies", "production_companies"),
        networks = obj.stringList("networks"),
        collectionName = obj.text("collectionName", "collection_name"),
        collectionId = obj.int("collectionId", "collection_id"),
        collectionParts = null,
        seasonPosters = obj.stringMap("seasonPosters", "season_posters"),
        appExtras = obj.appExtras()
    )
}

object CastMemberSerializer : KSerializer<CastMember> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("CastMember", PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: CastMember) {
        val json = encoder as? JsonEncoder ?: error("CastMember can only be serialized to JSON")
        json.encodeJsonElement(buildJsonObject {
            put("name", value.name)
            value.character?.let { put("character", it) }
            value.profilePath?.let { put("profilePath", it) }
        })
    }

    override fun deserialize(decoder: Decoder): CastMember {
        val json = decoder as? JsonDecoder ?: error("CastMember can only be deserialized from JSON")
        return castMemberFromJson(json.decodeJsonElement())
    }
}

object DetailTrailerSerializer : KSerializer<DetailTrailer> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("DetailTrailer", PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: DetailTrailer) {
        val json = encoder as? JsonEncoder ?: error("DetailTrailer can only be serialized to JSON")
        json.encodeJsonElement(buildJsonObject {
            put("id", value.id)
            put("title", value.title)
            put("type", value.type)
            put("url", value.url)
            value.thumbnail?.let { put("thumbnail", it) }
            put("source", value.source)
        })
    }

    override fun deserialize(decoder: Decoder): DetailTrailer {
        val json = decoder as? JsonDecoder ?: error("DetailTrailer can only be deserialized from JSON")
        return detailTrailerFromJson(json.decodeJsonElement())
    }
}

object MetaDetailSerializer : KSerializer<MetaDetail> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("MetaDetail", PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: MetaDetail) {
        val json = encoder as? JsonEncoder ?: error("MetaDetail can only be serialized to JSON")
        val jsonInstance = json.json
        json.encodeJsonElement(buildJsonObject {
            put("id", value.id)
            put("type", value.type)
            put("name", value.name)
            value.genres?.let { put("genres", jsonInstance.encodeToJsonElement(it)) }
            value.poster?.let { put("poster", it) }
            value.background?.let { put("background", it) }
            value.logo?.let { put("logo", it) }
            value.description?.let { put("description", it) }
            value.releaseInfo?.let { put("releaseInfo", it) }
            value.released?.let { put("released", it) }
            value.runtime?.let { put("runtime", it) }
            value.videos?.let { put("videos", jsonInstance.encodeToJsonElement(it)) }
            value.trailers?.let { put("trailers", jsonInstance.encodeToJsonElement(it)) }
            value.imdbRating?.let { put("imdbRating", it) }
            value.ageRating?.let { put("ageRating", it) }
            value.ratings?.let { put("ratings", jsonInstance.encodeToJsonElement(it)) }
            value.cast?.let { put("cast", jsonInstance.encodeToJsonElement(it)) }
            value.director?.let { put("director", jsonInstance.encodeToJsonElement(it)) }
            value.links?.let { put("links", jsonInstance.encodeToJsonElement(it)) }
            value.status?.let { put("status", it) }
            value.seasonsCount?.let { put("seasonsCount", it) }
            value.platforms?.let { put("platforms", jsonInstance.encodeToJsonElement(it)) }
            value.awards?.let { put("awards", it) }
            value.originalLanguage?.let { put("originalLanguage", it) }
            value.originalName?.let { put("originalName", it) }
            value.country?.let { put("country", it) }
            value.productionCompanies?.let { put("productionCompanies", jsonInstance.encodeToJsonElement(it)) }
            value.networks?.let { put("networks", jsonInstance.encodeToJsonElement(it)) }
            value.collectionName?.let { put("collectionName", it) }
            value.collectionId?.let { put("collectionId", it) }
            value.seasonPosters?.let { put("seasonPosters", jsonInstance.encodeToJsonElement(it)) }
            value.appExtras?.let { put("appExtras", jsonInstance.encodeToJsonElement(it)) }
        })
    }

    override fun deserialize(decoder: Decoder): MetaDetail {
        val json = decoder as? JsonDecoder ?: error("MetaDetail can only be deserialized from JSON")
        return metaDetailFromJson(json.decodeJsonElement())
    }
}
