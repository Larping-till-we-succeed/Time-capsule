package dev.timecapsule.app.ui.screens.dashboard

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.timecapsule.app.core.storage.GrantedDirectory
import dev.timecapsule.app.data.AppContainer
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.data.repository.DataSource
import dev.timecapsule.app.ui.common.displayMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class DashboardUiState(
    val projects: List<Project> = emptyList(),
    val project: Project? = null,
    val recentSnapshots: List<Snapshot> = emptyList(),
    val linkedDirectory: GrantedDirectory? = null,
    val isLoading: Boolean = true,
    val error: String? = null,
    val dataSource: DataSource = DataSource.MOCK,
)

class DashboardViewModel(private val container: AppContainer) : ViewModel() {

    private val repository = container.repository

    private val _state = MutableStateFlow(DashboardUiState(dataSource = repository.dataSource))
    val state = _state.asStateFlow()

    private var detailJob: Job? = null

    init {
        viewModelScope.launch {
            combine(
                repository.observeProjects(),
                container.selectedProject.selectedProjectId,
            ) { projects, selectedId -> projects to selectedId }
                .collect { (projects, selectedId) ->
                    val project = projects.firstOrNull { it.id == selectedId } ?: projects.firstOrNull()

                    if (project != null && project.id != selectedId) {
                        // Nothing persisted yet, or the stored project disappeared: adopt a
                        // valid selection instead of showing an empty screen.
                        container.selectedProject.select(project.id)
                        return@collect
                    }

                    _state.update {
                        it.copy(
                            projects = projects,
                            project = project,
                            isLoading = project != null,
                            error = null,
                            recentSnapshots = if (project == null) emptyList() else it.recentSnapshots,
                        )
                    }

                    if (project == null) return@collect

                    detailJob?.cancel()
                    detailJob = viewModelScope.launch { loadProjectDetails(project) }
                }
        }
    }

    fun selectProject(projectId: ProjectId) {
        container.selectedProject.select(projectId)
    }

    fun attachDirectory(directory: GrantedDirectory) {
        val project = _state.value.project ?: return
        container.projectDirectories.linkProject(project.id, directory)
        _state.update { it.copy(linkedDirectory = directory) }
    }

    fun detachDirectory() {
        val project = _state.value.project ?: return
        container.projectDirectories.unlinkProject(project.id)
        _state.update { it.copy(linkedDirectory = null) }
    }

    fun retry() {
        val project = _state.value.project ?: return
        detailJob?.cancel()
        detailJob = viewModelScope.launch { loadProjectDetails(project) }
    }

    private suspend fun loadProjectDetails(project: Project) {
        _state.update { it.copy(isLoading = true, error = null) }
        try {
            val snapshots = repository.listSnapshots(project.id)
            val refreshed = runCatching { repository.getProject(project.id) }.getOrDefault(project)
            val directory = withContext(Dispatchers.IO) {
                container.projectDirectories.linkedDirectory(project.id)
            }
            _state.update {
                it.copy(
                    project = refreshed,
                    recentSnapshots = snapshots.take(4),
                    linkedDirectory = directory,
                    isLoading = false,
                    error = null,
                )
            }
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (error: Exception) {
            _state.update { it.copy(isLoading = false, error = error.displayMessage()) }
        }
    }
}
