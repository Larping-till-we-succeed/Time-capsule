package dev.timecapsule.app.ui.common

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import dev.timecapsule.app.core.storage.GrantedDirectory
import dev.timecapsule.app.core.storage.SafProjectDirectoryStore

/**
 * SAF directory picker.
 *
 * The platform chooser is the only way this app obtains filesystem access: no path is
 * constructed and no broad storage permission is requested. The selected tree URI is persisted
 * with `takePersistableUriPermission` before [onPicked] runs, so the grant survives restarts.
 *
 * @return a function that opens the system document tree picker.
 */
@Composable
fun rememberProjectDirectoryPicker(
    store: SafProjectDirectoryStore,
    onPicked: (GrantedDirectory) -> Unit,
    onError: (String) -> Unit = {},
): () -> Unit {
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        val result = runCatching { store.persistGrant(uri) }
        result.onSuccess(onPicked).onFailure { error ->
            onError(error.message ?: "Android did not grant access to that folder.")
        }
    }
    return remember(launcher) { { launcher.launch(null) } }
}
