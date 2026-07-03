package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.R

@Composable
fun WelcomeScreen(
    onContinueWithNuvio: () -> Unit,
    onLoginWithStremio: () -> Unit,
    onContinueWithoutAccount: () -> Unit
) {
    val lang = "en"
    val deviceType = LocalDeviceType.current
    val isTv = deviceType == DeviceType.TV

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.radialGradient(
                    colors = listOf(Color(0xFF221F1F), FluxaColors.backgroundNearBlack),
                    center = androidx.compose.ui.geometry.Offset(0.5f, 0.5f)
                )
            ),
        contentAlignment = Alignment.Center
    ) {
        Column(
            modifier = Modifier.widthIn(max = if (isTv) 480.dp else 400.dp).padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = AppStrings.t(lang, "welcome.headline"),
                color = Color.White,
                fontSize = if (isTv) 40.sp else 30.sp,
                fontWeight = FontWeight.Black,
                textAlign = TextAlign.Center
            )
            Spacer(Modifier.height(12.dp))
            Text(
                text = AppStrings.t(lang, "welcome.subheadline"),
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 15.sp,
                textAlign = TextAlign.Center
            )
            Spacer(Modifier.height(40.dp))

            WelcomeOptionButton(
                label = AppStrings.t(lang, "auth.continue_with_nuvio"),
                iconRes = R.drawable.ic_nuvio,
                containerColor = Color.White,
                contentColor = Color.Black,
                onClick = onContinueWithNuvio
            )
            Spacer(Modifier.height(14.dp))
            WelcomeOptionButton(
                label = AppStrings.t(lang, "auth.continue_with_stremio"),
                iconRes = R.drawable.ic_stremio,
                containerColor = Color.White.copy(alpha = 0.08f),
                contentColor = Color.White,
                onClick = onLoginWithStremio
            )

            Spacer(Modifier.height(28.dp))

            TextButton(onClick = onContinueWithoutAccount) {
                Text(
                    text = AppStrings.t(lang, "auth.continue_without_account"),
                    color = Color.White.copy(alpha = 0.55f),
                    fontSize = 14.sp
                )
            }
        }
    }
}

@Composable
private fun WelcomeOptionButton(
    label: String,
    iconRes: Int,
    containerColor: Color,
    contentColor: Color,
    onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(56.dp),
        shape = RoundedCornerShape(14.dp),
        colors = ButtonDefaults.buttonColors(containerColor = containerColor, contentColor = contentColor)
    ) {
        Image(
            painter = painterResource(id = iconRes),
            contentDescription = null,
            modifier = Modifier.size(22.dp)
        )
        Spacer(Modifier.width(12.dp))
        Text(label, fontWeight = FontWeight.Bold, fontSize = 15.sp)
    }
}
