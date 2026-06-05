@file:OptIn(androidx.tv.material3.ExperimentalTvMaterial3Api::class)
package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.compose.runtime.*
import androidx.compose.ui.focus.FocusRequester
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.tv.material3.ExperimentalTvMaterial3Api

@Composable
fun SearchScreen(
    activeProfile: UserProfile?,
    searchResults: List<Meta>,
    onSearch: (String) -> Unit,
    onMovieClick: (Meta, String?, String?) -> Unit,
    onBack: () -> Unit,
    viewModel: HomeViewModel
) {
    val deviceType = LocalDeviceType.current
    val lang = activeProfile?.language ?: "en"
    val focusRequester = remember { FocusRequester() }
    var query by remember { mutableStateOf("") }
    val searchRows by viewModel.searchRows.collectAsStateWithLifecycle()

    LaunchedEffect(deviceType) {
        if (deviceType != DeviceType.Mobile) {
            focusRequester.requestFocus()
        }
    }

    if (deviceType == DeviceType.Mobile) {
        MobileSearchScreen(activeProfile, searchResults, searchRows, query, onSearch = { query = it; onSearch(it) }, onBack, onMovieClick, focusRequester, lang, viewModel)
    } else {
        TvSearchScreenContent(activeProfile, searchResults, searchRows, query, onSearch = { query = it; onSearch(it) }, onBack, onMovieClick, focusRequester, lang, viewModel)
    }
}
