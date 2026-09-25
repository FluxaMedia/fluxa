package com.fluxa.app.shared.ui

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.VisualTransformation

enum class FluxaButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Destructive,
}

@Composable
fun FluxaButton(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: FluxaButtonVariant = FluxaButtonVariant.Primary,
    enabled: Boolean = true,
) {
    when (variant) {
        FluxaButtonVariant.Ghost -> TextButton(onClick = onClick, modifier = modifier, enabled = enabled) {
            Text(label)
        }
        FluxaButtonVariant.Secondary -> OutlinedButton(onClick = onClick, modifier = modifier, enabled = enabled) {
            Text(label)
        }
        FluxaButtonVariant.Primary -> Button(
            onClick = onClick,
            modifier = modifier,
            enabled = enabled,
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ),
        ) {
            Text(label)
        }
        FluxaButtonVariant.Destructive -> Button(
            onClick = onClick,
            modifier = modifier,
            enabled = enabled,
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.error,
                contentColor = MaterialTheme.colorScheme.onError,
            ),
        ) {
            Text(label)
        }
    }
}

@Composable
fun FluxaIconButton(
    onClick: () -> Unit,
    contentDescription: String,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    IconButton(onClick = onClick, modifier = modifier.semantics { this.contentDescription = contentDescription }) {
        content()
    }
}

@Composable
fun FluxaTextField(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    label: String? = null,
    placeholder: String? = null,
    singleLine: Boolean = true,
    visualTransformation: VisualTransformation = VisualTransformation.None,
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier.fillMaxWidth(),
        label = label?.let { { Text(it) } },
        placeholder = placeholder?.let { { Text(it) } },
        singleLine = singleLine,
        visualTransformation = visualTransformation,
        shape = MaterialTheme.shapes.medium,
    )
}

@Composable
fun FluxaDialog(
    title: String,
    text: String? = null,
    onDismissRequest: () -> Unit,
    confirmLabel: String,
    onConfirm: () -> Unit,
    dismissLabel: String? = null,
    destructive: Boolean = false,
) {
    AlertDialog(
        onDismissRequest = onDismissRequest,
        title = { Text(title) },
        text = text?.let { { Text(it) } },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(confirmLabel, color = if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary)
            }
        },
        dismissButton = dismissLabel?.let {
            { TextButton(onClick = onDismissRequest) { Text(it) } }
        },
    )
}
