package dev.timecapsule.app.ui.screens.settings

import androidx.compose.foundation.BorderStroke
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.timecapsule.app.data.model.AppPreferences
import dev.timecapsule.app.ui.LocalAppContainer
import dev.timecapsule.app.ui.appViewModel
import dev.timecapsule.app.ui.common.AppHeader
import dev.timecapsule.app.ui.common.ChoiceRow
import dev.timecapsule.app.ui.common.DemoDataBanner
import dev.timecapsule.app.ui.common.ErrorState
import dev.timecapsule.app.ui.common.LoadingState
import dev.timecapsule.app.ui.common.Panel
import dev.timecapsule.app.ui.common.ScreenLayout
import dev.timecapsule.app.ui.common.SectionHeader
import dev.timecapsule.app.ui.common.StatusPill
import dev.timecapsule.app.ui.common.ToggleRow
import dev.timecapsule.app.ui.common.formatInterval
import dev.timecapsule.app.ui.common.formatRetention
import dev.timecapsule.app.ui.common.rememberProjectDirectoryPicker
import dev.timecapsule.app.ui.theme.LocalTimeCapsuleColors
import dev.timecapsule.app.ui.theme.MonoTextStyle

@Composable
fun SettingsScreen(modifier: Modifier = Modifier) {
    val viewModel = appViewModel { SettingsViewModel(it) }
    val state by viewModel.state.collectAsState()
    val container = LocalAppContainer.current

    var pickerError by remember { mutableStateOf<String?>(null) }
    val pickDirectory = rememberProjectDirectoryPicker(
        store = container.projectDirectories,
        onPicked = {
            pickerError = null
            viewModel.addDirectory(it)
        },
        onError = { pickerError = it },
    )

    SettingsContent(
        state = state,
        modifier = modifier,
        pickerError = pickerError,
        onPickDirectory = pickDirectory,
        onAutomaticSnapshots = viewModel::setAutomaticSnapshots,
        onInterval = viewModel::setSnapshotInterval,
        onSnapshotOnLaunch = viewModel::setSnapshotOnLaunch,
        onRetention = viewModel::setRetentionDays,
        onLinkDirectory = viewModel::linkDirectory,
        onUnlinkProject = viewModel::unlinkProject,
        onRemoveDirectory = viewModel::removeDirectory,
        onRetry = viewModel::retry,
    )
}

@Composable
private fun SettingsContent(
    state: SettingsUiState,
    modifier: Modifier = Modifier,
    pickerError: String?,
    onPickDirectory: () -> Unit,
    onAutomaticSnapshots: (Boolean) -> Unit,
    onInterval: (Int) -> Unit,
    onSnapshotOnLaunch: (Boolean) -> Unit,
    onRetention: (Int) -> Unit,
    onLinkDirectory: (dev.timecapsule.app.core.storage.GrantedDirectory) -> Unit,
    onUnlinkProject: () -> Unit,
    onRemoveDirectory: (String) -> Unit,
    onRetry: () -> Unit,
) {
    val preferences = state.preferences
    val semantic = LocalTimeCapsuleColors.current
    val errorMessage = state.error

    ScreenLayout(modifier = modifier) {
        item {
            AppHeader(
                title = "Settings",
                overline = "TimeCapsule",
                subtitle = "Preferences are stored on this device. Nothing here runs the engine yet.",
            )
        }

        item { DemoDataBanner() }

        if (pickerError != null) {
            item {
                Text(
                    text = pickerError,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }

        if (state.message != null) {
            item {
                Panel(color = MaterialTheme.colorScheme.primaryContainer) {
                    Text(
                        text = state.message,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                }
            }
        }

        item { SectionHeader(title = "Automatic snapshots", subtitle = "Saved locally — not yet executed") }

        item {
            Panel {
                ToggleRow(
                    title = "Automatic snapshots",
                    description = "Capture history without pressing a button.",
                    checked = preferences.automaticSnapshots,
                    onCheckedChange = onAutomaticSnapshots,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                ChoiceRow(
                    title = "Interval",
                    description = "How often a snapshot would be taken.",
                    value = preferences.intervalMinutes,
                    options = AppPreferences.INTERVAL_CHOICES,
                    onSelect = onInterval,
                    label = ::formatInterval,
                    enabled = preferences.automaticSnapshots,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                ToggleRow(
                    title = "Snapshot when the app opens",
                    description = "Capture the project before the first edit.",
                    checked = preferences.snapshotOnLaunch,
                    onCheckedChange = onSnapshotOnLaunch,
                )
                Spacer(Modifier.height(10.dp))
                EngineNotice(
                    text = "These switches only store your intent. The Rust engine is not integrated, " +
                        "so no snapshot is ever created automatically.",
                )
            }
        }

        item { SectionHeader(title = "History retention", subtitle = "Not enforced in this build") }

        item {
            Panel {
                ChoiceRow(
                    title = "Retention window",
                    description = "How much history the engine should keep.",
                    value = preferences.retentionDays,
                    options = AppPreferences.RETENTION_CHOICES,
                    onSelect = onRetention,
                    label = ::formatRetention,
                )
                Spacer(Modifier.height(8.dp))
                EngineNotice(
                    text = "Retention is a request to the future Rust core. Nothing is pruned by the app.",
                )
            }
        }

        item {
            SectionHeader(
                title = "Project folders",
                subtitle = "Access granted through Android's Storage Access Framework",
            )
        }

        item {
            if (errorMessage != null) {
                ErrorState(message = errorMessage, onRetry = onRetry)
            } else if (state.isLoading) {
                LoadingState(label = "Reading persisted folder grants…")
            } else if (state.directories.isEmpty()) {
                Panel {
                    Text(
                        text = "No folders have been granted to TimeCapsule yet.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Spacer(Modifier.height(12.dp))
                    OutlinedButton(onClick = onPickDirectory) {
                        Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Choose folder")
                    }
                    Spacer(Modifier.height(12.dp))
                    EngineNotice(
                        text = "Android never exposes a raw path to the app. A selected folder is " +
                            "represented by a persisted content:// tree URI.",
                    )
                }
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    state.directories.forEach { grant ->
                        DirectoryCard(
                            grant = grant,
                            projectName = state.project?.name,
                            isLinked = grant.directory.uri == state.linkedDirectoryUri,
                            onLink = { onLinkDirectory(grant.directory) },
                            onRemove = { onRemoveDirectory(grant.directory.uri) },
                        )
                    }
                    if (state.linkedDirectoryUri != null) {
                        TextButton(onClick = onUnlinkProject) {
                            Text("Unlink folder from ${state.project?.name ?: "project"}")
                        }
                    }
                    OutlinedButton(onClick = onPickDirectory) {
                        Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Add another folder")
                    }
                    EngineNotice(
                        text = "Linking records a device-local association only. Indexing the folder " +
                            "and capturing snapshots is the Rust core's job and is not implemented.",
                    )
                }
            }
        }

        item { SectionHeader(title = "Appearance", subtitle = "Developer-tool defaults") }

        item {
            Panel {
                ToggleRow(
                    title = "Dark theme",
                    description = "The only theme in this build; a light scheme is not implemented.",
                    checked = true,
                    onCheckedChange = {},
                    enabled = false,
                )
            }
        }

        item { SectionHeader(title = "Engine", subtitle = "Current integration state") }

        item {
            Panel {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = "Snapshot repository",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.weight(1f),
                    )
                    StatusPill(
                        text = if (state.dataSource == dev.timecapsule.app.data.repository.DataSource.MOCK) {
                            "Demo data"
                        } else {
                            "Rust core"
                        },
                        color = semantic.modified,
                    )
                }
                Spacer(Modifier.height(12.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = "UniFFI bindings",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.weight(1f),
                    )
                    StatusPill(
                        text = if (state.coreStatus is dev.timecapsule.app.core.rust.CoreAvailability.Available) {
                            "Linked"
                        } else {
                            "Not integrated"
                        },
                        color = semantic.deleted,
                    )
                }
                Spacer(Modifier.height(12.dp))
                Text(
                    text = "Proposed contract: docs/android-rust-api.md",
                    style = MaterialTheme.typography.bodySmall.copy(fontFamily = MonoTextStyle.fontFamily),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        item { SectionHeader(title = "About") }

        item {
            Panel {
                Text(
                    text = "TimeCapsule for Android",
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    text = "Version 0.1.0 · foundation build. Screens, navigation and repositories are " +
                        "in place; the Rust engine is not connected.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun EngineNotice(text: String) {
    Row(verticalAlignment = Alignment.Top) {
        Icon(
            imageVector = Icons.Filled.Info,
            contentDescription = null,
            tint = LocalTimeCapsuleColors.current.modified,
            modifier = Modifier.size(15.dp),
        )
        Spacer(Modifier.width(8.dp))
        Text(
            text = text,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun DirectoryCard(
    grant: DirectoryGrant,
    projectName: String?,
    isLinked: Boolean,
    onLink: () -> Unit,
    onRemove: () -> Unit,
) {
    val description = grant.description

    Surface(
        color = MaterialTheme.colorScheme.surfaceContainer,
        shape = RoundedCornerShape(18.dp),
        border = BorderStroke(
            1.dp,
            if (isLinked) {
                MaterialTheme.colorScheme.primary.copy(alpha = 0.45f)
            } else {
                MaterialTheme.colorScheme.outlineVariant
            },
        ),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = grant.directory.displayName,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f),
                )
                if (isLinked) {
                    StatusPill(text = "Linked", color = MaterialTheme.colorScheme.primary)
                }
            }
            Spacer(Modifier.height(6.dp))
            Text(
                text = grant.directory.uri,
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = MonoTextStyle.fontFamily),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                text = when {
                    description == null -> "Contents could not be read. The grant may have been revoked."
                    !description.isUsable -> "Read-only access · ${description.entryCount} top-level entries"
                    else -> "${description.entryCount} top-level entries · read and write access"
                },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(6.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                if (!isLinked) {
                    TextButton(onClick = onLink, contentPadding = PaddingValues(horizontal = 8.dp)) {
                        Text(if (projectName != null) "Use for $projectName" else "Select a project first")
                    }
                }
                TextButton(onClick = onRemove, contentPadding = PaddingValues(horizontal = 8.dp)) {
                    Text("Remove access", color = MaterialTheme.colorScheme.error)
                }
            }
        }
    }
}
