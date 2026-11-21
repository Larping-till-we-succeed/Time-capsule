package dev.timecapsule.app.core.storage

import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import androidx.documentfile.provider.DocumentFile
import dev.timecapsule.app.core.rust.DirectoryDescription
import dev.timecapsule.app.core.rust.ProjectStorageGateway
import dev.timecapsule.app.data.model.ProjectId
import java.util.concurrent.ConcurrentHashMap

/**
 * A directory the user granted access to with Android's Storage Access Framework.
 *
 * [uri] is the persisted `content://` tree URI. There is no filesystem path: the app has no
 * right to construct one, and the Rust adapter must go through [ProjectStorageGateway].
 */
data class GrantedDirectory(
    val displayName: String,
    val uri: String,
)

/**
 * Owns SAF directory grants and the local project-to-directory associations.
 *
 * Grants are recorded in SharedPreferences so the UI can show them after a restart, but the
 * source of truth for whether access still exists is
 * [ContentResolver.getPersistedUriPermissions] — stale entries are pruned on read.
 *
 * Linking a directory to a project only records a device-local association. It does not index
 * or copy anything; that is the Rust core's job once integrated.
 */
class SafProjectDirectoryStore(context: Context) {

    private val appContext = context.applicationContext
    private val prefs = appContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    /** Returns still-valid grants, dropping any whose permission was revoked outside the app. */
    fun grantedDirectories(): List<GrantedDirectory> {
        val persisted = appContext.contentResolver.persistedUriPermissions
            .filter { it.isReadPermission && it.isWritePermission }
            .map { it.uri.toString() }
            .toSet()

        val known = prefs.getStringSet(KEY_GRANTS, emptySet()).orEmpty()
        val stale = known - persisted
        if (stale.isNotEmpty()) {
            prefs.edit().putStringSet(KEY_GRANTS, known - stale).apply()
        }
        return (known intersect persisted)
            .map { uri -> GrantedDirectory(displayName = prefs.getString(nameKey(uri), null) ?: displayName(uri), uri = uri) }
            .sortedBy { it.displayName.lowercase() }
    }

    /** Persists the grant behind [uri] and remembers its display name. */
    fun persistGrant(uri: Uri): GrantedDirectory {
        val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
        appContext.contentResolver.takePersistableUriPermission(uri, flags)

        val name = displayName(uri.toString())
        prefs.edit()
            .putStringSet(KEY_GRANTS, prefs.getStringSet(KEY_GRANTS, emptySet()).orEmpty() + uri.toString())
            .putString(nameKey(uri.toString()), name)
            .apply()
        return GrantedDirectory(displayName = name, uri = uri.toString())
    }

    /** Releases a grant and removes any project links that pointed at it. */
    fun releaseGrant(uri: String) {
        runCatching {
            appContext.contentResolver.releasePersistableUriPermission(
                Uri.parse(uri),
                Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION,
            )
        }
        val links = prefs.getStringSet(KEY_LINKS, emptySet()).orEmpty()
            .filterNot { it.endsWith(LINK_SEPARATOR + uri) }
        prefs.edit()
            .putStringSet(KEY_GRANTS, prefs.getStringSet(KEY_GRANTS, emptySet()).orEmpty() - uri)
            .putStringSet(KEY_LINKS, links.toSet())
            .remove(nameKey(uri))
            .apply()
    }

    fun linkProject(projectId: ProjectId, directory: GrantedDirectory) {
        val links = prefs.getStringSet(KEY_LINKS, emptySet()).orEmpty()
            .filterNot { it.startsWith(projectId.value + LINK_SEPARATOR) }
            .toMutableSet()
        links += projectId.value + LINK_SEPARATOR + directory.uri
        prefs.edit().putStringSet(KEY_LINKS, links).apply()
    }

    fun linkedDirectory(projectId: ProjectId): GrantedDirectory? {
        val uri = prefs.getStringSet(KEY_LINKS, emptySet()).orEmpty()
            .firstOrNull { it.startsWith(projectId.value + LINK_SEPARATOR) }
            ?.substringAfter(LINK_SEPARATOR)
            ?: return null
        if (grantedDirectories().none { it.uri == uri }) return null
        return GrantedDirectory(displayName = prefs.getString(nameKey(uri), null) ?: displayName(uri), uri = uri)
    }

    fun unlinkProject(projectId: ProjectId) {
        val links = prefs.getStringSet(KEY_LINKS, emptySet()).orEmpty()
            .filterNot { it.startsWith(projectId.value + LINK_SEPARATOR) }
        prefs.edit().putStringSet(KEY_LINKS, links.toSet()).apply()
    }

    private fun displayName(uri: String): String {
        val parsed = Uri.parse(uri)
        val tree = runCatching { DocumentsContract.getTreeDocumentId(parsed) }.getOrNull()
        return tree?.substringAfterLast(':')?.takeIf { it.isNotBlank() }
            ?: parsed.lastPathSegment?.substringAfterLast('/')?.takeIf { it.isNotBlank() }
            ?: "Selected folder"
    }

    private fun nameKey(uri: String) = "$KEY_NAMES_PREFIX$uri"

    private companion object {
        const val PREFS_NAME = "timecapsule.directories"
        const val KEY_GRANTS = "grants"
        const val KEY_LINKS = "links"
        const val KEY_NAMES_PREFIX = "name:"
        const val LINK_SEPARATOR = "|"
    }
}

/**
 * SAF implementation of the storage gateway the Rust adapter will use.
 *
 * Enumerating documents is real work against the granted tree; reading file *contents* into
 * the core is deliberately left to the future UniFFI adapter, which must stream bytes through
 * the ContentResolver rather than assume a path.
 */
class SafProjectStorageGateway(context: Context) : ProjectStorageGateway {

    private val appContext = context.applicationContext
    private val descriptionCache = ConcurrentHashMap<String, DirectoryDescription>()

    override suspend fun describe(directoryUri: String): DirectoryDescription {
        descriptionCache[directoryUri]?.let { return it }
        val tree = runCatching { DocumentFile.fromTreeUri(appContext, Uri.parse(directoryUri)) }
            .getOrNull()
            ?: return DirectoryDescription(
                displayName = "Unavailable folder",
                entryCount = 0,
                canRead = false,
                canWrite = false,
            )

        val description = DirectoryDescription(
            displayName = tree.name ?: "Selected folder",
            entryCount = tree.listFiles().size,
            canRead = tree.canRead(),
            canWrite = tree.canWrite(),
        )
        descriptionCache[directoryUri] = description
        return description
    }

    override suspend fun listTopLevelEntries(directoryUri: String): List<String> {
        val tree = DocumentFile.fromTreeUri(appContext, Uri.parse(directoryUri)) ?: return emptyList()
        return tree.listFiles()
            .mapNotNull { it.name }
            .sortedBy { it.lowercase() }
    }
}
