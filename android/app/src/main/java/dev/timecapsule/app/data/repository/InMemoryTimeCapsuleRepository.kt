package dev.timecapsule.app.data.repository

import dev.timecapsule.app.data.model.Branch
import dev.timecapsule.app.data.model.ChangeKind
import dev.timecapsule.app.data.model.DailyActivity
import dev.timecapsule.app.data.model.FileChange
import dev.timecapsule.app.data.model.Project
import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.ProjectStatistics
import dev.timecapsule.app.data.model.Snapshot
import dev.timecapsule.app.data.model.SnapshotDiff
import dev.timecapsule.app.data.model.SnapshotId
import dev.timecapsule.app.data.model.TextualDiff
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import java.time.temporal.ChronoUnit
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map

/**
 * In-memory implementation used until the Rust core is integrated.
 *
 * Everything here is demonstration data held in RAM: nothing is read from or written to the
 * device, no snapshot is ever created, restored or compared for real, and no diff text or
 * per-line count is produced by an engine. The values are internally consistent (a diff is
 * derived from the demo file manifests, not hard-coded) so the UI can be exercised honestly,
 * but they are still synthetic — see `DataSource.MOCK` and the "Demo data" banner in the app.
 *
 * The class is deliberately self-contained and easily replaced: the UI only knows
 * [TimeCapsuleRepository].
 *
 * @param simulatedLatency delay added to suspending calls so loading states are reachable
 *   during development; tests pass [Duration.ZERO].
 */
class InMemoryTimeCapsuleRepository(
    private val clock: Clock = Clock.systemUTC(),
    private val simulatedLatency: Duration = Duration.ofMillis(200),
) : TimeCapsuleRepository {

    override val dataSource: DataSource = DataSource.MOCK

    private val seed = FakeSeed.create(clock.instant())

    private val projects = MutableStateFlow(seed.projects)
    private val snapshots = MutableStateFlow(seed.snapshots)

    override fun observeProjects(): Flow<List<Project>> = projects.asStateFlow()

    override suspend fun listProjects(): List<Project> {
        simulateWork()
        return projects.value
    }

    override suspend fun getProject(projectId: ProjectId): Project {
        simulateWork()
        return projects.value.firstOrNull { it.id == projectId }
            ?: throw RepositoryException.ProjectNotFound(projectId)
    }

    override fun observeProject(projectId: ProjectId): Flow<Project?> =
        projects.map { list -> list.firstOrNull { it.id == projectId } }

    override fun observeSnapshots(projectId: ProjectId): Flow<List<Snapshot>> =
        snapshots.map { all -> all.filter { it.projectId == projectId }.sortedByDescending { it.createdAt } }

    override suspend fun listSnapshots(projectId: ProjectId): List<Snapshot> {
        simulateWork()
        requireProject(projectId)
        return snapshots.value
            .filter { it.projectId == projectId }
            .sortedByDescending { it.createdAt }
    }

    override suspend fun getSnapshot(projectId: ProjectId, snapshotId: SnapshotId): Snapshot {
        simulateWork()
        return snapshotFor(projectId, snapshotId)
    }

    override suspend fun listBranches(projectId: ProjectId): List<Branch> {
        simulateWork()
        val project = requireProject(projectId)
        return branchesOf(project, snapshots.value.filter { it.projectId == projectId })
    }

    override suspend fun compareSnapshots(
        projectId: ProjectId,
        fromSnapshotId: SnapshotId,
        toSnapshotId: SnapshotId,
    ): SnapshotDiff {
        simulateWork()
        requireProject(projectId)
        val from = snapshotFor(projectId, fromSnapshotId)
        val to = snapshotFor(projectId, toSnapshotId)
        if (from.id == to.id) {
            throw RepositoryException.InvalidComparison("Choose two different snapshots to compare.")
        }
        // Comparisons are ordered by time rather than by argument position, so the UI can
        // always show "older -> newer" regardless of which picker the user used.
        val older = if (from.createdAt.isBefore(to.createdAt)) from else to
        val newer = if (older.id == from.id) to else from

        val before = seed.manifests[older.id].orEmpty()
        val after = seed.manifests[newer.id].orEmpty()

        val changes = buildList {
            (after.keys - before.keys).sorted().forEach { path ->
                val entry = after.getValue(path)
                add(FileChange(path, ChangeKind.ADDED, additions = entry.lines, deletions = 0))
            }
            (before.keys intersect after.keys).sorted().forEach { path ->
                val old = before.getValue(path)
                val new = after.getValue(path)
                if (old != new) {
                    val delta = new.lines - old.lines
                    add(
                        FileChange(
                            path = path,
                            kind = ChangeKind.MODIFIED,
                            additions = maxOf(1, delta),
                            deletions = maxOf(1, -delta),
                        ),
                    )
                }
            }
            (before.keys - after.keys).sorted().forEach { path ->
                val entry = before.getValue(path)
                add(FileChange(path, ChangeKind.DELETED, additions = 0, deletions = entry.lines))
            }
        }

        return SnapshotDiff(
            projectId = projectId,
            from = older.id,
            to = newer.id,
            changes = changes,
            // The fake repository cannot produce a unified diff: rendering one would mean
            // inventing file contents.
            textualDiff = TextualDiff.NotAvailable,
        )
    }

    override suspend fun getStatistics(projectId: ProjectId): ProjectStatistics {
        simulateWork()
        val project = requireProject(projectId)
        val projectSnapshots = snapshots.value.filter { it.projectId == projectId }
        val latest = projectSnapshots.maxByOrNull { it.createdAt }
        val latestManifest = latest?.let { seed.manifests[it.id] }.orEmpty()

        val today = LocalDate.now(clock)
        val start = today.minusDays(ACTIVITY_DAYS - 1L)
        val perDay = projectSnapshots
            .groupingBy { it.createdAt.atZone(ZoneOffset.UTC).toLocalDate() }
            .eachCount()

        val activity = (0 until ACTIVITY_DAYS).map { offset ->
            val date = start.plusDays(offset.toLong())
            DailyActivity(date = date, snapshotCount = perDay[date] ?: 0)
        }

        return ProjectStatistics(
            projectId = projectId,
            snapshotCount = projectSnapshots.size,
            fileCount = latestManifest.size,
            storageBytes = project.storageBytes,
            branchCount = projectSnapshots.map { it.branch }.distinct().size,
            earliestSnapshotAt = projectSnapshots.minOfOrNull { it.createdAt },
            latestSnapshotAt = projectSnapshots.maxOfOrNull { it.createdAt },
            activity = activity,
            branches = branchesOf(project, projectSnapshots),
        )
    }

    private fun branchesOf(project: Project, projectSnapshots: List<Snapshot>): List<Branch> =
        projectSnapshots
            .groupBy { it.branch }
            .map { (name, entries) ->
                Branch(
                    name = name,
                    snapshotCount = entries.size,
                    lastSnapshotAt = entries.maxOfOrNull { it.createdAt },
                    isActive = name == project.activeBranch,
                )
            }
            .sortedWith(compareByDescending<Branch> { it.isActive }.thenByDescending { it.lastSnapshotAt })

    private fun requireProject(projectId: ProjectId): Project =
        projects.value.firstOrNull { it.id == projectId }
            ?: throw RepositoryException.ProjectNotFound(projectId)

    private fun snapshotFor(projectId: ProjectId, snapshotId: SnapshotId): Snapshot =
        snapshots.value.firstOrNull { it.id == snapshotId && it.projectId == projectId }
            ?: throw RepositoryException.SnapshotNotFound(projectId, snapshotId)

    private suspend fun simulateWork() {
        val millis = simulatedLatency.toMillis()
        if (millis > 0) delay(millis)
    }

    private companion object {
        const val ACTIVITY_DAYS = 14
    }
}

// ---------------------------------------------------------------------------------------------
// Demo data
// ---------------------------------------------------------------------------------------------

/** Synthetic file state used to derive demo diffs. Never leaves this file. */
private data class Fingerprint(val revision: Int, val lines: Int)

private data class FakeSeed(
    val projects: List<Project>,
    val snapshots: List<Snapshot>,
    val manifests: Map<SnapshotId, Map<String, Fingerprint>>,
) {
    companion object {
        private fun project(
            id: ProjectId,
            name: String,
            description: String,
            activeBranch: String,
            createdAt: Instant,
            snapshots: List<Snapshot>,
            storageBytes: Long,
        ) = Project(
            id = id,
            name = name,
            description = description,
            // Demo projects are not linked to a real folder; linking requires the core engine.
            directory = null,
            createdAt = createdAt,
            activeBranch = activeBranch,
            snapshotCount = snapshots.size,
            storageBytes = storageBytes,
            lastSnapshotAt = snapshots.maxOfOrNull { it.createdAt },
        )

        private fun history(slug: String, projectId: String, now: Instant) =
            HistoryBuilder(slug, ProjectId(projectId), now)

        fun create(now: Instant): FakeSeed {
            val aurora = history("aurora", "proj_aurora_01", now)
                .seed(
                    "build.gradle.kts",
                    "settings.gradle.kts",
                    "gradle.properties",
                    "README.md",
                    "LICENSE",
                    ".gitignore",
                    "docs/architecture.md",
                    "docs/keybindings.md",
                    "src/main/kotlin/dev/aurora/App.kt",
                    "src/main/kotlin/dev/aurora/MainWindow.kt",
                    "src/main/kotlin/dev/aurora/editor/EditorPane.kt",
                    "src/main/kotlin/dev/aurora/editor/TextBuffer.kt",
                    "src/main/kotlin/dev/aurora/editor/Cursor.kt",
                    "src/main/kotlin/dev/aurora/editor/SyntaxHighlighter.kt",
                    "src/main/kotlin/dev/aurora/theme/Palette.kt",
                    "src/main/kotlin/dev/aurora/theme/Typography.kt",
                    "src/main/kotlin/dev/aurora/io/FileSession.kt",
                    "src/main/kotlin/dev/aurora/io/Autosave.kt",
                    "src/main/kotlin/dev/aurora/search/SearchIndex.kt",
                    "src/main/kotlin/dev/aurora/search/QuickOpen.kt",
                    "src/main/kotlin/dev/aurora/settings/SettingsStore.kt",
                    "src/main/kotlin/dev/aurora/settings/SettingsUi.kt",
                    "src/main/kotlin/dev/aurora/palette/CommandRegistry.kt",
                    "src/main/kotlin/dev/aurora/lsp/LanguageClient.kt",
                    "src/main/kotlin/dev/aurora/lsp/CompletionProvider.kt",
                    "src/main/kotlin/dev/aurora/util/Logger.kt",
                    "src/main/kotlin/dev/aurora/util/Disposable.kt",
                    "src/test/kotlin/dev/aurora/editor/TextBufferTest.kt",
                    "src/test/kotlin/dev/aurora/search/SearchIndexTest.kt",
                )

            aurora.snapshot(
                title = "Initial import",
                note = "First working editor loop: open a file, type, save.",
                branch = "main",
                daysAgo = 41,
            )
            aurora.snapshot(
                title = "Add syntax highlighting pipeline",
                note = "Tokeniser runs off the UI thread and feeds the renderer incrementally.",
                branch = "main",
                daysAgo = 36,
            ) {
                add("src/main/kotlin/dev/aurora/editor/Tokeniser.kt", lines = 214)
                add("src/main/kotlin/dev/aurora/editor/Token.kt", lines = 58)
                edit("src/main/kotlin/dev/aurora/editor/SyntaxHighlighter.kt", delta = 96)
                edit("src/main/kotlin/dev/aurora/editor/EditorPane.kt", delta = 41)
            }
            aurora.snapshot(
                title = "Persist editor settings",
                note = "Settings now survive restarts; palette selection is remembered.",
                branch = "main",
                daysAgo = 31,
            ) {
                edit("src/main/kotlin/dev/aurora/settings/SettingsStore.kt", delta = 74)
                edit("src/main/kotlin/dev/aurora/settings/SettingsUi.kt", delta = 132)
                edit("src/main/kotlin/dev/aurora/io/FileSession.kt", delta = 18)
            }
            aurora.snapshot(
                title = "Fix autosave race on window close",
                note = "Autosave worker is cancelled and flushed before the session disposes.",
                branch = "main",
                daysAgo = 27,
            ) {
                edit("src/main/kotlin/dev/aurora/io/Autosave.kt", delta = 46)
                edit("src/main/kotlin/dev/aurora/util/Disposable.kt", delta = 12)
            }
            aurora.snapshot(
                title = "Quick open scoring rewrite",
                note = "Fuzzy matcher prefers path-segment boundaries; 3x faster on large repos.",
                branch = "main",
                daysAgo = 24,
            ) {
                add("src/main/kotlin/dev/aurora/search/FuzzyMatcher.kt", lines = 168)
                edit("src/main/kotlin/dev/aurora/search/QuickOpen.kt", delta = -22)
                edit("src/test/kotlin/dev/aurora/search/SearchIndexTest.kt", delta = 88)
                remove("src/main/kotlin/dev/aurora/search/SearchIndex.kt")
                add("src/main/kotlin/dev/aurora/search/IncrementalIndex.kt", lines = 203)
            }
            aurora.snapshot(
                title = "Command palette groundwork",
                note = "Registry, fuzzy command matching, and a first pass at the overlay.",
                branch = "feature/command-palette",
                daysAgo = 18,
            ) {
                add("src/main/kotlin/dev/aurora/palette/CommandPalette.kt", lines = 248)
                add("src/main/kotlin/dev/aurora/palette/CommandMatcher.kt", lines = 96)
                edit("src/main/kotlin/dev/aurora/palette/CommandRegistry.kt", delta = 61)
            }
            aurora.snapshot(
                title = "Merge main into command palette",
                note = "Brings in the quick-open rewrite and autosave fix.",
                branch = "feature/command-palette",
                daysAgo = 15,
            ) {
                edit("src/main/kotlin/dev/aurora/palette/CommandPalette.kt", delta = 34)
            }
            aurora.snapshot(
                title = "Add LSP completion provider",
                note = "Completions arrive asynchronously and are filtered before display.",
                branch = "main",
                daysAgo = 12,
            ) {
                add("src/main/kotlin/dev/aurora/lsp/LspSession.kt", lines = 276)
                add("src/main/kotlin/dev/aurora/lsp/DiagnosticsModel.kt", lines = 84)
                edit("src/main/kotlin/dev/aurora/lsp/CompletionProvider.kt", delta = 112)
                edit("src/main/kotlin/dev/aurora/lsp/LanguageClient.kt", delta = 39)
            }
            aurora.snapshot(
                title = "Theme tokens for high contrast",
                note = "Palette split into semantic tokens; contrast checked for AA.",
                branch = "main",
                daysAgo = 9,
            ) {
                edit("src/main/kotlin/dev/aurora/theme/Palette.kt", delta = 57)
                edit("src/main/kotlin/dev/aurora/theme/Typography.kt", delta = 23)
                edit("docs/architecture.md", delta = 44)
            }
            aurora.snapshot(
                title = "Palette keyboard navigation",
                note = "Arrow keys and enter selection, with escape restoring prior focus.",
                branch = "feature/command-palette",
                daysAgo = 6,
            ) {
                edit("src/main/kotlin/dev/aurora/palette/CommandPalette.kt", delta = 78)
                edit("src/main/kotlin/dev/aurora/MainWindow.kt", delta = 26)
            }
            aurora.snapshot(
                title = "Document keybindings",
                note = "Default keymap and palette shortcuts written down.",
                branch = "main",
                daysAgo = 4,
            ) {
                edit("docs/keybindings.md", delta = 62)
                edit("README.md", delta = 19)
            }
            aurora.snapshot(
                title = "Refactor window disposal order",
                note = "Windows dispose children before the root scope closes.",
                branch = "main",
                daysAgo = 2,
            ) {
                edit("src/main/kotlin/dev/aurora/MainWindow.kt", delta = 51)
                edit("src/main/kotlin/dev/aurora/App.kt", delta = 17)
                edit("src/main/kotlin/dev/aurora/util/Logger.kt", delta = 9)
            }
            aurora.snapshot(
                title = "Palette polish before review",
                note = "Empty-query hint, better shadows, and matching scrollbar.",
                branch = "feature/command-palette",
                daysAgo = 1,
                hoursAgo = 6,
            ) {
                edit("src/main/kotlin/dev/aurora/palette/CommandPalette.kt", delta = 44)
                edit("src/main/kotlin/dev/aurora/theme/Palette.kt", delta = 12)
            }

            val lumen = history("lumen", "proj_lumen_02", now)
                .seed(
                    "app/src/main/kotlin/dev/lumen/MainActivity.kt",
                    "app/src/main/kotlin/dev/lumen/notes/NoteEditor.kt",
                    "app/src/main/kotlin/dev/lumen/notes/NoteList.kt",
                    "app/src/main/kotlin/dev/lumen/storage/NoteStore.kt",
                    "app/src/main/res/values/strings.xml",
                    "README.md",
                )
            lumen.snapshot(
                title = "Project created",
                note = "Initial Compose scaffold for the notes app.",
                branch = "main",
                daysAgo = 16,
            )
            lumen.snapshot(
                title = "Local note persistence",
                note = "Notes are saved as JSON in app storage.",
                branch = "main",
                daysAgo = 11,
            ) {
                edit("app/src/main/kotlin/dev/lumen/storage/NoteStore.kt", delta = 93)
                edit("app/src/main/kotlin/dev/lumen/notes/NoteEditor.kt", delta = 47)
            }
            lumen.snapshot(
                title = "Note list sorting",
                note = "Pinned notes first, then most recently edited.",
                branch = "main",
                daysAgo = 7,
            ) {
                edit("app/src/main/kotlin/dev/lumen/notes/NoteList.kt", delta = 64)
                add("app/src/main/kotlin/dev/lumen/notes/NoteSort.kt", lines = 41)
            }
            lumen.snapshot(
                title = "Empty state and copy",
                note = "First-run screen explains how notes are stored locally.",
                branch = "main",
                daysAgo = 3,
            ) {
                edit("app/src/main/res/values/strings.xml", delta = 22)
                edit("app/src/main/kotlin/dev/lumen/notes/NoteList.kt", delta = 38)
            }
            lumen.snapshot(
                title = "Strip unused dependency",
                note = "Removed an animation library the list no longer uses.",
                branch = "main",
                daysAgo = 1,
            ) {
                edit("README.md", delta = 6)
                edit("app/src/main/kotlin/dev/lumen/MainActivity.kt", delta = 4)
            }

            val auroraSnapshots = aurora.snapshots.toList()
            val lumenSnapshots = lumen.snapshots.toList()

            val auroraProject = project(
                id = ProjectId("proj_aurora_01"),
                name = "Aurora Editor",
                description = "Cross-platform code editor experiment",
                activeBranch = "feature/command-palette",
                createdAt = now.minus(120, ChronoUnit.DAYS),
                snapshots = auroraSnapshots,
                storageBytes = 48_213_760L,
            )
            val lumenProject = project(
                id = ProjectId("proj_lumen_02"),
                name = "Lumen Notes",
                description = "Minimal local-first notes app",
                activeBranch = "main",
                createdAt = now.minus(48, ChronoUnit.DAYS),
                snapshots = lumenSnapshots,
                storageBytes = 6_942_310L,
            )
            val labProject = Project(
                id = ProjectId("proj_hexlab_03"),
                name = "hex-lab",
                description = "Scratch project — no snapshots captured yet",
                directory = null,
                createdAt = now.minus(6, ChronoUnit.DAYS),
                activeBranch = "main",
                snapshotCount = 0,
                storageBytes = 0L,
                lastSnapshotAt = null,
            )

            val manifests = HashMap<SnapshotId, Map<String, Fingerprint>>()
            manifests.putAll(aurora.manifests)
            manifests.putAll(lumen.manifests)

            return FakeSeed(
                projects = listOf(auroraProject, lumenProject, labProject),
                snapshots = auroraSnapshots + lumenSnapshots,
                manifests = manifests,
            )
        }

    }
}

private class HistoryBuilder(
    private val slug: String,
    private val projectId: ProjectId,
    private val now: Instant,
) {
    private val working = linkedMapOf<String, Fingerprint>()

    val snapshots = mutableListOf<Snapshot>()
    val manifests = linkedMapOf<SnapshotId, Map<String, Fingerprint>>()

    fun seed(vararg paths: String): HistoryBuilder {
        paths.forEachIndexed { index, path ->
            working[path] = Fingerprint(revision = 1, lines = 40 + (index * 37) % 220)
        }
        return this
    }

    fun snapshot(
        title: String,
        note: String,
        branch: String,
        daysAgo: Long,
        hoursAgo: Long = 0,
        mutate: HistoryBuilder.() -> Unit = {},
    ): Snapshot {
        this.mutate()
        val createdAt = now.minus(daysAgo, ChronoUnit.DAYS).minus(hoursAgo, ChronoUnit.HOURS)
        val id = SnapshotId("snp_${slug}_${idSuffix(slug, snapshots.size)}")
        val snapshot = Snapshot(
            id = id,
            projectId = projectId,
            title = title,
            note = note,
            branch = branch,
            createdAt = createdAt,
            fileCount = working.size,
            sizeBytes = working.values.sumOf { it.lines.toLong() * 48L },
        )
        snapshots += snapshot
        manifests[id] = LinkedHashMap(working)
        return snapshot
    }

    fun add(path: String, lines: Int) {
        working[path] = Fingerprint(revision = 1, lines = lines)
    }

    fun edit(path: String, delta: Int) {
        val current = working[path] ?: return
        working[path] = Fingerprint(revision = current.revision + 1, lines = maxOf(8, current.lines + delta))
    }

    fun remove(path: String) {
        working.remove(path)
    }

    private fun idSuffix(slug: String, index: Int): String {
        val mixed = (slug.hashCode().toLong() * 31L + index) * 2_654_435_761L
        return (mixed and 0xFFFFFFFFL).toString(16).padStart(8, '0')
    }
}
