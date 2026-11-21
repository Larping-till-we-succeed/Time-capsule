package dev.timecapsule.app.ui.screens.compare

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.timecapsule.app.data.AppContainer
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.data.model.SnapshotDiff
import dev.timecapsule.app.data.model.SnapshotId
import dev.timecapsule.app.ui.common.displayMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class CompareUiState(
    val projects: List<Project> = emptyList(),
    val project: Project? = null,
    val snapshots: List<Snapshot> = emptyList(),
    val fromId: SnapshotId? = null,
    val toId: SnapshotId? = null,
    val diff: SnapshotDiff? = null,
    val isLoading: Boolean = true,
    val isDiffLoading: Boolean = false,
    val error: String? = null,
    val diffNote: String? = null,
)

class CompareViewModel(private val container: AppContainer) : ViewModel() {

    private val repository = container.repository

    private val _state = MutableStateFlow(CompareUiState())
    val state = _state.asStateFlow()

    private var loadJob: Job? = null
    private var diffJob: Job? = null

    init {
        viewModelScope.launch {
            combine(
                repository.observeProjects(),
                container.selectedProject.selectedProjectId,
            ) { projects, selectedId -> projects to selectedId }
                .collect { (projects, selectedId) ->
                    val project = projects.firstOrNull { it.id == selectedId }
                    if (project == null) {
                        _state.value = CompareUiState(projects = projects, isLoading = false)
                        return@collect
                    }
                    _state.update { it.copy(projects = projects, project = project, isLoading = true, error = null) }
                    loadJob?.cancel()
                    loadJob = viewModelScope.launch { loadSnapshots(project) }
                }
        }
    }

    fun selectProject(projectId: ProjectId) = container.selectedProject.select(projectId)

    fun selectFrom(snapshotId: SnapshotId) {
        if (snapshotId == _state.value.fromId) return
        _state.update { it.copy(fromId = snapshotId) }
        loadDiff()
    }

    fun selectTo(snapshotId: SnapshotId) {
        if (snapshotId == _state.value.toId) return
        _state.update { it.copy(toId = snapshotId) }
        loadDiff()
    }

    fun swap() {
        _state.update { it.copy(fromId = it.toId, toId = it.fromId) }
        loadDiff()
    }

    fun retry() {
        val snapshotState = _state.value
        if (snapshotState.snapshots.isEmpty()) {
            val project = snapshotState.project ?: return
            loadJob?.cancel()
            loadJob = viewModelScope.launch { loadSnapshots(project) }
        } else {
            loadDiff()
        }
    }

    private suspend fun loadSnapshots(project: Project) {
        try {
            val snapshots = repository.listSnapshots(project.id)
            val previous = _state.value
            val stillValid = previous.snapshots.map { it.id }.toSet()
            val keepFrom = previous.fromId?.takeIf { it in stillValid }
            val keepTo = previous.toId?.takeIf { it in stillValid }

            val to = keepTo ?: snapshots.getOrNull(0)?.id
            val from = keepFrom?.takeIf { it != to } ?: snapshots.getOrNull(1)?.id

            _state.update {
                it.copy(
                    project = project,
                    snapshots = snapshots,
                    fromId = from,
                    toId = to,
                    isLoading = false,
                    error = null,
                    diff = null,
                )
            }
            loadDiff()
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (error: Exception) {
            _state.update { it.copy(isLoading = false, error = error.displayMessage()) }
        }
    }

    private fun loadDiff() {
        val snapshotState = _state.value
        val project = snapshotState.project
        val from = snapshotState.fromId
        val to = snapshotState.toId

        if (project == null || from == null || to == null) {
            _state.update { it.copy(diff = null, isDiffLoading = false) }
            return
        }
        if (from == to) {
            _state.update {
                it.copy(
                    diff = null,
                    isDiffLoading = false,
                    diffNote = "Choose two different snapshots to see what changed.",
                )
            }
            return
        }

        diffJob?.cancel()
        diffJob = viewModelScope.launch {
            _state.update { it.copy(isDiffLoading = true, diffNote = null) }
            try {
                val diff = repository.compareSnapshots(project.id, from, to)
                _state.update {
                    it.copy(
                        diff = diff,
                        isDiffLoading = false,
                        diffNote = if (diff.isEmpty) "These snapshots have identical content." else null,
                    )
                }
            } catch (cancelled: CancellationException) {
                throw cancelled
            } catch (error: Exception) {
                _state.update {
                    it.copy(isDiffLoading = false, diff = null, diffNote = error.displayMessage())
                }
            }
        }
    }
}
