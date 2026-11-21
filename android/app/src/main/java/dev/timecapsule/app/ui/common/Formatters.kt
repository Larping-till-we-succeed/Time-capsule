package dev.timecapsule.app.ui.common

import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale

private val dateTimeFormatter = DateTimeFormatter.ofPattern("MMM d, yyyy · HH:mm", Locale.US)
private val dateFormatter = DateTimeFormatter.ofPattern("MMM d, yyyy", Locale.US)
private val dayFormatter = DateTimeFormatter.ofPattern("EEEE, MMM d", Locale.US)
private val shortDayFormatter = DateTimeFormatter.ofPattern("MMM d", Locale.US)

/** Compact, binary-unit byte formatting ("46.0 MB"). */
fun formatBytes(bytes: Long): String {
    if (bytes < 0) return "0 B"
    if (bytes < 1024) return "$bytes B"
    val units = arrayOf("KB", "MB", "GB", "TB", "PB")
    var value = bytes.toDouble() / 1024.0
    var unitIndex = 0
    while (value >= 1024.0 && unitIndex < units.lastIndex) {
        value /= 1024.0
        unitIndex++
    }
    val pattern = if (value >= 100) "%.0f %s" else "%.1f %s"
    return String.format(Locale.US, pattern, value, units[unitIndex])
}

/** Coarse relative time for lists ("just now", "6h ago", "3w ago"). */
fun formatRelativeTime(then: Instant, now: Instant = Instant.now()): String {
    val seconds = Duration.between(then, now).seconds.coerceAtLeast(0)
    return when {
        seconds < 45 -> "just now"
        seconds < 3_600 -> "${(seconds / 60).coerceAtLeast(1)}m ago"
        seconds < 86_400 -> "${seconds / 3_600}h ago"
        seconds < 604_800 -> "${seconds / 86_400}d ago"
        seconds < 2_592_000 -> "${seconds / 604_800}w ago"
        else -> "${seconds / 2_592_000}mo ago"
    }
}

fun formatDateTime(instant: Instant, zone: ZoneId = ZoneId.systemDefault()): String =
    dateTimeFormatter.format(instant.atZone(zone))

fun formatDate(instant: Instant, zone: ZoneId = ZoneId.systemDefault()): String =
    dateFormatter.format(instant.atZone(zone))

fun formatDayHeader(date: LocalDate, today: LocalDate = LocalDate.now()): String = when {
    date == today -> "Today"
    date == today.minusDays(1) -> "Yesterday"
    date.year == today.year -> dayFormatter.format(date)
    else -> dateFormatter.format(date)
}

fun formatShortDate(date: LocalDate): String = shortDayFormatter.format(date)

fun formatInterval(minutes: Int): String = when (minutes) {
    60 -> "Every hour"
    120 -> "Every 2 hours"
    else -> "Every $minutes minutes"
}

fun formatRetention(days: Int): String =
    if (days <= 0) "Keep every snapshot" else "Keep $days days of history"

fun pluralize(count: Int, singular: String, plural: String = "${singular}s"): String =
    "$count ${if (count == 1) singular else plural}"

/** Message safe to surface in an error state. */
fun Throwable.displayMessage(): String =
    message?.takeIf { it.isNotBlank() } ?: "Something went wrong while reading snapshot data."
