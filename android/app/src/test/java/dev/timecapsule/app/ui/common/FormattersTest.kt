package dev.timecapsule.app.ui.common

import java.time.Duration
import java.time.Instant
import java.time.ZoneOffset
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class FormattersTest {

    @Test
    fun formatBytes_zeroAndKilobytes() {
        assertEquals("0 B", formatBytes(0))
        assertEquals("1.0 KB", formatBytes(1024))
    }

    @Test
    fun formatBytes_megabytes() {
        val formatted = formatBytes(48_213_760L)
        assertTrue(formatted.endsWith("MB"))
    }

    @Test
    fun formatRelativeTime_hours() {
        val now = Instant.parse("2026-11-21T12:00:00Z")
        val then = now.minus(Duration.ofHours(2))
        assertEquals("2h ago", formatRelativeTime(then, now))
    }

    @Test
    fun formatRelativeTime_justNow() {
        val now = Instant.parse("2026-11-21T12:00:00Z")
        assertEquals("just now", formatRelativeTime(now, now))
    }

    @Test
    fun formatInterval_andRetention() {
        assertEquals("Every 30 minutes", formatInterval(30))
        assertEquals("Every hour", formatInterval(60))
        assertEquals("Keep every snapshot", formatRetention(0))
        assertEquals("Keep 7 days of history", formatRetention(7))
    }

    @Test
    fun formatDateTime_usesProvidedZone() {
        val instant = Instant.parse("2026-11-21T12:00:00Z")
        assertEquals("Nov 21, 2026 · 12:00", formatDateTime(instant, ZoneOffset.UTC))
    }
}
