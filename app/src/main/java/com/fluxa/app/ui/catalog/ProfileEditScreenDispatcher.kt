package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.UserProfile

import androidx.compose.runtime.Composable

@Composable
fun ProfileEditScreen(
    initialProfile: UserProfile? = null,
    onSave: (UserProfile) -> Unit,
    onDelete: ((UserProfile) -> Unit)? = null,
    onCancel: () -> Unit
) {
    if (LocalDeviceType.current == DeviceType.Mobile) {
        MobileProfileEditScreen(initialProfile, onSave, onDelete, onCancel)
    } else {
        TvProfileEditScreen(initialProfile, onSave, onDelete, onCancel)
    }
}
