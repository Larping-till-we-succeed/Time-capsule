package dev.timecapsule.app.ui.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.timecapsule.app.core.rust.CoreAvailability
import dev.timecapsule.app.core.rust.DirectoryDescription
import dev.timecapsule.app.core.storage.GrantedDirectory
import dev.timecapsule.app.data.AppContainer
import dev.timecapsule.app.data.model.AppPreferences
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.repository.DataSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class DirectoryGrant(
    val directory: GrantedDirectory,
    val description: DirectoryDescription?,
)

data class SettingsUiState(
    val preferences: AppPreferences = AppPreferences(),
    val project: Project? = null,
    val directories: List<DirectoryGrant> = emptyList(),
    val linkedDirectoryUri: String? = null,
    val isLoading: Boolean = true,
    val error: String? = null,
    val message: String? = null,
    val dataSource: DataSource = DataSource.MOCK,
    val coreStatus: CoreAvailability = CoreAvailability.NotIntegrated,
)

class SettingsViewModel(private val container: AppContainer) : ViewModel() {

    private val _state = MutableStateFlow(
        SettingsUiState(
            dataSource = container.repository.dataSource,
            coreStatus = container.coreAvailability,
        ),
    )
    val state = _state.asStateFlow()

    init {
        viewModelScope.launch {
            combine(
                container.preferences.preferences,
                container.selectedProject.selectedProjectId,
            ) { preferences, projectId -> preferences to projectId }
                .collectLatest { (preferences, projectId) ->
                    val project = projectId?.let { id ->
                        runCatching { container.repository.getProject(id) }.getOrNull()
                    }
                    _state.update { it.copy(preferences = preferences, project = project) }
                    loadDirectories(project)
                }
        }
    }

    fun setAutomaticSnapshots(enabled: Boolean) = container.preferences.setAutomaticSnapshots(enabled)

    fun setSnapshotInterval(minutes: Int) = container.preferences.setSnapshotInterval(minutes)

    fun setSnapshotOnLaunch(enabled: Boolean) = container.preferences.setSnapshotOnLaunch(enabled)

    fun setRetentionDays(days: Int) = container.preferences.setRetentionDays(days)

    /** Called after the SAF picker persisted a new grant. */
    fun addDirectory(directory: GrantedDirectory) {
        val project = _state.value.project
        if (project != null) {
            container.projectDirectories.linkProject(project.id, directory)
            _state.update {
                it.copy(message = "${directory.displayName} is now linked to ${project.name}.")
            }
        } else {
            _state.update {
                it.copy(message = "${directory.displayName} is available, but no project is selected.")
            }
        }
        refresh(project)
    }

    fun linkDirectory(directory: GrantedDirectory) {
        val project = _state.value.project ?: return
        container.projectDirectories.linkProject(project.id, directory)
        _state.update { it.copy(message = "${directory.displayName} is now linked to ${project.name}.") }
        refresh(project)
    }

    fun unlinkProject() {
        val project = _state.value.project ?: return
        container.projectDirectories.unlinkProject(project.id)
        _state.update { it.copy(message = "Folder unlinked from ${project.name}.") }
        refresh(project)
    }

    fun removeDirectory(uri: String) {
        container.projectDirectories.releaseGrant(uri)
        _state.update { it.copy(message = "Android access to that folder was released.") }
        refresh(_state.value.project)
    }

    fun clearMessage() {
        _state.update { it.copy(message = null) }
    }

    fun retry() {
        refresh(_state.value.project)
    }

    private fun refresh(project: Project?) {
        viewModelScope.launch { loadDirectories(project) }
    }

    private suspend fun loadDirectories(project: Project?) {
        try {
            val (grants, linkedUri) = withContext(Dispatchers.IO) {
                val grants = container.projectDirectories.grantedDirectories().map { granted ->
                    DirectoryGrant(
                        directory = granted,
                        description = runCatching {
                            container.storageGateway.describe(granted.uri)
                        }.getOrNull(),
                    )
                }
                val linked = project?.let { container.projectDirectories.linkedDirectory(it.id) }
                grants to linked?.uri
            }
            _state.update {
                it.copy(
                    directories = grants,
                    linkedDirectoryUri = linkedUri,
                    isLoading = false,
                    error = null,
                )
            }
        } catch (error: Exception) {
            _state.update {
                it.copy(isLoading = false, error = error.message ?: "Could not read folder permissions.")
            }
        }
    }
}
