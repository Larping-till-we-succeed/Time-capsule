package dev.timecapsule.app

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.NavigationRailItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import dev.timecapsule.app.data.AppContainer
import dev.timecapsule.app.ui.LocalAppContainer
import dev.timecapsule.app.ui.navigation.TimeCapsuleDestination
import dev.timecapsule.app.ui.screens.compare.CompareScreen
import dev.timecapsule.app.ui.screens.dashboard.DashboardScreen
import dev.timecapsule.app.ui.screens.settings.SettingsScreen
import dev.timecapsule.app.ui.screens.statistics.StatisticsScreen
import dev.timecapsule.app.ui.screens.timeline.TimelineScreen
import dev.timecapsule.app.ui.theme.TimeCapsuleTheme

@Composable
fun TimeCapsuleApp(container: AppContainer) {
    androidx.compose.runtime.CompositionLocalProvider(LocalAppContainer provides container) {
        TimeCapsuleTheme {
            val navController = rememberNavController()
            val backStackEntry by navController.currentBackStackEntryAsState()
            val current = TimeCapsuleDestination.fromRoute(backStackEntry?.destination?.route)

            val onSelect: (TimeCapsuleDestination) -> Unit = { destination ->
                if (destination != current) {
                    navController.navigate(destination.route) {
                        popUpTo(navController.graph.startDestinationId) { saveState = true }
                        launchSingleTop = true
                        restoreState = true
                    }
                }
            }

            BoxWithConstraints(Modifier.fillMaxSize()) {
                val wide = maxWidth >= 840.dp
                Scaffold(
                    containerColor = MaterialTheme.colorScheme.background,
                    bottomBar = { if (!wide) AppNavigationBar(current, onSelect) },
                    contentWindowInsets = WindowInsets(0, 0, 0, 0),
                ) { innerPadding ->
                    Row(Modifier.fillMaxSize().padding(innerPadding)) {
                        if (wide) {
                            AppNavigationRail(current, onSelect)
                        }
                        TimeCapsuleNavHost(navController, Modifier.weight(1f))
                    }
                }
            }
        }
    }
}

@Composable
private fun AppNavigationBar(
    current: TimeCapsuleDestination,
    onSelect: (TimeCapsuleDestination) -> Unit,
) {
    NavigationBar(
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
    ) {
        TimeCapsuleDestination.entries.forEach { destination ->
            NavigationBarItem(
                selected = destination == current,
                onClick = { onSelect(destination) },
                icon = { Icon(destination.icon, contentDescription = destination.label) },
                label = {
                    Text(text = destination.label, style = MaterialTheme.typography.labelSmall)
                },
                colors = NavigationBarItemDefaults.colors(
                    selectedIconColor = MaterialTheme.colorScheme.primary,
                    selectedTextColor = MaterialTheme.colorScheme.primary,
                    indicatorColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
                    unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                ),
            )
        }
    }
}

@Composable
private fun AppNavigationRail(
    current: TimeCapsuleDestination,
    onSelect: (TimeCapsuleDestination) -> Unit,
) {
    NavigationRail(
        containerColor = MaterialTheme.colorScheme.surface,
        windowInsets = WindowInsets(0, 0, 0, 0),
    ) {
        Spacer(Modifier.weight(0.08f))
        Text(
            text = "TC",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(bottom = 12.dp),
        )
        TimeCapsuleDestination.entries.forEach { destination ->
            NavigationRailItem(
                selected = destination == current,
                onClick = { onSelect(destination) },
                icon = { Icon(destination.icon, contentDescription = destination.label) },
                label = {
                    Text(text = destination.label, style = MaterialTheme.typography.labelSmall)
                },
                alwaysShowLabel = true,
                colors = NavigationRailItemDefaults.colors(
                    selectedIconColor = MaterialTheme.colorScheme.primary,
                    selectedTextColor = MaterialTheme.colorScheme.primary,
                    indicatorColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
                    unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                ),
            )
        }
        Spacer(Modifier.weight(1f))
    }
}

@Composable
private fun TimeCapsuleNavHost(navController: NavHostController, modifier: Modifier) {
    NavHost(
        navController = navController,
        startDestination = TimeCapsuleDestination.Dashboard.route,
        modifier = modifier,
    ) {
        composable(TimeCapsuleDestination.Dashboard.route) {
            DashboardScreen(onOpenTimeline = { navController.navigate(TimeCapsuleDestination.Timeline.route) })
        }
        composable(TimeCapsuleDestination.Timeline.route) { TimelineScreen() }
        composable(TimeCapsuleDestination.Compare.route) { CompareScreen() }
        composable(TimeCapsuleDestination.Statistics.route) { StatisticsScreen() }
        composable(TimeCapsuleDestination.Settings.route) { SettingsScreen() }
    }
}
