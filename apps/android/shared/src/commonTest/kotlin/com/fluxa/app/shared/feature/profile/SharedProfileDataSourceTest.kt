package com.fluxa.app.shared.feature.profile

import com.fluxa.app.common.PinHasher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class SharedProfileDataSourceTest {
    @Test
    fun pinSelectionIsSharedAcrossPlatforms() = runTest {
        val profile = ProfileUiModel("one", "One", null, "en", 0L, hasPin = true)
        val store = FakeProfileStore(profile, PinHasher.hash("1234"))
        val source = SharedProfileDataSource(store)

        source.selectProfile(profile.id)
        assertEquals(profile.id, source.observeProfiles().first().pendingPinProfile?.id)
        source.attemptPin(profile.id, "0000")
        assertTrue(source.observeProfiles().first().pinError)
        source.attemptPin(profile.id, "1234")
        assertFalse(source.observeProfiles().first().pinError)
        assertEquals(profile.id, store.activeId)
    }

    private class FakeProfileStore(
        profile: ProfileUiModel,
        private val hash: String?
    ) : ProfilePersistence {
        private val state = MutableStateFlow(ProfileStoreSnapshot(profiles = listOf(profile)))
        var activeId: String? = null

        override fun observe(): Flow<ProfileStoreSnapshot> = state
        override suspend fun pinHash(profileId: String): String? = hash
        override suspend fun activate(profileId: String) {
            activeId = profileId
            state.value = state.value.copy(activeProfile = state.value.profiles.first { it.id == profileId })
        }
        override suspend fun delete(profileId: String) = Unit
        override suspend fun save(edit: ProfileEditUiModel, pinHash: String?): String = edit.id.orEmpty()
    }
}
