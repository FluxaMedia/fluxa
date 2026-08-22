@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import java.io.File

internal fun String.toFileImageModel(): String {
    return File(this).takeIf { it.exists() }?.toURI()?.toString() ?: this
}
