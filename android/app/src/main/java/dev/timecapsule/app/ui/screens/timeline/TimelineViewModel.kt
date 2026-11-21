package dev.timecapsule.app.ui.screens.timeline

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.timecapsule.app.data.AppContainer
import dev.timecapsule.app.data.model.Branch
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.data.model.SnapshotId
import dev.timecapsule.app.ui.common.displayMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class TimelineUiState(
    val projects: List<Project> = emptyList(),
    val project: Project? = null,
    val snapshots: List<Snapshot> = emptyList(),
    val branches: List<Branch> = emptyList(),
    val branchFilter: String? = null,
    val expanded: Set<SnapshotId> = emptySet(),
    val isLoading: Boolean = true,
    val error: String? = null,
) {
    val visibleSnapshots: List<Snapshot>
        get() = snapshots.filter { branchFilter == null || it.branch == branchFilter }
}

class TimelineViewModel(private val container: AppContainer) : ViewModel() {

    private val repository = container.repository

    private val _state = MutableStateFlow(TimelineUiState())
    val state = _state.asStateFlow()

    init {
        viewModelScope.launch {
            combine(
                repository.observeProjects(),
                container.selectedProject.selectedProjectId,
            ) { projects, selectedId -> projects to selectedId }
                .collect { (projects, selectedId) ->
                    val project = projects.firstOrNull { it.id == selectedId }
                    if (project == null) {
                        _state.value = TimelineUiState(
                            projects = projects,
                            isLoading = false,
                            error = if (projects.isEmpty()) null else "Select a project to see its history.",
                        )
                        return@collect
                    }
                    _state.update {
                        it.copy(projects = projects, project = project, isLoading = true, error = null)
                    }
                    load(project)
                }
        }
    }

    fun selectProject(projectId: ProjectId) = container.selectedProject.select(projectId)

    fun selectBranch(branch: String?) {
        _state.update { it.copy(branchFilter = branch) }
    }

    fun toggleExpanded(snapshotId: SnapshotId) {
        _state.update { current ->
            val next = if (snapshotId in current.expanded) {
                current.expanded - snapshotId
            } else {
                current.expanded + snapshotId
            }
            current.copy(expanded = next)
        }
    }

    fun retry() {
        val project = _state.value.project ?: return
        viewModelScope.launch { load(project) }
    }

    private suspend fun load(project: Project) {
        try {
            val snapshots = repository.listSnapshots(project.id)
            val branches = repository.listBranches(project.id)
            _state.update { current ->
                current.copy(
                    project = project,
                    snapshots = snapshots,
                    branches = branches,
                    branchFilter = current.branchFilter?.takeIf { filter ->
                        branches.any { it.name == filter }
                    },
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
