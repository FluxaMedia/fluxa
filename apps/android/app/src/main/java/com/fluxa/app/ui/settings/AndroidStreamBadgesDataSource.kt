package com.fluxa.app.ui.settings

import android.content.Context
import com.fluxa.app.data.local.StreamBadgeRulesStore
import com.fluxa.app.shared.feature.streambadges.StreamBadgeImportUiModel
import com.fluxa.app.shared.feature.streambadges.StreamBadgesDataSource
import com.fluxa.app.shared.feature.streambadges.StreamBadgesUiState
import com.google.gson.JsonParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.withContext
import java.net.URL

class AndroidStreamBadgesDataSource(
    private val context: Context
) : StreamBadgesDataSource {

    private val uiState = MutableStateFlow(stateFromRulesJson(StreamBadgeRulesStore.read(context)))

    override fun observeStreamBadges(): Flow<StreamBadgesUiState> = uiState.asStateFlow()

    override suspend fun setImportUrlDraft(url: String) {
        uiState.value = uiState.value.copy(importUrlDraft = url)
    }

    override suspend fun importFromDraft() {
        val url = uiState.value.importUrlDraft.trim()
        if (url.isEmpty()) return
        uiState.value = uiState.value.copy(isImporting = true, error = null)
        val result = withContext(Dispatchers.IO) {
            runCatching {
                val payload = URL(url).readText()
                StreamBadgeRulesStore.importFromUrl(context, url, payload)
            }
        }
        uiState.value = result.fold(
            onSuccess = { rulesJson -> stateFromRulesJson(rulesJson) },
            onFailure = { error ->
                uiState.value.copy(isImporting = false, error = error.message ?: "Badge import failed")
            }
        )
    }

    override suspend fun setActiveSource(sourceUrl: String) {
        withContext(Dispatchers.IO) { StreamBadgeRulesStore.setActiveSource(context, sourceUrl) }
        uiState.value = stateFromRulesJson(StreamBadgeRulesStore.read(context)).copy(importUrlDraft = uiState.value.importUrlDraft)
    }

    override suspend fun removeSource(sourceUrl: String) {
        withContext(Dispatchers.IO) { StreamBadgeRulesStore.removeSource(context, sourceUrl) }
        uiState.value = stateFromRulesJson(StreamBadgeRulesStore.read(context)).copy(importUrlDraft = uiState.value.importUrlDraft)
    }

    override suspend fun dismissError() {
        uiState.value = uiState.value.copy(error = null)
    }

    override suspend fun setBadgePlacement(placement: String) {
        withContext(Dispatchers.IO) { StreamBadgeRulesStore.writePlacement(context, placement) }
        uiState.value = uiState.value.copy(badgePlacement = StreamBadgeRulesStore.readPlacement(context))
    }

    private fun stateFromRulesJson(rulesJson: String): StreamBadgesUiState {
        val imports = runCatching {
            val root = JsonParser.parseString(rulesJson).asJsonObject
            root.getAsJsonArray("imports")?.map { element ->
                val obj = element.asJsonObject
                StreamBadgeImportUiModel(
                    sourceUrl = obj.get("sourceUrl")?.asString.orEmpty(),
                    filterCount = obj.getAsJsonArray("filters")?.size() ?: 0,
                    isActive = obj.get("isActive")?.asBoolean ?: false
                )
            }.orEmpty()
        }.getOrDefault(emptyList())
        return StreamBadgesUiState(imports = imports, badgePlacement = StreamBadgeRulesStore.readPlacement(context))
    }
}
