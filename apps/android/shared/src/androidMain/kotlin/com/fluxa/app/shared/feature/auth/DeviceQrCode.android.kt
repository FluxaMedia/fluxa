package com.fluxa.app.shared.feature.auth

import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.material3.Text
import com.google.zxing.BarcodeFormat
import com.google.zxing.MultiFormatWriter

@Composable
actual fun DeviceQrCode(payload: String, modifier: Modifier) {
    if (payload.isBlank()) {
        Text("", modifier = modifier)
        return
    }
    val bitmap = remember(payload) {
        val matrix = MultiFormatWriter().encode(payload, BarcodeFormat.QR_CODE, 280, 280)
        android.graphics.Bitmap.createBitmap(280, 280, android.graphics.Bitmap.Config.ARGB_8888).also { output ->
            for (x in 0 until 280) for (y in 0 until 280) output.setPixel(x, y, if (matrix[x, y]) android.graphics.Color.BLACK else android.graphics.Color.WHITE)
        }
    }
    Image(bitmap.asImageBitmap(), contentDescription = null, modifier = modifier)
}
