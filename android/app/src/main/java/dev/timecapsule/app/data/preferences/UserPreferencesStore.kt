package dev.timecapsule.app.data.preferences

import android.content.Context
import dev.timecapsule.app.data.model.AppPreferences
import dev.timecapsule.app.data.model.ProjectId
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Persists UI configuration with SharedPreferences.
 *
 * The values are genuinely stored, but they are not consumed by any engine yet: automatic
 * snapshotting is performed by the Rust core and that integration does not exist. The Settings
 * screen marks these controls accordingly.
 */
class UserPreferencesStore(context: Context) {

    private val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    private val _preferences = MutableStateFlow(read())
    val preferences: StateFlow<AppPreferences> = _preferences.asStateFlow()

    fun setAutomaticSnapshots(enabled: Boolean) = update { it.copy(automaticSnapshots = enabled) }

    fun setSnapshotInterval(minutes: Int) = update { it.copy(intervalMinutes = minutes) }

    fun setSnapshotOnLaunch(enabled: Boolean) = update { it.copy(snapshotOnLaunch = enabled) }

    fun setRetentionDays(days: Int) = update { it.copy(retentionDays = days) }

    private fun update(transform: (AppPreferences) -> AppPreferences) {
        val next = transform(_preferences.value)
        prefs.edit()
            .putBoolean(KEY_AUTOMATIC, next.automaticSnapshots)
            .putInt(KEY_INTERVAL, next.intervalMinutes)
            .putBoolean(KEY_ON_LAUNCH, next.snapshotOnLaunch)
            .putInt(KEY_RETENTION, next.retentionDays)
            .apply()
        _preferences.value = next
    }

    private fun read() = AppPreferences(
        automaticSnapshots = prefs.getBoolean(KEY_AUTOMATIC, false),
        intervalMinutes = prefs.getInt(KEY_INTERVAL, 30),
        snapshotOnLaunch = prefs.getBoolean(KEY_ON_LAUNCH, false),
        retentionDays = prefs.getInt(KEY_RETENTION, 30),
    )

    private companion object {
        const val PREFS_NAME = "timecapsule.preferences"
        const val KEY_AUTOMATIC = "automatic_snapshots"
        const val KEY_INTERVAL = "snapshot_interval_minutes"
        const val KEY_ON_LAUNCH = "snapshot_on_launch"
        const val KEY_RETENTION = "retention_days"
    }
}

/**
 * The project the user is currently looking at, shared by every screen.
 *
 * Persisted so the same project is selected after a restart. Only an opaque id is stored; the
 * repository remains the source of truth for whether that project still exists.
 */
class SelectedProjectStore(context: Context) {

    private val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    private val _selectedProjectId = MutableStateFlow(
        prefs.getString(KEY_SELECTED, null)?.let(::ProjectId),
    )
    val selectedProjectId: StateFlow<ProjectId?> = _selectedProjectId.asStateFlow()

    fun select(projectId: ProjectId?) {
        prefs.edit().apply {
            if (projectId == null) remove(KEY_SELECTED) else putString(KEY_SELECTED, projectId.value)
        }.apply()
        _selectedProjectId.value = projectId
    }

    private companion object {
        const val PREFS_NAME = "timecapsule.session"
        const val KEY_SELECTED = "selected_project_id"
    }
}
