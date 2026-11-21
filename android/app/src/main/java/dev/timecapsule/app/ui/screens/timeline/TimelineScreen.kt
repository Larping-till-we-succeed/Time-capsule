package dev.timecapsule.app.ui.screens.timeline

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.ui.appViewModel
import dev.timecapsule.app.ui.common.AppHeader
import dev.timecapsule.app.ui.common.BranchChip
import dev.timecapsule.app.ui.common.EmptyState
import dev.timecapsule.app.ui.common.ErrorState
import dev.timecapsule.app.ui.common.FilterPills
import dev.timecapsule.app.ui.common.KeyValueRow
import dev.timecapsule.app.ui.common.LoadingState
import dev.timecapsule.app.ui.common.Panel
import dev.timecapsule.app.ui.common.ProjectSelector
import dev.timecapsule.app.ui.common.ScreenLayout
import dev.timecapsule.app.ui.common.formatBytes
import dev.timecapsule.app.ui.common.formatDateTime
import dev.timecapsule.app.ui.common.formatDayHeader
import dev.timecapsule.app.ui.common.formatRelativeTime
import dev.timecapsule.app.ui.common.pluralize
import dev.timecapsule.app.ui.theme.LocalTimeCapsuleColors
import java.time.LocalDate
import java.time.ZoneId

@Composable
fun TimelineScreen(modifier: Modifier = Modifier) {
    val viewModel = appViewModel { TimelineViewModel(it) }
    val state by viewModel.state.collectAsState()

    TimelineContent(
        state = state,
        modifier = modifier,
        onSelectProject = viewModel::selectProject,
        onSelectBranch = viewModel::selectBranch,
        onToggleExpanded = viewModel::toggleExpanded,
        onRetry = viewModel::retry,
    )
}

@Composable
private fun TimelineContent(
    state: TimelineUiState,
    modifier: Modifier = Modifier,
    onSelectProject: (dev.timecapsule.app.data.model.ProjectId) -> Unit,
    onSelectBranch: (String?) -> Unit,
    onToggleExpanded: (dev.timecapsule.app.data.model.SnapshotId) -> Unit,
    onRetry: () -> Unit,
) {
    val project = state.project
    val errorMessage = state.error
    val visible = state.visibleSnapshots
    val today = remember { LocalDate.now() }
    val zone = remember { ZoneId.systemDefault() }

    ScreenLayout(modifier = modifier) {
        item {
            AppHeader(
                title = "Timeline",
                overline = project?.name ?: "History",
                subtitle = if (project == null) {
                    "Snapshot history for the selected project."
                } else {
                    "${pluralize(project.snapshotCount, "snapshot")} captured · newest first"
                },
            )
        }

        if (state.projects.isNotEmpty()) {
            item {
                ProjectSelector(
                    projects = state.projects,
                    selected = project,
                    onSelect = onSelectProject,
                )
            }
        }

        when {
            errorMessage != null -> item { ErrorState(message = errorMessage, onRetry = onRetry) }

            state.isLoading -> item { LoadingState() }

            project == null -> item {
                EmptyState(
                    title = "No project selected",
                    message = "Pick a project on the Dashboard to browse its snapshots.",
                )
            }

            state.snapshots.isEmpty() -> item {
                EmptyState(
                    title = "No snapshots yet",
                    message = "This project has no captured history. Snapshot creation is performed " +
                        "by the Rust engine, which is not integrated in this build.",
                )
            }

            else -> {
                item {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .horizontalScroll(rememberScrollState()),
                    ) {
                        FilterPills(
                            options = state.branches.map { it.name },
                            selected = state.branchFilter,
                            onSelect = onSelectBranch,
                        )
                    }
                }

                if (visible.isEmpty()) {
                    item {
                        EmptyState(
                            title = "No snapshots on this branch",
                            message = "Choose another branch or show the full history.",
                            actionLabel = "Show all branches",
                            onAction = { onSelectBranch(null) },
                        )
                    }
                } else {
                    val groups = visible.groupBy { it.createdAt.atZone(zone).toLocalDate() }
                    groups.forEach { (date, snapshots) ->
                        item(key = "day-$date") { DayHeader(date = date, count = snapshots.size, today = today) }
                        items(
                            count = snapshots.size,
                            key = { index -> snapshots[index].id.value },
                        ) { index ->
                            val snapshot = snapshots[index]
                            SnapshotHistoryCard(
                                snapshot = snapshot,
                                expanded = snapshot.id in state.expanded,
                                onToggle = { onToggleExpanded(snapshot.id) },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun DayHeader(date: LocalDate, count: Int, today: LocalDate) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(7.dp)
                .background(MaterialTheme.colorScheme.primary, CircleShape),
        )
        Spacer(Modifier.width(10.dp))
        Text(
            text = formatDayHeader(date, today),
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(Modifier.width(10.dp))
        HorizontalDivider(
            modifier = Modifier.weight(1f),
            color = MaterialTheme.colorScheme.outlineVariant,
        )
        Spacer(Modifier.width(10.dp))
        Text(
            text = pluralize(count, "snapshot"),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun SnapshotHistoryCard(snapshot: Snapshot, expanded: Boolean, onToggle: () -> Unit) {
    val colors = LocalTimeCapsuleColors.current
    Panel(
        modifier = Modifier.clickable(onClick = onToggle),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 14.dp),
    ) {
        Row(verticalAlignment = Alignment.Top) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = snapshot.title,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(Modifier.height(3.dp))
                Text(
                    text = "${formatRelativeTime(snapshot.createdAt)} · ${formatDateTime(snapshot.createdAt)}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(Modifier.width(12.dp))
            BranchChip(snapshot.branch)
        }

        if (snapshot.note.isNotBlank()) {
            Spacer(Modifier.height(10.dp))
            Text(
                text = snapshot.note,
                style = MaterialTheme.typography.bodySmall,
                color = colors.textMuted,
                maxLines = if (expanded) Int.MAX_VALUE else 2,
                overflow = TextOverflow.Ellipsis,
            )
        }

        Spacer(Modifier.height(10.dp))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = "${pluralize(snapshot.fileCount, "file")} · ${formatBytes(snapshot.sizeBytes)}",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.weight(1f))
            TextButton(onClick = onToggle, contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp)) {
                Text(if (expanded) "Hide details" else "Details")
            }
        }

        if (expanded) {
            Spacer(Modifier.height(10.dp))
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Spacer(Modifier.height(12.dp))
            KeyValueRow(label = "Snapshot id", value = snapshot.id.value, mono = true)
            Spacer(Modifier.height(8.dp))
            KeyValueRow(label = "Branch", value = snapshot.branch)
            Spacer(Modifier.height(8.dp))
            KeyValueRow(label = "Captured", value = formatDateTime(snapshot.createdAt))
            Spacer(Modifier.height(8.dp))
            KeyValueRow(
                label = "Contents",
                value = "${pluralize(snapshot.fileCount, "file")} · ${formatBytes(snapshot.sizeBytes)}",
            )
            Spacer(Modifier.height(10.dp))
            Text(
                text = "Restoring this snapshot is not available: the Rust core is not integrated. " +
                    "This row is read-only demo data.",
                style = MaterialTheme.typography.bodySmall,
                color = colors.textFaint,
            )
        }
    }
}
