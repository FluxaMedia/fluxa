package com.fluxa.app.data.local

import androidx.room.Room
import androidx.sqlite.driver.bundled.BundledSQLiteDriver
import kotlinx.coroutines.Dispatchers
import java.io.File

fun buildDesktopAppDatabase(databaseFile: File): AppDatabase {
    databaseFile.parentFile?.mkdirs()
    return Room.databaseBuilder<AppDatabase>(
        name = databaseFile.absolutePath,
        factory = { AppDatabaseConstructor.initialize() }
    )
        .setDriver(BundledSQLiteDriver())
        .setQueryCoroutineContext(Dispatchers.IO)
        .fallbackToDestructiveMigration(false)
        .build()
}
