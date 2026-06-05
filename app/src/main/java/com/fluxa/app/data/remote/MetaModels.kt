package com.fluxa.app.data.remote

import androidx.compose.runtime.Immutable
import com.google.gson.JsonDeserializer
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.google.gson.annotations.JsonAdapter
import com.google.gson.annotations.SerializedName
import java.lang.reflect.Type

@Immutable
data class MetaRating(val source: String, val value: Any?)

@Immutable
data class AppExtras(
    val seasonPosters: List<String?>? = null,
    val certification: String? = null,
    val certificationLocal: String? = null,
    val cast: List<CastMember>? = null
)

@Immutable
@JsonAdapter(MetaDetailDeserializer::class)
data class MetaDetail(
    val id: String,
    val type: String,
    val name: String,
    @JsonAdapter(StringListDeserializer::class) val genres: List<String>?,
    val poster: String?,
    val background: String?,
    val logo: String?,
    val description: String?,
    val releaseInfo: String?,
    val released: String? = null,
    val runtime: String?,
    @SerializedName(value = "videos", alternate = ["episodes"]) val videos: List<Video>?,
    val trailers: List<DetailTrailer>? = null,
    val imdbRating: String? = null,
    val ageRating: String? = null,
    val ratings: List<MetaRating>? = null,
    val cast: List<CastMember>? = null,
    @JsonAdapter(StringListDeserializer::class) val director: List<String>? = null,
    val links: List<MetaLink>? = null,
    val status: String? = null,
    val seasonsCount: Int? = null,
    @JsonAdapter(StringListDeserializer::class) val platforms: List<String>? = null,
    val awards: String? = null,
    val originalLanguage: String? = null,
    val originalName: String? = null,
    val country: String? = null,
    @JsonAdapter(StringListDeserializer::class) val productionCompanies: List<String>? = null,
    @JsonAdapter(StringListDeserializer::class) val networks: List<String>? = null,
    val collectionName: String? = null,
    val collectionId: Int? = null,
    val collectionParts: List<Meta>? = null,
    val seasonPosters: Map<String, String>? = null,
    @SerializedName("app_extras") val appExtras: AppExtras? = null
)

class MetaDetailDeserializer : JsonDeserializer<MetaDetail> {
    override fun deserialize(json: JsonElement, typeOfT: Type, context: com.google.gson.JsonDeserializationContext): MetaDetail {
        val obj = json.asObjectOrNull() ?: JsonObject()
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
}

@Immutable
@JsonAdapter(DetailTrailerDeserializer::class)
data class DetailTrailer(
    val id: String,
    val title: String,
    val type: String,
    val url: String,
    val thumbnail: String? = null,
    val source: String
)

class DetailTrailerDeserializer : JsonDeserializer<DetailTrailer> {
    override fun deserialize(json: JsonElement, typeOfT: Type, context: com.google.gson.JsonDeserializationContext): DetailTrailer {
        return detailTrailerFromJson(json)
    }
}

class StringListDeserializer : JsonDeserializer<List<String>?> {
    override fun deserialize(json: JsonElement, typeOfT: Type, context: com.google.gson.JsonDeserializationContext): List<String>? {
        if (json.isJsonNull) return null
        if (json.isJsonArray) {
            return json.asJsonArray
                .mapNotNull { item -> item.safeString()?.trim()?.takeIf { value -> value.isNotBlank() } }
                .takeIf { it.isNotEmpty() }
        }
        return json.safeString()
            ?.takeIf { it.isNotBlank() }
            ?.split(',')
            ?.mapNotNull { it.trim().takeIf(String::isNotBlank) }
            ?.takeIf { it.isNotEmpty() }
    }
}

@Immutable
@JsonAdapter(CastMemberDeserializer::class)
data class CastMember(val name: String, val character: String?, val profilePath: String?)

class CastMemberDeserializer : JsonDeserializer<CastMember> {
    override fun deserialize(json: JsonElement, typeOfT: Type, context: com.google.gson.JsonDeserializationContext): CastMember {
        return castMemberFromJson(json)
    }
}
@Immutable
data class MetaLink(val name: String, val category: String, val url: String)
@Immutable
data class Video(
    val id: String,
    @SerializedName(value = "name", alternate = ["title"]) val name: String?,
    val season: Int?,
    @SerializedName(value = "number", alternate = ["episode"]) val number: Int?,
    @SerializedName(value = "released", alternate = ["firstAired"]) val released: String?,
    val thumbnail: String?,
    @SerializedName(value = "overview", alternate = ["description"]) val overview: String? = null,
    val rating: String? = null,
    val episodeRuntime: Int? = null
)

@Immutable
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
    val originalName: String? = null, // TMDB original_name for CS3 multi-search
    val continueWatchingPoster: String? = null,
    val continueWatchingBackground: String? = null,
    val focusGifUrl: String? = null,
    val coverEmoji: String? = null,
    val hideTitle: Boolean? = null,
    val focusGlowEnabled: Boolean? = null,
    @SerializedName(value = "videos", alternate = ["episodes"]) val videos: List<Video>? = null,
    val trailers: List<DetailTrailer>? = null,
    val seasonPosters: Map<String, String>? = null
)

private fun JsonElement?.safeString(): String? {
    val element = this?.takeIf { !it.isJsonNull } ?: return null
    return if (element.isJsonPrimitive) element.asJsonPrimitive.asString else null
}

private fun JsonElement?.asObjectOrNull(): JsonObject? {
    val element = this?.takeIf { !it.isJsonNull } ?: return null
    return if (element.isJsonObject) element.asJsonObject else null
}

private fun JsonObject.first(vararg keys: String): JsonElement? =
    keys.firstNotNullOfOrNull { key -> get(key)?.takeIf { !it.isJsonNull } }

private fun JsonObject.text(vararg keys: String): String? =
    first(*keys)?.safeString()?.trim()?.takeIf { it.isNotBlank() }

private fun detailTrailerFromJson(json: JsonElement): DetailTrailer {
    val obj = json.asObjectOrNull() ?: JsonObject().apply {
        addProperty("source", json.safeString())
    }
    fun text(vararg keys: String): String? = keys.firstNotNullOfOrNull { key ->
        obj.get(key)?.safeString()?.trim()?.takeIf { it.isNotBlank() }
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

private fun castMemberFromJson(json: JsonElement): CastMember {
    if (json.isJsonPrimitive) return CastMember(json.safeString().orEmpty(), null, null)
    val obj = json.asObjectOrNull() ?: JsonObject()
    return CastMember(
        name = obj.castMemberName(),
        character = obj.text("character", "role", "as"),
        profilePath = obj.text("profilePath", "profile_path", "photo", "profile", "image", "img")
    )
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

private fun JsonObject.int(vararg keys: String): Int? {
    val value = first(*keys)?.safeString()?.trim()?.takeIf { it.isNotBlank() } ?: return null
    return value.toIntOrNull() ?: value.toDoubleOrNull()?.toInt()
}

private fun JsonObject.stringList(vararg keys: String): List<String>? {
    val value = first(*keys) ?: return null
    return when {
        value.isJsonArray -> value.asJsonArray
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
    return when {
        value.isJsonObject -> value.asJsonObject.entrySet()
            .mapNotNull { entry -> entry.value.safeString()?.takeIf(String::isNotBlank)?.let { entry.key to it } }
            .toMap()
            .takeIf { it.isNotEmpty() }
        value.isJsonArray -> value.asJsonArray
            .mapIndexedNotNull { index, item -> item.safeString()?.takeIf(String::isNotBlank)?.let { index.toString() to it } }
            .toMap()
            .takeIf { it.isNotEmpty() }
        else -> null
    }
}

private fun JsonObject.objectList(vararg keys: String): List<JsonObject> {
    val value = first(*keys) ?: return emptyList()
    return when {
        value.isJsonArray -> value.asJsonArray.mapNotNull { it.asObjectOrNull() }
        value.isJsonObject -> listOf(value.asJsonObject)
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

private fun JsonObject.trailerList(): List<DetailTrailer>? {
    val value = first("trailers") ?: return null
    val items = when {
        value.isJsonArray -> value.asJsonArray.toList()
        else -> listOf(value)
    }
    return items.mapNotNull { runCatching { detailTrailerFromJson(it) }.getOrNull() }
        .takeIf { it.isNotEmpty() }
}

private fun JsonObject.ratingList(): List<MetaRating>? =
    objectList("ratings").mapNotNull { obj ->
        val source = obj.text("source") ?: return@mapNotNull null
        MetaRating(source = source, value = obj.first("value")?.safeString())
    }.takeIf { it.isNotEmpty() }

private fun JsonObject.castList(vararg keys: String): List<CastMember>? {
    val value = first(*keys) ?: return null
    val items = when {
        value.isJsonArray -> value.asJsonArray.toList()
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

private fun JsonObject.appExtras(): AppExtras? {
    val obj = first("app_extras", "appExtras").asObjectOrNull() ?: return null
    return AppExtras(
        seasonPosters = obj.first("seasonPosters")?.let { value ->
            when {
                value.isJsonArray -> value.asJsonArray.map { it.safeString() }
                else -> listOf(value.safeString())
            }
        },
        certification = obj.text("certification"),
        certificationLocal = obj.text("certificationLocal", "certification_local"),
        cast = obj.castList("cast")
    )
}
