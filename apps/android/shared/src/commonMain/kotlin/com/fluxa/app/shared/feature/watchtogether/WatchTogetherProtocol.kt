package com.fluxa.app.shared.feature.watchtogether

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put

/**
 * Single source of truth for the Watch Together wire protocol.
 *
 * Keep raw JSON keys and message type strings here so transports, coordinator logic and tests do
 * not silently drift apart when the protocol evolves.
 */
internal object WatchTogetherProtocol {
    const val CREATE = "create"
    const val JOIN = "join"
    const val LEAVE = "leave"
    const val STATE = "state"
    const val CONTENT = "content"
    const val BUFFERING = "buffering"
    const val PING = "ping"

    const val ROOM = "room"
    const val MEMBERS = "members"
    const val SYNC = "sync"
    const val ERROR = "error"
    const val PONG = "pong"

    private val json = Json { ignoreUnknownKeys = true }

    fun parse(text: String): JsonObject? = runCatching {
        json.parseToJsonElement(text).jsonObject
    }.getOrNull()

    fun create(displayName: String): JsonObject = buildJsonObject {
        put(Type, CREATE)
        put(Name, displayName)
    }

    fun join(roomCode: String, displayName: String): JsonObject = buildJsonObject {
        put(Type, JOIN)
        put(RoomCode, roomCode)
        put(Name, displayName)
    }

    fun leave(): JsonObject = buildJsonObject { put(Type, LEAVE) }

    fun ping(clientTimeMs: Long): JsonObject = buildJsonObject {
        put(Type, PING)
        put(ClientTimeMs, clientTimeMs)
    }

    fun buffering(isBuffering: Boolean): JsonObject = buildJsonObject {
        put(Type, BUFFERING)
        put(Buffering, isBuffering)
    }

    fun playbackState(snapshot: WatchTogetherPlaybackSnapshot, content: WatchTogetherContent?): JsonObject = buildJsonObject {
        put(Type, STATE)
        put(PositionMs, snapshot.positionMs)
        put(DurationMs, snapshot.durationMs)
        put(Playing, snapshot.isPlaying)
        put(Buffering, snapshot.isBuffering)
        content?.let { putContent(it) }
    }

    fun content(content: WatchTogetherContent): JsonObject = buildJsonObject {
        put(Type, CONTENT)
        putContent(content)
    }

    fun messageType(obj: JsonObject): String? = obj.string(Type)
    fun errorMessage(obj: JsonObject): String? = obj.string(Message)
    fun roomCode(obj: JsonObject): String? = obj.string(RoomCode)
    fun clientId(obj: JsonObject): String? = obj.string(ClientId)
    fun hostId(obj: JsonObject): String? = obj.string(HostId)
    fun senderId(obj: JsonObject): String? = obj.string(SenderId)
    fun sequence(obj: JsonObject): Long? = obj.long(Sequence)
    fun clientTimeMs(obj: JsonObject): Long? = obj.long(ClientTimeMs)
    fun serverTimeMs(obj: JsonObject): Long? = obj.long(ServerTimeMs)
    fun positionMs(obj: JsonObject): Long? = obj.long(PositionMs)
    fun playing(obj: JsonObject): Boolean? = obj.boolean(Playing)
    fun anyBuffering(obj: JsonObject): Boolean? = obj.boolean(AnyBuffering) ?: obj.boolean(Buffering)

    fun contentFrom(obj: JsonObject): WatchTogetherContent? {
        val id = obj.string(ContentId)?.takeIf { it.isNotBlank() } ?: return null
        return WatchTogetherContent(
            id = id,
            type = obj.string(ContentType) ?: "movie",
            videoId = obj.string(VideoId),
            title = obj.string(Title).orEmpty(),
        )
    }

    fun members(obj: JsonObject): List<WatchTogetherMember> {
        val hostId = hostId(obj)
        val values = (obj[Members] as? JsonArray) ?: return emptyList()
        return values.mapNotNull { element ->
            val member = runCatching { element.jsonObject }.getOrNull() ?: return@mapNotNull null
            val id = member.string(Id) ?: return@mapNotNull null
            WatchTogetherMember(
                id = id,
                name = member.string(Name) ?: "Guest",
                isHost = id == hostId,
                buffering = member.boolean(Buffering) ?: false,
            )
        }
    }

    private fun kotlinx.serialization.json.JsonObjectBuilder.putContent(content: WatchTogetherContent) {
        put(ContentId, content.id)
        put(ContentType, content.type)
        content.videoId?.let { put(VideoId, it) }
        put(Title, content.title)
    }

    private fun JsonObject.string(key: String): String? = this[key]?.jsonPrimitive?.contentOrNull
    private fun JsonObject.long(key: String): Long? = this[key]?.jsonPrimitive?.longOrNull
    private fun JsonObject.boolean(key: String): Boolean? = this[key]?.jsonPrimitive?.booleanOrNull

    private const val Type = "type"
    private const val Name = "name"
    private const val RoomCode = "room"
    private const val Message = "message"
    private const val ClientId = "clientId"
    private const val HostId = "hostId"
    private const val SenderId = "senderId"
    private const val Members = "members"
    private const val Id = "id"
    private const val Sequence = "sequence"
    private const val ClientTimeMs = "clientTimeMs"
    private const val ServerTimeMs = "serverTimeMs"
    private const val PositionMs = "positionMs"
    private const val DurationMs = "durationMs"
    private const val Playing = "playing"
    private const val Buffering = "buffering"
    private const val AnyBuffering = "anyBuffering"
    private const val ContentId = "contentId"
    private const val ContentType = "contentType"
    private const val VideoId = "videoId"
    private const val Title = "title"
}
