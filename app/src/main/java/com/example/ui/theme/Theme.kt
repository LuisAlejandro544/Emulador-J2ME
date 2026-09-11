package com.example.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val RetroDarkColorScheme = darkColorScheme(
    primary = RetroCyanPrimary,
    onPrimary = Color(0xFF00363D),
    primaryContainer = RetroCyanContainer,
    onPrimaryContainer = RetroCyanOnContainer,

    secondary = RetroAmberAccent,
    onSecondary = Color.Black,
    secondaryContainer = RetroAmberContainer,
    onSecondaryContainer = Color(0xFFFFDDB3),

    tertiary = RetroGreenSuccess,
    onTertiary = Color.White,

    background = RetroNavyDark,
    onBackground = RetroTextPrimary,

    surface = RetroNavyDark,
    onSurface = RetroTextPrimary,
    surfaceVariant = RetroNavyCard,
    onSurfaceVariant = RetroTextSecondary,

    outline = RetroNavyCardBorder,
    error = RetroRedDanger,
    errorContainer = RetroRedContainer
)

private val RetroLightColorScheme = lightColorScheme(
    primary = Color(0xFF006874),
    onPrimary = Color.White,
    primaryContainer = Color(0xFF97F0FF),
    onPrimaryContainer = Color(0xFF001F24),

    secondary = Color(0xFF825500),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFFFDDB3),
    onSecondaryContainer = Color(0xFF291800),

    tertiary = Color(0xFF006D44),
    onTertiary = Color.White,

    background = Color(0xFFF8FAFC),
    onBackground = Color(0xFF0F172A),

    surface = Color(0xFFFFFFFF),
    onSurface = Color(0xFF0F172A),
    surfaceVariant = Color(0xFFF1F5F9),
    onSurfaceVariant = Color(0xFF475569),

    outline = Color(0xFFCBD5E1),
    error = RetroRedDanger,
    errorContainer = Color(0xFFFFDAD6)
)

/**
 * Tema principal para el Emulador J2ME.
 * Proporciona un estilo oscuro inmersivo por defecto para emulación y estética retro.
 */
@Composable
fun J2meEmulatorTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    // Por tratarse de un emulador retro, favorecemos el esquema inmersivo RetroDark
    val colorScheme = if (darkTheme) RetroDarkColorScheme else RetroDarkColorScheme

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}

// Alias para compatibilidad
@Composable
fun MyApplicationTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    J2meEmulatorTheme(darkTheme = darkTheme, content = content)
}
