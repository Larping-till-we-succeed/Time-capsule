package dev.timecapsule.app.ui.screens.dashboard

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.ui.LocalAppContainer
import dev.timecapsule.app.ui.appViewModel
import dev.timecapsule.app.ui.common.AppHeader
import dev.timecapsule.app.ui.common.BranchChip
import dev.timecapsule.app.ui.common.DemoDataBanner
import dev.timecapsule.app.ui.common.EmptyState
import dev.timecapsule.app.ui.common.ErrorState
import dev.timecapsule.app.ui.common.KeyValueRow
import dev.timecapsule.app.ui.common.LoadingState
import dev.timecapsule.app.ui.common.Panel
import dev.timecapsule.app.ui.common.ProjectSelector
import dev.timecapsule.app.ui.common.ResponsiveGrid
import dev.timecapsule.app.ui.common.ScreenLayout
import dev.timecapsule.app.ui.common.SectionHeader
import dev.timecapsule.app.ui.common.StatTile
import dev.timecapsule.app.ui.common.formatBytes
import dev.timecapsule.app.ui.common.formatDateTime
import dev.timecapsule.app.ui.common.formatRelativeTime
import dev.timecapsule.app.ui.common.pluralize
import dev.timecapsule.app.ui.common.rememberProjectDirectoryPicker
import dev.timecapsule.app.ui.theme.LocalTimeCapsuleColors

@Composable
fun DashboardScreen(
    onOpenTimeline: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val viewModel = appViewModel { DashboardViewModel(it) }
    val state by viewModel.state.collectAsState()
    val container = LocalAppContainer.current

    var pickerError by remember { mutableStateOf<String?>(null) }
    val pickDirectory = rememberProjectDirectoryPicker(
        store = container.projectDirectories,
        onPicked = {
            pickerError = null
            viewModel.attachDirectory(it)
        },
        onError = { pickerError = it },
    )

    DashboardContent(
        state = state,
        modifier = modifier,
        onSelectProject = viewModel::selectProject,
        onOpenTimeline = onOpenTimeline,
        onRetry = viewModel::retry,
        onPickDirectory = pickDirectory,
        onDetachDirectory = viewModel::detachDirectory,
        pickerError = pickerError,
    )
}

@Composable
private fun DashboardContent(
    state: DashboardUiState,
    modifier: Modifier = Modifier,
    onSelectProject: (dev.timecapsule.app.data.model.ProjectId) -> Unit,
    onOpenTimeline: () -> Unit,
    onRetry: () -> Unit,
    onPickDirectory: () -> Unit,
    onDetachDirectory: () -> Unit,
    pickerError: String?,
) {
    val project = state.project
    val errorMessage = state.error

    ScreenLayout(modifier = modifier) {
        item {
            AppHeader(
                title = project?.name ?: "Dashboard",
                overline = "Overview",
                subtitle = project?.description
                    ?: "Choose a project to inspect its snapshot history.",
            )
        }

        item { DemoDataBanner() }

        item {
            ProjectSelector(
                projects = state.projects,
                selected = project,
                onSelect = onSelectProject,
            )
        }

        when {
            errorMessage != null -> item { ErrorState(message = errorMessage, onRetry = onRetry) }

            state.isLoading -> item { LoadingState() }

            project == null -> item {
                EmptyState(
                    title = "No projects yet",
                    message = "Projects are registered by the Rust core. Once it is integrated, " +
                        "its projects will appear here.",
                    actionLabel = "Refresh",
                    onAction = onRetry,
                )
            }

            else -> {
                item {
                    ResponsiveGrid(
                        items = listOf(
                            Metric("Snapshots", project.snapshotCount.toString(), "captured in this project"),
                            Metric("Snapshot data", formatBytes(project.storageBytes), "on-disk history size"),
                            Metric(
                                "Active branch",
                                project.activeBranch,
                                if (project.snapshotCount == 0) "no history yet" else "new snapshots land here",
                            ),
                            Metric(
                                "Last captured",
                                project.lastSnapshotAt?.let { formatRelativeTime(it) } ?: "—",
                                project.lastSnapshotAt?.let { formatDateTime(it) } ?: "never",
                            ),
                        ),
                        columnsFor = { width -> if (width >= 720.dp) 4 else 2 },
                    ) { metric ->
                        StatTile(
                            label = metric.label,
                            value = metric.value,
                            detail = metric.detail,
                        )
                    }
                }

                item {
                    SectionHeader(
                        title = "Recent snapshots",
                        subtitle = if (project.snapshotCount == 0) {
                            "Nothing captured yet"
                        } else {
                            "Latest ${state.recentSnapshots.size} of ${project.snapshotCount}"
                        },
                        trailing = {
                            if (state.recentSnapshots.isNotEmpty()) {
                                TextButton(onClick = onOpenTimeline) { Text("Full timeline") }
                            }
                        },
                    )
                }

                if (state.recentSnapshots.isEmpty()) {
                    item {
                        EmptyState(
                            title = "No snapshots in this project",
                            message = "Create a snapshot from the Rust engine once it is integrated; " +
                                "this build cannot capture files.",
                        )
                    }
                } else {
                    items(state.recentSnapshots.size) { index ->
                        RecentSnapshotCard(
                            snapshot = state.recentSnapshots[index],
                            onClick = onOpenTimeline,
                        )
                    }
                }

                item {
                    SectionHeader(
                        title = "Project folder",
                        subtitle = "Access is granted through Android's document picker",
                    )
                }

                item {
                    ProjectFolderPanel(
                        directoryName = state.linkedDirectory?.displayName,
                        directoryUri = state.linkedDirectory?.uri,
                        pickerError = pickerError,
                        onPickDirectory = onPickDirectory,
                        onDetachDirectory = onDetachDirectory,
                    )
                }

                item {
                    SectionHeader(
                        title = "Engine",
                        subtitle = "What is actually backing this screen",
                    )
                }

                item {
                    Panel {
                        KeyValueRow(label = "Data source", value = "In-memory demo data")
                        Spacer(Modifier.height(10.dp))
                        KeyValueRow(label = "Rust core", value = "Not integrated")
                        Spacer(Modifier.height(10.dp))
                        KeyValueRow(label = "Typical latency", value = "Simulated", mono = true)
                    }
                }
            }
        }
    }
}

private data class Metric(val label: String, val value: String, val detail: String)

@Composable
private fun RecentSnapshotCard(snapshot: Snapshot, onClick: () -> Unit) {
    val colors = LocalTimeCapsuleColors.current
    Panel(
        modifier = Modifier.clickable(onClick = onClick),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 14.dp),
    ) {
        Row(verticalAlignment = Alignment.Top) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = snapshot.title,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    text = "${formatRelativeTime(snapshot.createdAt)} · ${formatDateTime(snapshot.createdAt)}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (snapshot.note.isNotBlank()) {
                    Spacer(Modifier.height(8.dp))
                    Text(
                        text = snapshot.note,
                        style = MaterialTheme.typography.bodySmall,
                        color = colors.textFaint,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            Spacer(Modifier.width(12.dp))
            Column(horizontalAlignment = Alignment.End) {
                BranchChip(snapshot.branch)
                Spacer(Modifier.height(8.dp))
                Text(
                    text = pluralize(snapshot.fileCount, "file"),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun ProjectFolderPanel(
    directoryName: String?,
    directoryUri: String?,
    pickerError: String?,
    onPickDirectory: () -> Unit,
    onDetachDirectory: () -> Unit,
) {
    if (directoryName == null) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            EmptyState(
                title = "No folder linked on this device",
                message = "Android does not allow browsing arbitrary paths. Link a project folder " +
                    "with the system picker so the Rust core can be handed access later.",
                actionLabel = "Choose folder",
                onAction = onPickDirectory,
            )
            if (pickerError != null) {
                Text(
                    text = pickerError,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                    modifier = Modifier.padding(horizontal = 4.dp),
                )
            }
        }
        return
    }

    Panel(color = MaterialTheme.colorScheme.surfaceContainerHigh) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Surface(
                color = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
                shape = RoundedCornerShape(10.dp),
            ) {
                Text(
                    text = "SAF",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                )
            }
            Spacer(Modifier.width(10.dp))
            Text(
                text = directoryName,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Spacer(Modifier.height(10.dp))
        Text(
            text = directoryUri.orEmpty(),
            style = MaterialTheme.typography.bodySmall.copy(
                fontFamily = dev.timecapsule.app.ui.theme.MonoTextStyle.fontFamily,
            ),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
        Spacer(Modifier.height(12.dp))
        Text(
            text = "Access is persisted by Android. Linking a folder does not index it: that " +
                "requires the Rust core.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(6.dp))
        Row {
            TextButton(onClick = onPickDirectory) { Text("Change folder") }
            TextButton(onClick = onDetachDirectory) { Text("Unlink") }
        }
    }
}
