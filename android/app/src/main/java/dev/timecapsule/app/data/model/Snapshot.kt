package dev.timecapsule.app.data.model

import java.time.Instant

/**
 * An immutable point-in-time snapshot of a project directory.
 *
 * [fileCount] and [sizeBytes] describe the content captured by the snapshot, not the size of
 * the app's own storage. [note] is optional author commentary shown in the timeline details.
 */
data class Snapshot(
    val id: SnapshotId,
    val projectId: ProjectId,
    val title: String,
    val note: String,
    val branch: String,
    val createdAt: Instant,
    val fileCount: Int,
    val sizeBytes: Long,
) {
    /** Short, human-friendly form of the identifier, shown in monospace in the UI. */
    val shortId: CharSequence
        get() = id.value.takeLast(8)
}

/**
 * A named line of history. [isActive] marks the branch new snapshots are currently written to.
 */
data class Branch(
    val name: String,
    val snapshotCount: Int,
    val lastSnapshotAt: Instant?,
    val isActive: Boolean,
)
