package dev.timecapsule.app.data.repository

import dev.timecapsule.app.data.model.ProjectId
import dev.timecapsule.app.data.model.SnapshotId
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.ZoneOffset
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.assertThrows
import org.junit.Test

class InMemoryTimeCapsuleRepositoryTest {

    private val now = Instant.parse("2026-11-21T12:00:00Z")
    private val repository = InMemoryTimeCapsuleRepository(
        clock = Clock.fixed(now, ZoneOffset.UTC),
        simulatedLatency = Duration.ZERO,
    )

    @Test
    fun listProjects_returnsThreeDemoProjects() {
        val projects = kotlinx.coroutines.runBlocking { repository.listProjects() }
        assertEquals(3, projects.size)
        assertTrue(projects.any { it.name == "Aurora Editor" })
        assertTrue(projects.any { it.name == "hex-lab" && it.snapshotCount == 0 })
    }

    @Test
    fun listSnapshots_areNewestFirstAndConsistentWithProject() {
        val project = kotlinx.coroutines.runBlocking { repository.listProjects() }.first { it.name == "Aurora Editor" }
        val snapshots = kotlinx.coroutines.runBlocking { repository.listSnapshots(project.id) }
        assertEquals(project.snapshotCount, snapshots.size)
        for (i in 0 until snapshots.size - 1) {
            assertTrue(snapshots[i].createdAt.isAfter(snapshots[i + 1].createdAt))
        }
    }

    @Test
    fun compareSnapshots_groupsEveryChange() {
        val project = kotlinx.coroutines.runBlocking { repository.listProjects() }.first { it.name == "Aurora Editor" }
        val snapshots = kotlinx.coroutines.runBlocking { repository.listSnapshots(project.id) }
        val oldest = snapshots.last().id
        val newest = snapshots.first().id

        val diff = kotlinx.coroutines.runBlocking {
            repository.compareSnapshots(project.id, oldest, newest)
        }

        assertEquals(oldest, diff.from)
        assertEquals(newest, diff.to)
        assertEquals(
            diff.changes.size,
            diff.added.size + diff.modified.size + diff.deleted.size,
        )
        assertTrue(diff.changes.isNotEmpty())
        // The fake repository never fabricates patch text.
        assertEquals(dev.timecapsule.app.data.model.TextualDiff.NotAvailable, diff.textualDiff)
    }

    @Test
    fun compareSnapshots_rejectsIdenticalSnapshots() {
        val project = kotlinx.coroutines.runBlocking { repository.listProjects() }.first { it.name == "Lumen Notes" }
        val snapshots = kotlinx.coroutines.runBlocking { repository.listSnapshots(project.id) }
        assertThrows(RepositoryException.InvalidComparison::class.java) {
            kotlinx.coroutines.runBlocking {
                repository.compareSnapshots(project.id, snapshots.first().id, snapshots.first().id)
            }
        }
    }

    @Test
    fun getStatistics_activityCoversFourteenDays() {
        val project = kotlinx.coroutines.runBlocking { repository.listProjects() }.first { it.name == "Lumen Notes" }
        val statistics = kotlinx.coroutines.runBlocking { repository.getStatistics(project.id) }
        assertEquals(14, statistics.activity.size)
        assertEquals(project.snapshotCount, statistics.snapshotCount)
        assertEquals(project.storageBytes, statistics.storageBytes)
        assertTrue(statistics.activity.all { it.snapshotCount >= 0 })
        assertTrue(statistics.activity.size == 14)
    }

    @Test
    fun getProject_throwsForUnknownId() {
        assertThrows(RepositoryException.ProjectNotFound::class.java) {
            kotlinx.coroutines.runBlocking { repository.getProject(ProjectId("does-not-exist")) }
        }
    }

    @Test
    fun branches_markActiveBranch() {
        val project = kotlinx.coroutines.runBlocking { repository.listProjects() }.first { it.name == "Aurora Editor" }
        val branches = kotlinx.coroutines.runBlocking { repository.listBranches(project.id) }
        assertTrue(branches.any { it.isActive && it.name == project.activeBranch })
    }

    @Test
    fun getSnapshot_throwsForUnknownSnapshot() {
        val project = kotlinx.coroutines.runBlocking { repository.listProjects() }.first { it.name == "Lumen Notes" }
        assertThrows(RepositoryException.SnapshotNotFound::class.java) {
            kotlinx.coroutines.runBlocking {
                repository.getSnapshot(project.id, SnapshotId("nope"))
            }
        }
    }
}
