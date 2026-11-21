package dev.timecapsule.app.ui.screens.compare

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.timecapsule.app.data.model.FileChange
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.SnapshotId
import dev.timecapsule.app.ui.appViewModel
import dev.timecapsule.app.ui.common.AppHeader
import dev.timecapsule.app.ui.common.ChangeBadge
import dev.timecapsule.app.ui.common.EmptyState
import dev.timecapsule.app.ui.common.ErrorState
import dev.timecapsule.app.ui.common.LoadingState
import dev.timecapsule.app.ui.common.Panel
import dev.timecapsule.app.ui.common.ProjectSelector
import dev.timecapsule.app.ui.common.ResponsiveGrid
import dev.timecapsule.app.ui.common.ScreenLayout
import dev.timecapsule.app.ui.common.SectionHeader
import dev.timecapsule.app.ui.common.SnapshotSelector
import dev.timecapsule.app.ui.common.StatTile
import dev.timecapsule.app.ui.common.StatusPill
import dev.timecapsule.app.ui.theme.LocalTimeCapsuleColors
import dev.timecapsule.app.ui.theme.MonoTextStyle

@Composable
fun CompareScreen(modifier: Modifier = Modifier) {
    val viewModel = appViewModel { CompareViewModel(it) }
    val state by viewModel.state.collectAsState()

    CompareContent(
        state = state,
        modifier = modifier,
        onSelectProject = viewModel::selectProject,
        onSelectFrom = viewModel::selectFrom,
        onSelectTo = viewModel::selectTo,
        onSwap = viewModel::swap,
        onRetry = viewModel::retry,
    )
}

private data class DiffSummary(
    val label: String,
    val files: Int,
    val additions: Int,
    val deletions: Int,
)

@Composable
private fun CompareContent(
    state: CompareUiState,
    modifier: Modifier = Modifier,
    onSelectProject: (ProjectId) -> Unit,
    onSelectFrom: (SnapshotId) -> Unit,
    onSelectTo: (SnapshotId) -> Unit,
    onSwap: () -> Unit,
    onRetry: () -> Unit,
) {
    val project = state.project
    val errorMessage = state.error
    val diff = state.diff
    val semantic = LocalTimeCapsuleColors.current

    ScreenLayout(modifier = modifier) {
        item {
            AppHeader(
                title = "Compare",
                overline = project?.name ?: "Changes",
                subtitle = "The older snapshot is the baseline; the newer one is compared against it.",
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

            state.isLoading -> item { LoadingState(label = "Loading snapshot history…") }

            project == null -> item {
                EmptyState(
                    title = "No project selected",
                    message = "Select a project on the Dashboard before comparing snapshots.",
                )
            }

            state.snapshots.size < 2 -> item {
                EmptyState(
                    title = "Not enough history",
                    message = "Comparing requires at least two snapshots in this project.",
                )
            }

            else -> {
                item {
                    SectionHeader(
                        title = "Snapshots",
                        subtitle = "Baseline and candidate",
                        trailing = { TextButton(onClick = onSwap) { Text("Swap") } },
                    )
                }

                item {
                    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
                        if (maxWidth >= 640.dp) {
                            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                SnapshotSelector(
                                    label = "Baseline",
                                    snapshots = state.snapshots,
                                    selectedId = state.fromId,
                                    onSelect = onSelectFrom,
                                    modifier = Modifier.weight(1f),
                                )
                                SnapshotSelector(
                                    label = "Compare with",
                                    snapshots = state.snapshots,
                                    selectedId = state.toId,
                                    onSelect = onSelectTo,
                                    modifier = Modifier.weight(1f),
                                )
                            }
                        } else {
                            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                                SnapshotSelector(
                                    label = "Baseline",
                                    snapshots = state.snapshots,
                                    selectedId = state.fromId,
                                    onSelect = onSelectFrom,
                                )
                                SnapshotSelector(
                                    label = "Compare with",
                                    snapshots = state.snapshots,
                                    selectedId = state.toId,
                                    onSelect = onSelectTo,
                                )
                            }
                        }
                    }
                }

                when {
                    state.isDiffLoading -> item { LoadingState(label = "Comparing snapshots…") }

                    diff == null -> item {
                        EmptyState(
                            title = "Nothing to compare",
                            message = state.diffNote
                                ?: "Choose two different snapshots to see what changed.",
                        )
                    }

                    diff.isEmpty -> item {
                        EmptyState(
                            title = "Identical content",
                            message = "These two snapshots contain the same files.",
                        )
                    }

                    else -> {
                        val summaries = listOf(
                            DiffSummary(
                                label = "Added",
                                files = diff.added.size,
                                additions = diff.added.sumOf { it.additions },
                                deletions = diff.added.sumOf { it.deletions },
                            ),
                            DiffSummary(
                                label = "Modified",
                                files = diff.modified.size,
                                additions = diff.modified.sumOf { it.additions },
                                deletions = diff.modified.sumOf { it.deletions },
                            ),
                            DiffSummary(
                                label = "Deleted",
                                files = diff.deleted.size,
                                additions = diff.deleted.sumOf { it.additions },
                                deletions = diff.deleted.sumOf { it.deletions },
                            ),
                        )

                        item {
                            ResponsiveGrid(
                                items = summaries,
                                columnsFor = { width -> if (width >= 560.dp) 3 else 1 },
                            ) { summary ->
                                val accent = when (summary.label) {
                                    "Added" -> semantic.added
                                    "Modified" -> semantic.modified
                                    else -> semantic.deleted
                                }
                                StatTile(
                                    label = summary.label,
                                    value = summary.files.toString(),
                                    detail = lineSummary(summary.additions, summary.deletions),
                                    accent = accent,
                                )
                            }
                        }

                        item { TextualDiffPlaceholder() }

                        item {
                            SectionHeader(
                                title = "Changed files",
                                subtitle = "${diff.changes.size} paths differ between the selected snapshots",
                            )
                        }

                        changeGroup("Added", diff.added)
                        changeGroup("Modified", diff.modified)
                        changeGroup("Deleted", diff.deleted)
                    }
                }
            }
        }
    }
}

private fun lineSummary(additions: Int, deletions: Int): String = when {
    additions > 0 && deletions > 0 -> "+$additions / −$deletions lines"
    additions > 0 -> "+$additions lines"
    deletions > 0 -> "−$deletions lines"
    else -> "no line changes"
}

private fun LazyListScope.changeGroup(label: String, changes: List<FileChange>) {
    if (changes.isEmpty()) return

    item(key = "group-header-$label") {
        val color = when (label) {
            "Added" -> LocalTimeCapsuleColors.current.added
            "Modified" -> LocalTimeCapsuleColors.current.modified
            else -> LocalTimeCapsuleColors.current.deleted
        }
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(6.dp)
                    .background(color, CircleShape),
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = "$label · ${changes.size}",
                style = MaterialTheme.typography.labelMedium,
                color = color,
            )
        }
    }

    items(
        count = changes.size,
        key = { index -> "change-$label-${changes[index].path}" },
    ) { index ->
        ChangeRow(changes[index])
    }
}

@Composable
private fun ChangeRow(change: FileChange) {
    val semantic = LocalTimeCapsuleColors.current
    Panel(contentPadding = PaddingValues(horizontal = 14.dp, vertical = 11.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            ChangeBadge(kind = change.kind)
            Spacer(Modifier.width(10.dp))
            Text(
                text = change.path,
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = MonoTextStyle.fontFamily),
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
            if (change.additions > 0) {
                Spacer(Modifier.width(8.dp))
                Text(
                    text = "+${change.additions}",
                    style = MaterialTheme.typography.labelMedium,
                    color = semantic.added,
                )
            }
            if (change.deletions > 0) {
                Spacer(Modifier.width(6.dp))
                Text(
                    text = "−${change.deletions}",
                    style = MaterialTheme.typography.labelMedium,
                    color = semantic.deleted,
                )
            }
        }
    }
}

@Composable
private fun TextualDiffPlaceholder() {
    Column {
        SectionHeader(
            title = "Textual diff",
            subtitle = "Line-level contents of the changed files",
        )
        Spacer(Modifier.height(8.dp))
        Panel(color = MaterialTheme.colorScheme.surfaceContainerHigh) {
            StatusPill(
                text = "Not available",
                color = LocalTimeCapsuleColors.current.modified,
            )
            Spacer(Modifier.height(10.dp))
            Text(
                text = "The demo repository stores no file contents, and the Rust core that produces " +
                    "unified diffs is not integrated. The file list above reflects real differences " +
                    "between the demo snapshots; the patch text does not exist yet.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(10.dp))
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Spacer(Modifier.height(10.dp))
            Text(
                text = "Planned: unified diff in a horizontally scrollable monospace pane.",
                style = MaterialTheme.typography.bodySmall,
                color = LocalTimeCapsuleColors.current.textFaint,
            )
        }
    }
}
