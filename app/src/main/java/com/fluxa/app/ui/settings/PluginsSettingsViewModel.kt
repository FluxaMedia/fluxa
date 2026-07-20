package com.fluxa.app.ui.settings

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.fluxa.app.plugins.PluginRepositoryManager
import com.fluxa.app.plugins.PluginScraperUiModel
import com.fluxa.app.plugins.PluginSettingsFieldUiModel
import com.fluxa.app.plugins.PluginsUiState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

data class PluginScraperSettingsUiState(
    val scraper: PluginScraperUiModel,
    val loading: Boolean,
    val fields: List<PluginSettingsFieldUiModel>
)

@HiltViewModel
class PluginsSettingsViewModel @Inject constructor(
    private val manager: PluginRepositoryManager
) : ViewModel() {

    val state: StateFlow<PluginsUiState> = manager.state
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), PluginsUiState())

    var settingsSheet by mutableStateOf<PluginScraperSettingsUiState?>(null)
        private set

    fun addRepository(url: String) {
        val trimmed = url.trim()
        if (trimmed.isEmpty()) return
        viewModelScope.launch { manager.addRepository(trimmed) }
    }

    fun removeRepository(manifestUrl: String) {
        viewModelScope.launch { manager.removeRepository(manifestUrl) }
    }

    fun refreshRepository(manifestUrl: String) {
        viewModelScope.launch { manager.refreshRepository(manifestUrl) }
    }

    fun toggleScraper(scraperId: String, enabled: Boolean) {
        viewModelScope.launch { manager.toggleScraper(scraperId, enabled) }
    }

    fun openScraperSettings(scraper: PluginScraperUiModel) {
        settingsSheet = PluginScraperSettingsUiState(scraper = scraper, loading = true, fields = emptyList())
        viewModelScope.launch {
            val fields = manager.getSettingsLayout(scraper)
            if (settingsSheet?.scraper?.id == scraper.id) {
                settingsSheet = settingsSheet?.copy(loading = false, fields = fields)
            }
        }
    }

    fun dismissScraperSettings() {
        settingsSheet = null
    }

    fun saveScraperSettings(scraperId: String, values: Map<String, Any?>) {
        viewModelScope.launch { manager.updateScraperSettings(scraperId, values) }
        settingsSheet = null
    }
}
