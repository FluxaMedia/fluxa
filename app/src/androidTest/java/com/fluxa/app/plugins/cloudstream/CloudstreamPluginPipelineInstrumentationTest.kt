package com.fluxa.app.plugins.cloudstream

import android.util.Base64
import androidx.test.ext.junit.runners.AndroidJUnit4
import java.security.KeyPairGenerator
import java.security.Signature
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class CloudstreamPluginPipelineInstrumentationTest {
    @Test
    fun verifiesManifestPayloadWithPublisherKey() {
        val pair = KeyPairGenerator.getInstance("Ed25519").generateKeyPair()
        val payload = "{\"name\":\"Signed repository\",\"plugins\":[]}".encodeToByteArray()
        val signer = Signature.getInstance("Ed25519")
        signer.initSign(pair.private)
        signer.update(payload)
        val signature = Base64.encodeToString(signer.sign(), Base64.NO_WRAP)
        val publisherPublicKey = Base64.encodeToString(pair.public.encoded, Base64.NO_WRAP)

        assertTrue(verifyEd25519Signature(payload, signature, publisherPublicKey))
    }

    @Test
    fun rejectsModifiedManifestPayload() {
        val pair = KeyPairGenerator.getInstance("Ed25519").generateKeyPair()
        val payload = "{\"name\":\"Signed repository\",\"plugins\":[]}".encodeToByteArray()
        val signer = Signature.getInstance("Ed25519")
        signer.initSign(pair.private)
        signer.update(payload)
        val signature = Base64.encodeToString(signer.sign(), Base64.NO_WRAP)
        val publisherPublicKey = Base64.encodeToString(pair.public.encoded, Base64.NO_WRAP)

        assertFalse(verifyEd25519Signature(payload + byteArrayOf(0x20), signature, publisherPublicKey))
    }
}
