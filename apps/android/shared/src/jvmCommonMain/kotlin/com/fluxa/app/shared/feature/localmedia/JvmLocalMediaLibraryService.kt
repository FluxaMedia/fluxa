package com.fluxa.app.shared.feature.localmedia

import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.platform.PlatformSecureStore
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.coroutines.runBlocking
import java.io.File
import java.security.MessageDigest
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

class JvmLocalMediaLibraryService(
    addonRepository: AddonRepository,
    stateFile: File,
    private val sourceReaders: List<LocalMediaSourceReader>,
    authKey: () -> String,
    localAddons: () -> List<String>,
    language: () -> String,
    private val secureStore: PlatformSecureStore? = null,
    private val ioDispatcher: CoroutineDispatcher = Dispatchers.IO,
) : LocalMediaLibraryService {
    private val store: LocalMediaStateStore = JsonFileLocalMediaStateStore(stateFile)
    private val resolver = LocalMediaMetadataResolver(addonRepository, authKey, localAddons, language)
    private val mutex = Mutex()
    private val scanMutex = Mutex()
    private val credentialCache = ConcurrentHashMap<String, String>()
    @Volatile private var persisted = store.load()
    private val _state = MutableStateFlow(persisted.toSnapshot())
    override val state: StateFlow<LocalMediaLibrarySnapshot> = _state.asStateFlow()
    private val gateway = LocalMediaHttpGateway(::openFile)

    override suspend fun addSource(input: LocalMediaSourceInput) = mutex.withLock {
        val location = input.location.trim()
        if (location.isBlank()) return@withLock
        val duplicate = persisted.sources.any {
            it.sourceType == input.sourceType && it.kind == input.kind && normalizeLocation(it.location) == normalizeLocation(location)
        }
        if (duplicate) return@withLock
        val sourceId = UUID.randomUUID().toString()
        val password = input.password.takeIf(String::isNotBlank)
        val platformSecureStore = secureStore
        val secured = if (password != null && platformSecureStore != null) {
            runCatching { platformSecureStore.writeSecret(credentialKey(sourceId), password) }.isSuccess
        } else false
        if (password != null) credentialCache[sourceId] = password
        val config = LocalMediaSourceConfig(
            id = sourceId,
            kind = input.kind,
            sourceType = input.sourceType,
            location = location,
            displayName = input.displayName.trim().ifBlank { defaultSourceName(input.sourceType, location) },
            username = input.username.trim().takeIf(String::isNotBlank),
            // Prefer the platform secure store. Inline persistence is only a fallback for
            // platforms without one or if secure storage is temporarily unavailable.
            password = password.takeUnless { secured },
        )
        persisted = persisted.copy(sources = persisted.sources + config)
        store.save(persisted)
        _state.value = persisted.toSnapshot()
    }

    override suspend fun removeSource(sourceId: String) = mutex.withLock {
        credentialCache.remove(sourceId)
        secureStore?.let { runCatching { it.removeSecret(credentialKey(sourceId)) } }
        persisted = persisted.copy(
            sources = persisted.sources.filterNot { it.id == sourceId },
            files = persisted.files.filterNot { it.sourceId == sourceId },
        ).let { next ->
            val liveIds = next.files.mapNotNull { it.contentId }.toSet()
            next.copy(catalog = next.catalog.filter { it.contentId in liveIds })
        }
        store.save(persisted)
        _state.value = persisted.toSnapshot()
    }

    override suspend fun scan(forceMetadata: Boolean) {
        scanMutex.withLock {
            val before = mutex.withLock {
                _state.value = persisted.toSnapshot().copy(isScanning = true, error = null)
                persisted
            }
            val result = runCatching {
                withContext(ioDispatcher) { performScan(before, forceMetadata) }
            }
            mutex.withLock {
                result.fold(
                    onSuccess = { scanResult ->
                        // Sources may have been added or removed while a slow NAS scan was in flight.
                        // Keep the latest source configuration and only commit files that still belong
                        // to a live source instead of overwriting concurrent UI changes with the snapshot.
                        val liveSources = persisted.sources
                        val liveSourceIds = liveSources.mapTo(HashSet()) { it.id }
                        val files = scanResult.state.files.filter { it.sourceId in liveSourceIds }
                        val liveContentIds = files.mapNotNullTo(HashSet()) { it.contentId }
                        val next = scanResult.state.copy(
                            sources = liveSources,
                            files = files,
                            catalog = scanResult.state.catalog.filter { it.contentId in liveContentIds },
                        )
                        persisted = next
                        store.save(next)
                        _state.value = next.toSnapshot().copy(error = scanResult.warning)
                    },
                    onFailure = { error ->
                        _state.value = persisted.toSnapshot().copy(
                            error = error.message ?: error::class.simpleName,
                        )
                    },
                )
            }
        }
    }

    private data class ScanResult(
        val state: LocalMediaPersistedState,
        val warning: String? = null,
    )

    private suspend fun performScan(before: LocalMediaPersistedState, forceMetadata: Boolean): ScanResult {
        val oldBySignature = before.files.associateBy { it.signature }
        val discovered = ArrayList<Pair<LocalMediaSourceConfig, LocalMediaFileCandidate>>()
        val sourceErrors = ArrayList<String>()
        for (storedSource in before.sources.filter { it.enabled }) {
            val source = hydrateCredentials(storedSource)
            val reader = sourceReaders.firstOrNull { it.supports(source.sourceType) }
            if (reader == null) {
                sourceErrors += "No reader for ${source.sourceType}"
                continue
            }
            runCatching { reader.listFiles(source) }
                .onSuccess { files -> files.forEach { discovered += source to it } }
                .onFailure { error -> sourceErrors += "${source.displayName}: ${error.message ?: error::class.simpleName}" }
        }

        data class Pending(
            val source: LocalMediaSourceConfig,
            val candidate: LocalMediaFileCandidate,
            val parsed: LocalMediaParsedName,
            val signature: String,
        )

        val retained = ArrayList<LocalMediaIndexedFile>()
        val pending = ArrayList<Pending>()
        for ((source, candidate) in discovered) {
            val signature = signature(source.id, candidate)
            val cached = oldBySignature[signature]
            if (!forceMetadata && cached != null) {
                retained += cached
                continue
            }
            val parsed = jvmLocalMediaCorePolicy.parse(candidate.displayName, candidate.parentHints, source.kind) ?: continue
            pending += Pending(source, candidate, parsed, signature)
        }

        val catalogById = before.catalog.associateBy { it.contentId }.toMutableMap()
        val groups = pending.groupBy { item ->
            listOf(item.source.kind.name, jvmLocalMediaCorePolicy.normalizedTitle(item.parsed.title), item.parsed.year?.toString().orEmpty()).joinToString("|")
        }
        for (group in groups.values) {
            val representative = group.first()
            val match = resolver.resolve(representative.parsed, representative.source.kind)
            for (item in group) {
                val video = match?.let { resolver.resolveVideo(item.parsed, it.detail) }
                val contentId = match?.catalog?.contentId
                retained += LocalMediaIndexedFile(
                    id = stableId(item.source.id, item.candidate.locator),
                    sourceId = item.source.id,
                    locator = item.candidate.locator,
                    displayName = item.candidate.displayName,
                    sizeBytes = item.candidate.sizeBytes,
                    modifiedAtMs = item.candidate.modifiedAtMs,
                    signature = item.signature,
                    parsedTitle = item.parsed.title,
                    parsedYear = item.parsed.year,
                    season = video?.season ?: item.parsed.season,
                    episode = video?.number ?: item.parsed.episode,
                    absoluteEpisode = item.parsed.absoluteEpisode,
                    contentId = contentId,
                    contentType = match?.catalog?.contentType,
                    videoId = video?.id,
                    metadataAddonUrl = match?.catalog?.metadataAddonUrl,
                    matchConfidence = match?.confidence ?: 0f,
                )
            }
            if (match != null) catalogById[match.catalog.contentId] = match.catalog
        }

        val liveIds = retained.mapNotNull { it.contentId }.toSet()
        val fileCounts = retained.mapNotNull { file -> file.contentId?.let { it to file } }.groupBy({ it.first }, { it.second }).mapValues { it.value.size }
        val catalog = catalogById.values
            .filter { it.contentId in liveIds }
            .map { it.copy(fileCount = fileCounts[it.contentId] ?: it.fileCount) }
            .sortedBy { it.title.lowercase() }
        val state = LocalMediaPersistedState(
            sources = before.sources,
            files = retained.distinctBy { it.id },
            catalog = catalog,
            lastScanAtMs = System.currentTimeMillis(),
        )
        val warning = sourceErrors.takeIf { it.isNotEmpty() }?.joinToString("; ")
        if (warning != null && state.files.isEmpty()) error(warning)
        return ScanResult(state, warning)
    }

    override fun playbackStreams(contentId: String, contentType: String, videoId: String?): List<LocalMediaPlaybackStream> {
        val snapshot = persisted
        val sourceById = snapshot.sources.associateBy { it.id }
        val normalizedType = contentType.lowercase()
        val files = snapshot.files.filter { file ->
            file.contentId == contentId && when {
                normalizedType == "movie" -> true
                videoId.isNullOrBlank() -> file.videoId == null || file.season == 1
                file.videoId == videoId -> true
                else -> episodeMatchesVideoId(file, videoId)
            }
        }
        return files.map { file ->
            LocalMediaPlaybackStream(
                fileId = file.id,
                title = file.displayName,
                playableUrl = gateway.urlFor(file.id),
                sizeBytes = file.sizeBytes,
                sourceLabel = sourceById[file.sourceId]?.displayName ?: "Local",
            )
        }
    }

    private fun episodeMatchesVideoId(file: LocalMediaIndexedFile, videoId: String): Boolean {
        val parts = videoId.split(':')
        val season = parts.getOrNull(parts.lastIndex - 1)?.toIntOrNull()
        val episode = parts.lastOrNull()?.toIntOrNull()
        return season != null && episode != null && file.season == season && file.episode == episode
    }

    private fun openFile(fileId: String, offset: Long): LocalMediaOpenedStream? {
        val snapshot = persisted
        val file = snapshot.files.firstOrNull { it.id == fileId } ?: return null
        val storedSource = snapshot.sources.firstOrNull { it.id == file.sourceId } ?: return null
        val source = hydrateCredentialsBlocking(storedSource)
        val reader = sourceReaders.firstOrNull { it.supports(source.sourceType) } ?: return null
        return reader.open(source, file.locator, offset)
    }

    private suspend fun hydrateCredentials(source: LocalMediaSourceConfig): LocalMediaSourceConfig {
        source.password?.takeIf(String::isNotBlank)?.let {
            credentialCache[source.id] = it
            return source
        }
        val cached = credentialCache[source.id]
        if (!cached.isNullOrBlank()) return source.copy(password = cached)
        val password = secureStore?.let { store ->
            runCatching { store.readSecret(credentialKey(source.id)) }.getOrNull()
        }?.takeIf(String::isNotBlank)
        if (password != null) credentialCache[source.id] = password
        return source.copy(password = password)
    }

    private fun hydrateCredentialsBlocking(source: LocalMediaSourceConfig): LocalMediaSourceConfig {
        source.password?.takeIf(String::isNotBlank)?.let { return source }
        credentialCache[source.id]?.takeIf(String::isNotBlank)?.let { return source.copy(password = it) }
        val store = secureStore ?: return source
        val password = runCatching {
            runBlocking(ioDispatcher) { store.readSecret(credentialKey(source.id)) }
        }.getOrNull()?.takeIf(String::isNotBlank)
        if (password != null) credentialCache[source.id] = password
        return source.copy(password = password)
    }

    private fun credentialKey(sourceId: String): String = "local_media.nas.$sourceId.password"

    override fun close() = gateway.close()

    private fun LocalMediaPersistedState.toSnapshot(): LocalMediaLibrarySnapshot {
        val sourceModels = sources.map { source ->
            LocalMediaSourceUiModel(source.id, source.kind, source.sourceType, source.displayName, source.location, source.enabled)
        }
        return LocalMediaLibrarySnapshot(
            sources = sourceModels,
            movies = catalog.filter { it.kind == LocalMediaKind.Movies },
            tvShows = catalog.filter { it.kind == LocalMediaKind.TvShows },
            anime = catalog.filter { it.kind == LocalMediaKind.Anime },
            indexedFileCount = files.size,
            unmatchedFileCount = files.count { it.contentId == null },
            lastScanAtMs = lastScanAtMs,
        )
    }

    private fun signature(sourceId: String, file: LocalMediaFileCandidate): String =
        "$sourceId|${file.locator}|${file.sizeBytes}|${file.modifiedAtMs}"

    private fun stableId(sourceId: String, locator: String): String = sha256("$sourceId|$locator").take(32)

    private fun sha256(value: String): String = MessageDigest.getInstance("SHA-256")
        .digest(value.toByteArray())
        .joinToString("") { "%02x".format(it) }

    private fun normalizeLocation(value: String): String = value.trim().trimEnd('/').lowercase()
    private fun defaultSourceName(type: LocalMediaSourceType, location: String): String = when (type) {
        LocalMediaSourceType.LocalFolder -> File(location.removePrefix("file://")).name.ifBlank { "Local folder" }
        LocalMediaSourceType.Smb -> "SMB · ${location.substringAfter("smb://").substringBefore('/')}"
        LocalMediaSourceType.WebDav -> "WebDAV · ${runCatching { java.net.URI(location).host }.getOrNull() ?: location}"
    }
}
