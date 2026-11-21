package dev.timecapsule.app.data.model

import java.time.Instant

/**
 * Stable identifiers owned by the Rust core once it is integrated. The Android UI treats
 * them as opaque strings; nothing in the UI may derive meaning from their format.
 */
@JvmInline
value class ProjectId(val value: String) {
    override fun toString(): String = value
}

@JvmInline
value class SnapshotId(val value: String) {
    override fun toString(): String = value
}

/**
 * A directory the user has granted access to through the Storage Access Framework.
 *
 * The URI is a `content://` tree URI, not a filesystem path: Android does not let the app
 * read arbitrary paths, so this is the only representation of a project location that exists
 * on this platform.
 */
data class ProjectDirectory(
    val displayName: String,
    val uri: String,
)

/**
 * A snapshot project.
 *
 * [snapshotCount], [storageBytes] and [lastSnapshotAt] are denormalised for cheap list
 * rendering; the authoritative values are still owned by the core engine.
 */
data class Project(
    val id: ProjectId,
    val name: String,
    val description: String,
    val directory: ProjectDirectory?,
    val createdAt: Instant,
    val activeBranch: String,
    val snapshotCount: Int,
    val storageBytes: Long,
    val lastSnapshotAt: Instant?,
)
