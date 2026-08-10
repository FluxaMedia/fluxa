package com.fluxa.app.shared.feature.localmedia

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File

@Serializable
internal data class LocalMediaPersistedState(
    val sources: List<LocalMediaSourceConfig> = emptyList(),
    val files: List<LocalMediaIndexedFile> = emptyList(),
    val catalog: List<LocalMediaCatalogEntry> = emptyList(),
    val lastScanAtMs: Long = 0L,
)

internal interface LocalMediaStateStore {
    fun load(): LocalMediaPersistedState
    fun save(state: LocalMediaPersistedState)
}

internal class JsonFileLocalMediaStateStore(
    private val file: File,
) : LocalMediaStateStore {
    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        prettyPrint = false
    }

    override fun load(): LocalMediaPersistedState = runCatching {
        if (!file.isFile) return@runCatching LocalMediaPersistedState()
        json.decodeFromString<LocalMediaPersistedState>(file.readText())
    }.getOrDefault(LocalMediaPersistedState())

    override fun save(state: LocalMediaPersistedState) {
        runCatching {
            file.parentFile?.mkdirs()
            val tmp = File(file.parentFile, "${file.name}.tmp")
            tmp.writeText(json.encodeToString(state))
            if (!tmp.renameTo(file)) {
                file.writeText(tmp.readText())
                tmp.delete()
            }
        }
    }
}
