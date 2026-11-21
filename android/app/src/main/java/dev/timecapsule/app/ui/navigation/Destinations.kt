package dev.timecapsule.app.ui.navigation

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.ui.graphics.vector.ImageVector

/** Top-level app destinations. Routes are plain strings so no serialization plugin is needed. */
enum class TimeCapsuleDestination(
    val route: String,
    val label: String,
    val icon: ImageVector,
    val description: String,
) {
    Dashboard(
        route = "dashboard",
        label = "Dashboard",
        icon = Icons.Filled.Home,
        description = "Project overview and recent activity",
    ),
    Timeline(
        route = "timeline",
        label = "Timeline",
        icon = Icons.Filled.DateRange,
        description = "Chronological snapshot history",
    ),
    Compare(
        route = "compare",
        label = "Compare",
        icon = Icons.Filled.Search,
        description = "Changes between two snapshots",
    ),
    Statistics(
        route = "statistics",
        label = "Statistics",
        icon = Icons.Filled.List,
        description = "Snapshot, file and storage totals",
    ),
    Settings(
        route = "settings",
        label = "Settings",
        icon = Icons.Filled.Settings,
        description = "Preferences, storage access and engine status",
    );

    companion object {
        fun fromRoute(route: String?): TimeCapsuleDestination =
            entries.firstOrNull { it.route == route } ?: Dashboard
    }
}
