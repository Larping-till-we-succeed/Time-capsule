package dev.timecapsule.app.data.repository

import dev.timecapsule.app.data.model.Branch
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.ProjectStatistics
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.data.model.SnapshotDiff
import dev.timecapsule.app.data.model.SnapshotId
import kotlinx.coroutines.flow.Flow

/** Where the data shown by the UI actually comes from. */
enum class DataSource {
    /** In-memory demonstration data. Nothing is written to disk. */
    MOCK,

    /** A real Rust core reached through UniFFI bindings. */
    RUST_CORE,
}

/**
 * Read access to projects.
 *
 * Implementations must never touch Android UI types; a UniFFI adapter implements this
 * interface unchanged.
 */
interface ProjectRepository {
    fun observeProjects(): Flow<List<Project>>

    suspend fun listProjects(): List<Project>

    suspend fun getProject(projectId: ProjectId): Project

    fun observeProject(projectId: ProjectId): Flow<Project?>

    suspend fun listBranches(projectId: ProjectId): List<Branch>

    suspend fun getStatistics(projectId: ProjectId): ProjectStatistics
}

/** Read access to a project's snapshot history. */
interface SnapshotRepository {
    fun observeSnapshots(projectId: ProjectId): Flow<List<Snapshot>>

    suspend fun listSnapshots(projectId: ProjectId): List<Snapshot>

    suspend fun getSnapshot(projectId: ProjectId, snapshotId: SnapshotId): Snapshot

    /**
     * Compares [fromSnapshotId] (baseline) with [toSnapshotId] (newer snapshot).
     *
     * Both snapshots must belong to [projectId].
     */
    suspend fun compareSnapshots(
        projectId: ProjectId,
        fromSnapshotId: SnapshotId,
        toSnapshotId: SnapshotId,
    ): SnapshotDiff
}

/**
 * The single boundary the UI depends on.
 *
 * Today the only implementation is [InMemoryTimeCapsuleRepository]. A future
 * `UniffiTimeCapsuleRepository` implements the same interface on top of the Rust core,
 * which is why no screen imports a concrete implementation.
 */
interface TimeCapsuleRepository : ProjectRepository, SnapshotRepository {
    val dataSource: DataSource

    /** True while the app is backed by demonstration data rather than the Rust engine. */
    val isMock: Boolean get() = dataSource == DataSource.MOCK
}

/** Base type for failures that are safe to show to the user. */
sealed class RepositoryException(message: String) : Exception(message) {
    class ProjectNotFound(val projectId: ProjectId) :
        RepositoryException("Project ${projectId.value} was not found.")

    class SnapshotNotFound(val projectId: ProjectId, val snapshotId: SnapshotId) :
        RepositoryException("Snapshot ${snapshotId.value} does not exist in this project.")

    class InvalidComparison(message: String) : RepositoryException(message)
}
