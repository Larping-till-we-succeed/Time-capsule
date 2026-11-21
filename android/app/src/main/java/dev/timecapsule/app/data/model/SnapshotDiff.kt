package dev.timecapsule.app.data.model

/** How a file changed between two snapshots. */
enum class ChangeKind { ADDED, MODIFIED, DELETED }

/** One changed file between two snapshots. Line counts come from the core engine's diff. */
data class FileChange(
    val path: String,
    val kind: ChangeKind,
    val additions: Int,
    val deletions: Int,
)

/**
 * Textual diff payload.
 *
 * The fake repository can never produce [Available]; the screen renders an explicit
 * "not available" placeholder instead of inventing diff text.
 */
sealed interface TextualDiff {
    data object NotAvailable : TextualDiff

    data class Available(val unified: String) : TextualDiff
}

/**
 * The difference between two snapshots of the same project.
 *
 * [from] is the older baseline snapshot and [to] is the newer snapshot being compared.
 */
data class SnapshotDiff(
    val projectId: ProjectId,
    val from: SnapshotId,
    val to: SnapshotId,
    val changes: List<FileChange>,
    val textualDiff: TextualDiff = TextualDiff.NotAvailable,
) {
    val added: List<FileChange> get() = changes.filter { it.kind == ChangeKind.ADDED }
    val modified: List<FileChange> get() = changes.filter { it.kind == ChangeKind.MODIFIED }
    val deleted: List<FileChange> get() = changes.filter { it.kind == ChangeKind.DELETED }

    val additions: Int get() = changes.sumOf { it.additions }
    val deletions: Int get() = changes.sumOf { it.deletions }

    val isEmpty: Boolean get() = changes.isEmpty()
}
