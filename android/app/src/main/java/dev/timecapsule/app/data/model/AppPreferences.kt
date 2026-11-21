package dev.timecapsule.app.data.model

/**
 * User-facing configuration for automatic snapshots.
 *
 * These values are persisted on the device, but nothing consumes them yet: snapshot creation
 * is performed by the Rust core and that integration does not exist. The Settings screen says
 * so explicitly rather than implying the automation runs.
 */
data class AppPreferences(
    val automaticSnapshots: Boolean = false,
    val intervalMinutes: Int = 30,
    val snapshotOnLaunch: Boolean = false,
    val retentionDays: Int = 30,
) {
    companion object {
        val INTERVAL_CHOICES = listOf(5, 15, 30, 60, 120)
        val RETENTION_CHOICES = listOf(7, 14, 30, 90, 0)
    }
}
