package com.lagradost.cloudstream3.actions

import android.content.Context
import com.lagradost.cloudstream3.utils.ExtractorLinkType

abstract class VideoClickAction {
    abstract val name: String
    open val sourceTypes: Set<ExtractorLinkType> = emptySet()
    var sourcePlugin: String? = null

    open suspend fun runAction(context: Context, data: String): Boolean = false

    fun uniqueId(): String = "$sourcePlugin:${this::class.qualifiedName}"
}

object VideoClickActionHolder {
    val allVideoClickActions: MutableList<VideoClickAction> = mutableListOf()
}
