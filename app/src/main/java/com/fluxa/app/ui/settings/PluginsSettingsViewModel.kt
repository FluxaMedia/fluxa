package com.fluxa.app.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.fluxa.app.plugins.PluginRepositoryManager
import com.fluxa.app.plugins.PluginsUiState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class PluginsSettingsViewModel @Inject constructor(
    private val manager: PluginRepositoryManager
) : ViewModel() {

    val state: StateFlow<PluginsUiState> = manager.state
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), PluginsUiState())

    fun addRepository(url: String) {
        val trimmed = url.trim()
        if (trimmed.isEmpty()) return
        viewModelScope.launch { manager.addRepository(trimmed) }
    }

    fun removeRepository(manifestUrl: String) {
        viewModelScope.launch { manager.removeRepository(manifestUrl) }
    }

    fun toggleScraper(scraperId: String, enabled: Boolean) {
        viewModelScope.launch { manager.toggleScraper(scraperId, enabled) }
    }
}
