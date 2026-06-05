package com.fluxa.app.ui.catalog

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.util.concurrent.ConcurrentHashMap

object AvatarProvider {
    private val cache = ConcurrentHashMap<String, List<AvatarCharacter>>()

    suspend fun fetchAvatarsForShow(context: Context, folderName: String): List<AvatarCharacter> = withContext(Dispatchers.IO) {
        cache[folderName]?.let { return@withContext it }

        val basePath = "profile_avatars/$folderName"
        val characters = runCatching {
            context.assets.list(basePath)
                .orEmpty()
                .filter { fileName -> fileName.endsWith(".jpg", true) || fileName.endsWith(".jpeg", true) || fileName.endsWith(".png", true) || fileName.endsWith(".webp", true) || fileName.endsWith(".gif", true) }
                .sorted()
                .map { fileName ->
                    val name = fileName.substringBeforeLast('.')
                        .replace('-', ' ')
                        .replace('_', ' ')
                        .split(' ')
                        .filter { it.isNotBlank() }
                        .joinToString(" ") { part -> part.replaceFirstChar(Char::titlecase) }
                    AvatarCharacter(name, "file:///android_asset/$basePath/${File(fileName).name}")
                }
        }.getOrDefault(emptyList())
        cache[folderName] = characters
        characters
        }
}
