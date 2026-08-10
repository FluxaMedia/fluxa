package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import kotlinx.coroutines.delay
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings
import com.fluxa.app.shared.feature.profile.ProfileAction
import com.fluxa.app.shared.feature.profile.ProfileUiState
import com.fluxa.app.ui.catalog.FluxaColors

@Composable
internal fun SettingsAccountContent(
    model: SettingsAccountUiModel,
    lang: String?,
    brandIcons: SettingsBrandIcons,
    onAction: (SettingsAction) -> Unit,
    onNavigate: (SettingsCategory) -> Unit,
    profileState: ProfileUiState? = null,
    onProfileAction: (ProfileAction) -> Unit = {},
    onSwitchProfiles: () -> Unit = {},
    continueWatchingSource: String?,
    onContinueWatchingSourceChanged: (String) -> Unit
) {
    profileState?.takeIf { it.profiles.isNotEmpty() }?.let { profilesState ->
        SettingsSectionHeader(AppStrings.t(lang, "auto.profile"))
        SettingsGroupCard {
            profilesState.profiles.forEach { profile ->
                val active = profile.id == profilesState.activeProfile?.id
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable(enabled = !active) { onProfileAction(ProfileAction.Selected(profile)) }
                        .padding(vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .clip(CircleShape)
                            .background(FluxaColors.surfaceRaised)
                            .then(
                                if (active) Modifier.border(2.dp, LocalSettingsAccentColor.current, CircleShape)
                                else Modifier
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        if (!profile.avatarUrl.isNullOrBlank()) {
                            com.fluxa.app.shared.image.FluxaRemoteImage(
                                imageUrl = profile.avatarUrl,
                                cacheKey = "settings-profile:${profile.avatarUrl}",
                                contentDescription = profile.name,
                                modifier = Modifier.size(40.dp).clip(CircleShape),
                                contentScale = ContentScale.Crop
                            )
                        } else {
                            com.fluxa.app.shared.feature.profile.ProfileDefaultAvatar(Modifier.size(22.dp))
                        }
                    }
                    Text(
                        text = profile.name,
                        color = Color.White,
                        modifier = Modifier.weight(1f),
                        fontWeight = if (active) androidx.compose.ui.text.font.FontWeight.SemiBold else androidx.compose.ui.text.font.FontWeight.Normal
                    )
                    if (active) {
                        Text("✓", color = LocalSettingsAccentColor.current, fontSize = 16.sp)
                    }
                }
            }
            SettingsNavRow(AppStrings.t(lang, "profiles.add_profile")) {
                onProfileAction(ProfileAction.AddRequested)
            }
            SettingsNavRow(AppStrings.t(lang, "profiles.manage")) { onSwitchProfiles() }
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "auto.account_sync"))
    SettingsGroupCard {
        val syncFailedLabel = AppStrings.t(lang, "integration.sync_failed")
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.stremio"),
            connected = model.hasStremio,
            connectedLabel = if (model.email.isNotBlank()) AppStrings.format(lang, "settings.connected_as", model.email) else AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.stremio,
            hasSyncFailure = "stremio" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountStremio) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.nuvio"),
            connected = model.hasNuvio,
            connectedLabel = model.nuvioEmail?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.nuvio,
            hasSyncFailure = "nuvio" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountNuvio) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.trakt"),
            connected = model.hasTrakt,
            connectedLabel = model.traktUsername?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.trakt,
            hasSyncFailure = "trakt" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountTrakt) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.simkl"),
            connected = model.hasSimkl,
            connectedLabel = model.simklUsername?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.simkl,
            hasSyncFailure = "simkl" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountSimkl) }
        SettingsConnectionRow(
            AppStrings.t(lang, "brand.anilist"),
            connected = model.hasAnilist,
            connectedLabel = model.anilistUsername?.takeIf { it.isNotBlank() }?.let { AppStrings.format(lang, "settings.connected_as", it) } ?: AppStrings.t(lang, "auto.connected"),
            icon = brandIcons.anilist,
            hasSyncFailure = "anilist" in model.syncFailedProviders,
            syncFailedLabel = syncFailedLabel
        ) { onNavigate(SettingsCategory.AccountAnilist) }
    }
    SettingsSectionHeader(AppStrings.t(lang, "settings.sync_preferences"))
    SettingsGroupCard {
        val connectedSourceOptions = buildList {
            add(SettingsChoiceOption("local", AppStrings.t(lang, "settings.cw_source_of_truth_local")))
            if (model.hasStremio) add(SettingsChoiceOption("stremio", AppStrings.t(lang, "settings.cw_source_of_truth_stremio")))
            if (model.hasNuvio) add(SettingsChoiceOption("nuvio", AppStrings.t(lang, "settings.cw_source_of_truth_nuvio")))
            if (model.hasTrakt) add(SettingsChoiceOption("trakt", AppStrings.t(lang, "settings.cw_source_of_truth_trakt")))
            if (model.hasSimkl) add(SettingsChoiceOption("simkl", AppStrings.t(lang, "settings.cw_source_of_truth_simkl")))
            if (model.hasAnilist) add(SettingsChoiceOption("anilist", AppStrings.t(lang, "settings.cw_source_of_truth_anilist")))
        }
        val connectedLibraryOptions = buildList {
            add(SettingsChoiceOption("local", AppStrings.t(lang, "settings.cw_source_of_truth_local")))
            if (model.hasStremio) add(SettingsChoiceOption("stremio", AppStrings.t(lang, "settings.cw_source_of_truth_stremio")))
            if (model.hasNuvio) add(SettingsChoiceOption("nuvio", AppStrings.t(lang, "settings.cw_source_of_truth_nuvio")))
            if (model.hasTrakt) add(SettingsChoiceOption("trakt", AppStrings.t(lang, "settings.cw_source_of_truth_trakt")))
            if (model.hasSimkl) add(SettingsChoiceOption("simkl", AppStrings.t(lang, "settings.cw_source_of_truth_simkl")))
            if (model.hasAnilist) add(SettingsChoiceOption("anilist", AppStrings.t(lang, "settings.cw_source_of_truth_anilist")))
        }
        SettingsChoiceRow(
            label = AppStrings.t(lang, "settings.continue_watching_source"),
            value = continueWatchingSource ?: "local",
            options = connectedSourceOptions
        ) { onContinueWatchingSourceChanged(it) }
        SettingsChoiceRow(
            label = AppStrings.t(lang, "settings.library_source_of_truth"),
            value = model.integrationLibrarySource,
            options = connectedLibraryOptions
        ) { onAction(SettingsAction.TmdbAccountChanged(model.copy(integrationLibrarySource = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.apis"))
    val tmdbConfigured = !model.tmdbApiKey.isNullOrBlank()
    val mdblistConfigured = !model.mdblistApiKey.isNullOrBlank()
    SettingsGroupCard {
        SettingsNavRow(
            AppStrings.t(lang, "brand.tmdb"),
            value = AppStrings.t(lang, if (tmdbConfigured) "settings.tmdb_api_configured" else "settings.tmdb_api_not_configured")
        ) { onNavigate(SettingsCategory.TmdbFeatures) }
        SettingsNavRow(
            AppStrings.t(lang, "settings.mdblist_api"),
            value = AppStrings.t(lang, if (mdblistConfigured) "settings.tmdb_api_configured" else "settings.tmdb_api_not_configured")
        ) { onNavigate(SettingsCategory.MdblistApi) }
    }
}

@Composable
internal fun SettingsMdblistApiContent(model: SettingsAccountUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.mdblist_api"))
    SettingsInlineSecretField(
        AppStrings.t(lang, "settings.mdblist_api_key"),
        model.mdblistApiKey.orEmpty(),
        placeholder = AppStrings.t(lang, "settings.mdblist_api_key_placeholder")
    ) {
        onAction(SettingsAction.TmdbAccountChanged(model.copy(mdblistApiKey = it)))
    }
}

internal enum class SettingsAccountProvider { Stremio, Nuvio, Trakt, Simkl, Anilist }

@Composable
internal fun SettingsAccountDetailContent(
    provider: SettingsAccountProvider,
    model: SettingsAccountUiModel,
    lang: String?,
    onAction: (SettingsAction) -> Unit
) {
    val connected = when (provider) {
        SettingsAccountProvider.Stremio -> model.hasStremio
        SettingsAccountProvider.Nuvio -> model.hasNuvio
        SettingsAccountProvider.Trakt -> model.hasTrakt
        SettingsAccountProvider.Simkl -> model.hasSimkl
        SettingsAccountProvider.Anilist -> model.hasAnilist
    }
    val titleKey = when (provider) {
        SettingsAccountProvider.Stremio -> "brand.stremio"
        SettingsAccountProvider.Nuvio -> "brand.nuvio"
        SettingsAccountProvider.Trakt -> "brand.trakt"
        SettingsAccountProvider.Simkl -> "brand.simkl"
        SettingsAccountProvider.Anilist -> "brand.anilist"
    }
    val lastSyncAt = when (provider) {
        SettingsAccountProvider.Stremio -> model.stremioLastSyncAt
        SettingsAccountProvider.Nuvio -> model.nuvioLastSyncAt
        SettingsAccountProvider.Trakt -> model.traktLastSyncAt
        SettingsAccountProvider.Simkl -> model.simklLastSyncAt
        SettingsAccountProvider.Anilist -> model.anilistLastSyncAt
    }
    val email = when (provider) {
        SettingsAccountProvider.Stremio -> model.stremioEmail.orEmpty().ifBlank { model.email }
        SettingsAccountProvider.Nuvio -> model.nuvioEmail.orEmpty().ifBlank { model.email }
        SettingsAccountProvider.Simkl -> model.simklUsername.orEmpty().ifBlank { model.email }
        SettingsAccountProvider.Anilist -> model.anilistUsername.orEmpty().ifBlank { model.email }
        else -> model.email
    }
    val providerKey = when (provider) {
        SettingsAccountProvider.Stremio -> "stremio"
        SettingsAccountProvider.Nuvio -> "nuvio"
        SettingsAccountProvider.Trakt -> "trakt"
        SettingsAccountProvider.Simkl -> "simkl"
        SettingsAccountProvider.Anilist -> "anilist"
    }
    val hasSyncFailure = providerKey in model.syncFailedProviders
    val isSyncing = providerKey in model.syncingProviders
    val isCredentialProvider = provider == SettingsAccountProvider.Stremio || provider == SettingsAccountProvider.Nuvio
    var justSynced by remember(provider) { mutableStateOf(false) }
    var observedSyncStart by remember(provider) { mutableStateOf(false) }
    var confirmingDisconnect by remember(provider) { mutableStateOf(false) }
    var showCredentialForm by remember(provider) { mutableStateOf(false) }
    LaunchedEffect(isSyncing) {
        if (isSyncing) {
            observedSyncStart = true
            justSynced = false
        } else if (observedSyncStart) {
            observedSyncStart = false
            justSynced = true
            delay(2_000L)
            justSynced = false
        }
    }
    LaunchedEffect(connected) {
        if (connected) showCredentialForm = false
    }
    val onSync = {
        onAction(
            when (provider) {
                SettingsAccountProvider.Stremio -> SettingsAction.ConnectStremioRequested
                SettingsAccountProvider.Nuvio -> SettingsAction.ConnectNuvioRequested
                SettingsAccountProvider.Trakt -> SettingsAction.ConnectTraktRequested
                SettingsAccountProvider.Simkl -> SettingsAction.ConnectSimklRequested
                SettingsAccountProvider.Anilist -> SettingsAction.ConnectAnilistRequested
            }
        )
    }
    val onSyncNow = {
        if (connected) {
            onAction(SettingsAction.SyncProviderRequested(providerKey))
        } else {
            onSync()
        }
    }
    val onConnect = {
        if (isCredentialProvider) {
            showCredentialForm = true
        } else {
            onSync()
        }
    }

    if (connected) {
        SettingsConnectedAccountCard(
            statusLabel = AppStrings.t(lang, "settings.connected_account"),
            email = email,
            badgeText = AppStrings.t(lang, "auto.connected")
        )
        if (hasSyncFailure) {
            SettingsGroupCard {
                SettingsInfoRow(AppStrings.t(lang, "common.error"), AppStrings.t(lang, "integration.sync_failed"))
            }
        }
    } else {
    SettingsSectionHeader(AppStrings.t(lang, "settings.sync_with"))
    SettingsGroupCard {
        SettingsActionRow(
            label = AppStrings.t(lang, titleKey),
            value = AppStrings.t(lang, "settings.connect_account"),
            onClick = onConnect
        )
        if (hasSyncFailure) {
            SettingsInfoRow(AppStrings.t(lang, "common.error"), AppStrings.t(lang, "integration.sync_failed"))
        }
    }
    }

    if (!connected && isCredentialProvider && showCredentialForm) {
        SettingsCredentialLoginForm(
            lang = lang,
            busy = isSyncing,
            errorMessage = model.connectErrors[providerKey]?.let { code ->
                when {
                    code == "invalid_credentials" && provider == SettingsAccountProvider.Stremio -> AppStrings.t(lang, "login.stremio_failed")
                    code == "invalid_credentials" -> AppStrings.t(lang, "auth.error.invalid_credentials")
                    else -> AppStrings.format(lang, "login.connection_error", code)
                }
            },
            onSubmit = { email, password ->
                onAction(
                    if (provider == SettingsAccountProvider.Stremio) {
                        SettingsAction.ConnectStremioWithCredentials(email, password)
                    } else {
                        SettingsAction.ConnectNuvioWithCredentials(email, password)
                    }
                )
            },
            onCancel = { showCredentialForm = false }
        )
    }

    if (connected) {
        SettingsSectionHeader(AppStrings.t(lang, "settings.provider_library"))
        SettingsGroupCard {
            SettingsInfoRow(
                AppStrings.t(lang, "integration.sync_now"),
                when {
                    isSyncing -> AppStrings.t(lang, "integration.syncing")
                    justSynced -> AppStrings.t(lang, "integration.synced")
                    lastSyncAt <= 0L -> AppStrings.t(lang, "integration.never_synced")
                    else -> AppStrings.t(lang, "integration.just_now")
                }
            )
            SettingsInfoRow(AppStrings.t(lang, "integration.account_info"), email)
            val importedItems = when (provider) {
                SettingsAccountProvider.Stremio -> model.stremioItemCount
                SettingsAccountProvider.Nuvio -> model.nuvioItemCount
                SettingsAccountProvider.Trakt -> model.traktItemCount
                SettingsAccountProvider.Simkl -> model.simklItemCount
                SettingsAccountProvider.Anilist -> model.anilistItemCount
            }
            SettingsInfoRow(AppStrings.t(lang, "integration.imported_items"), AppStrings.format(lang, "integration.item_count", importedItems))
            val continueWatchingItems = when (provider) {
                SettingsAccountProvider.Stremio -> model.stremioContinueWatchingCount
                SettingsAccountProvider.Nuvio -> model.nuvioContinueWatchingCount
                SettingsAccountProvider.Trakt -> model.traktContinueWatchingCount
                SettingsAccountProvider.Simkl -> model.simklContinueWatchingCount
                SettingsAccountProvider.Anilist -> model.anilistContinueWatchingCount
            }
            SettingsInfoRow(AppStrings.t(lang, "integration.continue_watching"), AppStrings.format(lang, "integration.item_count", continueWatchingItems))
            val libraryItems = when (provider) {
                SettingsAccountProvider.Stremio -> model.stremioLibraryCount
                SettingsAccountProvider.Nuvio -> model.nuvioLibraryCount
                SettingsAccountProvider.Trakt -> model.traktLibraryCount
                SettingsAccountProvider.Simkl -> model.simklLibraryCount
                SettingsAccountProvider.Anilist -> model.anilistLibraryCount
            }
            SettingsInfoRow(AppStrings.t(lang, "integration.library_items"), AppStrings.format(lang, "integration.item_count", libraryItems))
            val providerAddonCount = when (provider) {
                SettingsAccountProvider.Stremio -> model.stremioAddonCount
                SettingsAccountProvider.Nuvio -> model.nuvioAddonCount
                else -> null
            }
            providerAddonCount?.let { addonCount ->
                SettingsInfoRow(
                    AppStrings.t(lang, "integration.addons"),
                    AppStrings.format(lang, "integration.item_count", addonCount)
                )
            }
        }

        if (provider == SettingsAccountProvider.Trakt) {
            SettingsSectionHeader(AppStrings.t(lang, "brand.trakt"))
            SettingsGroupCard {
                SettingsChoiceRow(
                    label = AppStrings.t(lang, "settings.continue_watching_window"),
                    value = model.continueWatchingDays.toString(),
                    options = listOf(
                        SettingsChoiceOption("0", AppStrings.t(lang, "settings.continue_watching_window_all")),
                        SettingsChoiceOption("7", "7"),
                        SettingsChoiceOption("30", "30"),
                        SettingsChoiceOption("90", "90"),
                        SettingsChoiceOption("365", "365")
                    )
                ) { value -> onAction(SettingsAction.TmdbAccountChanged(model.copy(continueWatchingDays = value.toInt()))) }
                SettingsToggleRow(
                    label = AppStrings.t(lang, "settings.trakt_comments"),
                    description = AppStrings.t(lang, "settings.trakt_comments_desc"),
                    value = model.traktCommentsEnabled
                ) { onAction(SettingsAction.TmdbAccountChanged(model.copy(traktCommentsEnabled = it))) }
            }
        }

        SettingsPrimaryButton(AppStrings.t(lang, "integration.sync_now")) { onSyncNow() }
        SettingsDestructiveLink(AppStrings.t(lang, "integration.disconnect")) { confirmingDisconnect = true }
    }

    if (confirmingDisconnect) {
        androidx.compose.material3.AlertDialog(
            onDismissRequest = { confirmingDisconnect = false },
            title = { Text(AppStrings.t(lang, titleKey)) },
            text = { Text(AppStrings.t(lang, "settings.disconnect_confirm")) },
            confirmButton = {
                TextButton(onClick = {
                    confirmingDisconnect = false
                    onAction(SettingsAction.DisconnectProviderRequested(providerKey))
                }) {
                    Text(AppStrings.t(lang, "integration.disconnect"), color = FluxaColors.errorRed)
                }
            },
            dismissButton = {
                TextButton(onClick = { confirmingDisconnect = false }) {
                    Text(AppStrings.t(lang, "common.cancel"))
                }
            }
        )
    }
}

@Composable
internal fun SettingsCredentialLoginForm(
    lang: String?,
    busy: Boolean,
    errorMessage: String?,
    onSubmit: (String, String) -> Unit,
    onCancel: () -> Unit
) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    val fieldColors = OutlinedTextFieldDefaults.colors(
        focusedTextColor = Color.White,
        unfocusedTextColor = Color.White,
        focusedBorderColor = Color.White.copy(alpha = 0.4f),
        unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
        focusedLabelColor = Color.White.copy(alpha = 0.7f),
        unfocusedLabelColor = Color.White.copy(alpha = 0.4f),
        cursorColor = Color.White
    )
    SettingsGroupCard {
        Column(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            OutlinedTextField(
                value = email,
                onValueChange = { email = it },
                label = { Text(AppStrings.t(lang, "auth.field.email")) },
                singleLine = true,
                enabled = !busy,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                colors = fieldColors,
                modifier = Modifier.fillMaxWidth()
            )
            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text(AppStrings.t(lang, "auth.field.password")) },
                singleLine = true,
                enabled = !busy,
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                colors = fieldColors,
                modifier = Modifier.fillMaxWidth()
            )
            if (errorMessage != null) {
                Text(errorMessage, color = FluxaColors.errorRed, fontSize = 12.sp)
            }
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                TextButton(onClick = onCancel, enabled = !busy, modifier = Modifier.weight(1f)) {
                    Text(AppStrings.t(lang, "common.cancel"))
                }
                Button(
                    onClick = { onSubmit(email.trim(), password) },
                    enabled = !busy && email.isNotBlank() && password.isNotBlank(),
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = Color.Black),
                    modifier = Modifier.weight(1f)
                ) {
                    if (busy) {
                        CircularProgressIndicator(modifier = Modifier.size(16.dp), color = Color.Black, strokeWidth = 2.dp)
                    } else {
                        Text(AppStrings.t(lang, "auth.nuvio.sign_in"))
                    }
                }
            }
        }
    }
}

@Composable
internal fun SettingsTmdbFeaturesContent(model: SettingsAccountUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    val allEnabled = model.tmdbCastImagesEnabled && model.tmdbSimilarResultsEnabled && model.tmdbTrailersEnabled &&
        model.tmdbRecommendationsEnabled && model.tmdbCollectionInfoEnabled && model.tmdbEpisodeImagesEnabled &&
        model.tmdbLogosBackdropsEnabled && model.tmdbRatingsEnabled && model.tmdbBasicInfoEnabled &&
        model.tmdbDetailsEnabled && model.tmdbProductionsEnabled && model.tmdbNetworksEnabled

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_api"))
    SettingsInlineSecretField(
        AppStrings.t(lang, "settings.tmdb_api_key"),
        model.tmdbApiKey.orEmpty(),
        placeholder = AppStrings.t(lang, "settings.tmdb_api_key_placeholder")
    ) {
        onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbApiKey = it)))
    }

    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_enable_all"), value = allEnabled) {
            onAction(
                SettingsAction.TmdbAccountChanged(
                    model.copy(
                        tmdbCastImagesEnabled = it, tmdbSimilarResultsEnabled = it, tmdbTrailersEnabled = it,
                        tmdbRecommendationsEnabled = it, tmdbCollectionInfoEnabled = it, tmdbEpisodeImagesEnabled = it,
                        tmdbLogosBackdropsEnabled = it, tmdbRatingsEnabled = it, tmdbBasicInfoEnabled = it,
                        tmdbDetailsEnabled = it, tmdbProductionsEnabled = it, tmdbNetworksEnabled = it
                    )
                )
            )
        }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_group_media_info"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_basic_info"), description = AppStrings.t(lang, "settings.tmdb_basic_info_desc"), value = model.tmdbBasicInfoEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbBasicInfoEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_details"), description = AppStrings.t(lang, "settings.tmdb_details_desc"), value = model.tmdbDetailsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbDetailsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_productions"), description = AppStrings.t(lang, "settings.tmdb_productions_desc"), value = model.tmdbProductionsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbProductionsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_networks"), description = AppStrings.t(lang, "settings.tmdb_networks_desc"), value = model.tmdbNetworksEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbNetworksEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_collection_info"), description = AppStrings.t(lang, "settings.tmdb_collection_info_desc"), value = model.tmdbCollectionInfoEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbCollectionInfoEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_ratings"), description = AppStrings.t(lang, "settings.tmdb_ratings_desc"), value = model.tmdbRatingsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbRatingsEnabled = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_group_images"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_cast_images"), description = AppStrings.t(lang, "settings.tmdb_cast_images_desc"), value = model.tmdbCastImagesEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbCastImagesEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_episode_images"), description = AppStrings.t(lang, "settings.tmdb_episode_images_desc"), value = model.tmdbEpisodeImagesEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbEpisodeImagesEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_logos_backdrops"), description = AppStrings.t(lang, "settings.tmdb_logos_backdrops_desc"), value = model.tmdbLogosBackdropsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbLogosBackdropsEnabled = it))) }
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.tmdb_group_discovery"))
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_similar_results"), description = AppStrings.t(lang, "settings.tmdb_similar_results_desc"), value = model.tmdbSimilarResultsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbSimilarResultsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_recommendations"), description = AppStrings.t(lang, "settings.tmdb_recommendations_desc"), value = model.tmdbRecommendationsEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbRecommendationsEnabled = it))) }
        SettingsToggleRow(AppStrings.t(lang, "settings.tmdb_trailers"), description = AppStrings.t(lang, "settings.tmdb_trailers_desc"), value = model.tmdbTrailersEnabled) { onAction(SettingsAction.TmdbAccountChanged(model.copy(tmdbTrailersEnabled = it))) }
    }
}

@Composable
internal fun SettingsNotificationsContent(model: SettingsNotificationsUiModel, lang: String?, onAction: (SettingsAction) -> Unit) {
    SettingsGroupCard {
        SettingsToggleRow(AppStrings.t(lang, "settings.enable_notifications"), description = AppStrings.t(lang, "settings.enable_notifications_desc"), value = model.notificationsEnabled) {
            onAction(SettingsAction.NotificationsChanged(model.copy(notificationsEnabled = it)))
        }
        SettingsToggleRow(AppStrings.t(lang, "settings.alert_new_episodes"), description = AppStrings.t(lang, "settings.alert_new_episodes_desc"), value = model.alertNewEpisodes) {
            onAction(SettingsAction.NotificationsChanged(model.copy(alertNewEpisodes = it)))
        }
    }
}
