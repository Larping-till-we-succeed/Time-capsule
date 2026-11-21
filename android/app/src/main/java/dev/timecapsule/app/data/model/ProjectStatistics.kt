package dev.timecapsule.app.data.model

import java.time.Instant
import java.time.LocalDate

/** Snapshot activity for a single calendar day, used by the statistics chart. */
data class DailyActivity(
    val date: LocalDate,
    val snapshotCount: Int,
)

/**
 * Aggregated numbers for one project.
 *
 * [fileCount] is the number of distinct files tracked by the latest snapshot; [storageBytes]
 * is the on-disk cost of the project's snapshot data, not the size of the working directory.
 */
data class ProjectStatistics(
    val projectId: ProjectId,
    val snapshotCount: Int,
    val fileCount: Int,
    val storageBytes: Long,
    val branchCount: Int,
    val earliestSnapshotAt: Instant?,
    val latestSnapshotAt: Instant?,
    val activity: List<DailyActivity>,
    val branches: List<Branch>,
)
