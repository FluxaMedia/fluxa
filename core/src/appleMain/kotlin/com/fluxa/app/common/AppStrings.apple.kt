package com.fluxa.app.common

import kotlinx.cinterop.BetaInteropApi
import kotlinx.cinterop.ExperimentalForeignApi
import platform.Foundation.NSBundle
import platform.Foundation.NSData
import platform.Foundation.NSString
import platform.Foundation.NSUTF8StringEncoding
import platform.Foundation.create
import platform.Foundation.dataWithContentsOfFile

@OptIn(ExperimentalForeignApi::class, BetaInteropApi::class)
internal actual fun readI18nAssetText(fileName: String): String? {
    val baseName = fileName.removeSuffix(".json")
    val path = NSBundle.mainBundle.pathForResource(baseName, "json", "i18n") ?: return null
    val data = NSData.dataWithContentsOfFile(path) ?: return null
    return NSString.create(data, NSUTF8StringEncoding)?.toString()
}

internal actual fun createStringsCache(): MutableMap<String, AppStrings> = mutableMapOf()
