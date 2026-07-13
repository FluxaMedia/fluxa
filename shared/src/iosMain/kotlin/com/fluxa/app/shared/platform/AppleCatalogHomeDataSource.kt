package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.catalog.CatalogHomeDataSource
import com.fluxa.app.shared.feature.catalog.CatalogHomeUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class AppleCatalogHomeDataSource : CatalogHomeDataSource {
    private val state = MutableStateFlow(CatalogHomeUiState())

    override fun observeHome(): Flow<CatalogHomeUiState> = state.asStateFlow()

    override suspend fun refresh() = Unit

    override suspend fun loadMore(rowId: String) = Unit

    fun update(home: CatalogHomeUiState) {
        state.value = home
    }
}
