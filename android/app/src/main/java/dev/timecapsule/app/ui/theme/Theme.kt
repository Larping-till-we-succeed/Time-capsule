package dev.timecapsule.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

private val TimeCapsuleColorScheme = darkColorScheme(
    primary = Signal,
    onPrimary = Color(0xFF04211D),
    primaryContainer = SignalDeep,
    onPrimaryContainer = SignalMuted,
    secondary = BranchViolet,
    onSecondary = Color(0xFF151B3A),
    secondaryContainer = BranchVioletDeep,
    onSecondaryContainer = BranchViolet,
    tertiary = ModifiedAmber,
    onTertiary = Color(0xFF2A1D00),
    background = Ink,
    onBackground = FogStrong,
    surface = InkSurface,
    onSurface = FogStrong,
    surfaceVariant = InkSurfaceHigh,
    onSurfaceVariant = FogMuted,
    surfaceContainerLowest = Ink,
    surfaceContainerLow = InkSurface,
    surfaceContainer = InkSurfaceHigh,
    surfaceContainerHigh = InkSurfaceHighest,
    surfaceContainerHighest = Color(0xFF232B3A),
    error = DeletedRed,
    onError = Color(0xFF2A0A0A),
    errorContainer = Color(0xFF421616),
    onErrorContainer = Color(0xFFFEB4B4),
    outline = InkOutline,
    outlineVariant = InkOutlineMuted,
    inverseSurface = FogStrong,
    inverseOnSurface = Ink,
    scrim = Color(0xFF000000),
)

/**
 * Extra semantic colors that Material 3 does not model, exposed through a CompositionLocal so
 * components do not hard-code hex values in multiple places.
 */
@Immutable
data class TimeCapsuleSemanticColors(
    val added: Color = AddedGreen,
    val modified: Color = ModifiedAmber,
    val deleted: Color = DeletedRed,
    val branch: Color = BranchViolet,
    val accent: Color = Signal,
    val textFaint: Color = FogFaint,
    val textMuted: Color = FogMuted,
    val gridLine: Color = InkOutlineMuted,
)

val LocalTimeCapsuleColors = staticCompositionLocalOf { TimeCapsuleSemanticColors() }

/**
 * The application theme.
 *
 * TimeCapsule is a dark-first developer tool, so only the dark scheme ships. A light scheme is
 * deliberately absent rather than approximated; Settings states this to the user.
 */
@Composable
fun TimeCapsuleTheme(content: @Composable () -> Unit) {
    androidx.compose.runtime.CompositionLocalProvider(
        LocalTimeCapsuleColors provides TimeCapsuleSemanticColors(),
    ) {
        MaterialTheme(
            colorScheme = TimeCapsuleColorScheme,
            typography = TimeCapsuleTypography,
            content = content,
        )
    }
}
