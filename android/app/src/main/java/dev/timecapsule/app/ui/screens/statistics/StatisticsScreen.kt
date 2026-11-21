package dev.timecapsule.app.ui.screens.statistics

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.timecapsule.app.data.model.Branch
import dev.timecapsule.app.data.model.DailyActivity
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.ProjectStatistics
import dev.timecapsule.app.ui.appViewModel
import dev.timecapsule.app.ui.common.AppHeader
import dev.timecapsule.app.ui.common.BranchChip
import dev.timecapsule.app.ui.common.EmptyState
import dev.timecapsule.app.ui.common.ErrorState
import dev.timecapsule.app.ui.common.LoadingState
import dev.timecapsule.app.ui.common.Panel
import dev.timecapsule.app.ui.common.ProjectSelector
import dev.timecapsule.app.ui.common.ResponsiveGrid
import dev.timecapsule.app.ui.common.ScreenLayout
import dev.timecapsule.app.ui.common.SectionHeader
import dev.timecapsule.app.ui.common.StatTile
import dev.timecapsule.app.ui.common.formatBytes
import dev.timecapsule.app.ui.common.formatDate
import dev.timecapsule.app.ui.common.formatRelativeTime
import dev.timecapsule.app.ui.common.formatShortDate
import dev.timecapsule.app.ui.common.pluralize
import dev.timecapsule.app.ui.theme.LocalTimeCapsuleColors
import java.time.Duration

@Composable
fun StatisticsScreen(modifier: Modifier = Modifier) {
    val viewModel = appViewModel { StatisticsViewModel(it) }
    val state by viewModel.state.collectAsState()

    StatisticsContent(
        state = state,
        modifier = modifier,
        onSelectProject = viewModel::selectProject,
        onRetry = viewModel::retry,
    )
}

@Composable
private fun StatisticsContent(
    state: StatisticsUiState,
    modifier: Modifier = Modifier,
    onSelectProject: (ProjectId) -> Unit,
    onRetry: () -> Unit,
) {
    val statistics = state.statistics
    val errorMessage = state.error

    ScreenLayout(modifier = modifier) {
        item {
            AppHeader(
                title = "Statistics",
                overline = state.project?.name ?: "Totals",
                subtitle = "What this project has accumulated so far.",
            )
        }

        if (state.projects.isNotEmpty()) {
            item {
                ProjectSelector(
                    projects = state.projects,
                    selected = state.project,
                    onSelect = onSelectProject,
                )
            }
        }

        when {
            errorMessage != null -> item { ErrorState(message = errorMessage, onRetry = onRetry) }

            state.isLoading -> item { LoadingState(label = "Aggregating snapshot data…") }

            statistics == null -> item {
                EmptyState(
                    title = "No statistics yet",
                    message = "Select a project to aggregate its snapshot history.",
                )
            }

            statistics.snapshotCount == 0 -> item {
                EmptyState(
                    title = "Nothing captured yet",
                    message = "Statistics appear once this project has snapshots. Capturing snapshots " +
                        "requires the Rust engine, which is not integrated in this build.",
                )
            }

            else -> {
                item {
                    ResponsiveGrid(
                        items = listOf(
                            Metric("Snapshots", statistics.snapshotCount.toString(), "in total"),
                            Metric("Tracked files", statistics.fileCount.toString(), "in latest snapshot"),
                            Metric("Storage", formatBytes(statistics.storageBytes), "snapshot data on disk"),
                            Metric("Branches", statistics.branchCount.toString(), "lines of history"),
                        ),
                        columnsFor = { width -> if (width >= 720.dp) 4 else 2 },
                    ) { metric ->
                        StatTile(label = metric.label, value = metric.value, detail = metric.detail)
                    }
                }

                item {
                    SectionHeader(
                        title = "Snapshot activity",
                        subtitle = "Last 14 days",
                    )
                }

                item { ActivityPanel(statistics.activity) }

                item {
                    SectionHeader(
                        title = "History span",
                        subtitle = "First and most recent capture",
                    )
                }

                item { HistorySpanPanel(statistics) }

                item {
                    SectionHeader(
                        title = "Branches",
                        subtitle = pluralize(statistics.branchCount, "branch", "branches"),
                    )
                }

                item {
                    Panel {
                        statistics.branches.forEachIndexed { index, branch ->
                            if (index > 0) {
                                HorizontalDivider(
                                    modifier = Modifier.padding(vertical = 10.dp),
                                    color = MaterialTheme.colorScheme.outlineVariant,
                                )
                            }
                            BranchRow(branch)
                        }
                    }
                }
            }
        }
    }
}

private data class Metric(val label: String, val value: String, val detail: String)

@Composable
private fun ActivityPanel(activity: List<DailyActivity>) {
    val total = activity.sumOf { it.snapshotCount }
    val busiest = activity.maxByOrNull { it.snapshotCount }

    Panel {
        Row(verticalAlignment = Alignment.Bottom) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "$total snapshots captured",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(Modifier.height(3.dp))
                Text(
                    text = if (busiest == null || busiest.snapshotCount == 0) {
                        "No activity in this window."
                    } else {
                        "Busiest day: ${formatShortDate(busiest.date)} · ${pluralize(busiest.snapshotCount, "snapshot")}"
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        Spacer(Modifier.height(18.dp))
        ActivityChart(activity = activity)

        if (activity.isNotEmpty()) {
            Spacer(Modifier.height(8.dp))
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text(
                    text = formatShortDate(activity.first().date),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = formatShortDate(activity[activity.size / 2].date),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = formatShortDate(activity.last().date),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun ActivityChart(activity: List<DailyActivity>) {
    val semantic = LocalTimeCapsuleColors.current
    val barColor = MaterialTheme.colorScheme.primary
    val emptyColor = semantic.gridLine

    Canvas(
        modifier = Modifier
            .fillMaxWidth()
            .height(132.dp),
    ) {
        if (activity.isEmpty()) return@Canvas

        val maxCount = activity.maxOf { it.snapshotCount }.coerceAtLeast(1)
        val slotWidth = size.width / activity.size
        val barWidth = slotWidth * 0.52f
        val usableHeight = size.height - 2.dp.toPx()

        activity.forEachIndexed { index, day ->
            val heightFraction = day.snapshotCount.toFloat() / maxCount
            val barHeight = if (day.snapshotCount == 0) {
                2.dp.toPx()
            } else {
                (usableHeight * heightFraction).coerceAtLeast(8.dp.toPx())
            }
            drawRoundRect(
                color = if (day.snapshotCount == 0) emptyColor else barColor,
                topLeft = Offset(
                    x = index * slotWidth + (slotWidth - barWidth) / 2f,
                    y = size.height - barHeight,
                ),
                size = Size(barWidth, barHeight),
                cornerRadius = CornerRadius(3.dp.toPx()),
            )
        }
    }
}

@Composable
private fun HistorySpanPanel(statistics: ProjectStatistics) {
    val earliest = statistics.earliestSnapshotAt
    val latest = statistics.latestSnapshotAt

    Panel {
        if (earliest == null || latest == null) {
            Text(
                text = "No captures recorded.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            return@Panel
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            SpanPoint(label = "First snapshot", value = formatDate(earliest), modifier = Modifier.weight(1f))
            Text(
                text = "→",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 10.dp),
            )
            SpanPoint(
                label = "Latest snapshot",
                value = formatDate(latest),
                modifier = Modifier.weight(1f),
                alignEnd = true,
            )
        }
        Spacer(Modifier.height(14.dp))
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        Spacer(Modifier.height(12.dp))
        Text(
            text = "Span of ${Duration.between(earliest, latest).toDays()} days · " +
                "latest activity ${formatRelativeTime(latest)}",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun SpanPoint(
    label: String,
    value: String,
    modifier: Modifier = Modifier,
    alignEnd: Boolean = false,
) {
    Column(
        modifier = modifier,
        horizontalAlignment = if (alignEnd) Alignment.End else Alignment.Start,
    ) {
        Text(
            text = label.uppercase(),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(4.dp))
        Text(
            text = value,
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun BranchRow(branch: Branch) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(7.dp)
                .background(
                    color = if (branch.isActive) {
                        LocalTimeCapsuleColors.current.branch
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
                    },
                    shape = CircleShape,
                ),
        )
        Spacer(Modifier.width(10.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = branch.name,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = pluralize(branch.snapshotCount, "snapshot") +
                    (branch.lastSnapshotAt?.let { " · ${formatRelativeTime(it)}" } ?: ""),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        if (branch.isActive) {
            BranchChip("active")
        }
    }
}
