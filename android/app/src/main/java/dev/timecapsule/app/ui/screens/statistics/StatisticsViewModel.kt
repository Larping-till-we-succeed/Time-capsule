package dev.timecapsule.app.ui.screens.statistics

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.timecapsule.app.data.AppContainer
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.ProjectStatistics
import dev.timecapsule.app.ui.common.displayMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class StatisticsUiState(
    val projects: List<Project> = emptyList(),
    val project: Project? = null,
    val statistics: ProjectStatistics? = null,
    val isLoading: Boolean = true,
    val error: String? = null,
)

class StatisticsViewModel(private val container: AppContainer) : ViewModel() {

    private val repository = container.repository

    private val _state = MutableStateFlow(StatisticsUiState())
    val state = _state.asStateFlow()

    private var loadJob: Job? = null

    init {
        viewModelScope.launch {
            combine(
                repository.observeProjects(),
                container.selectedProject.selectedProjectId,
            ) { projects, selectedId -> projects to selectedId }
                .collect { (projects, selectedId) ->
                    val project = projects.firstOrNull { it.id == selectedId }
                    if (project == null) {
                        _state.value = StatisticsUiState(projects = projects, isLoading = false)
                        return@collect
                    }
                    _state.update { it.copy(projects = projects, project = project, isLoading = true, error = null) }
                    loadJob?.cancel()
                    loadJob = viewModelScope.launch { load(project) }
                }
        }
    }

    fun selectProject(projectId: ProjectId) = container.selectedProject.select(projectId)

    fun retry() {
        val project = _state.value.project ?: return
        loadJob?.cancel()
        loadJob = viewModelScope.launch { load(project) }
    }

    private suspend fun load(project: Project) {
        try {
            val statistics = repository.getStatistics(project.id)
            _state.update {
                it.copy(project = project, statistics = statistics, isLoading = false, error = null)
            }
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (error: Exception) {
            _state.update { it.copy(isLoading = false, error = error.displayMessage()) }
        }
    }
}
