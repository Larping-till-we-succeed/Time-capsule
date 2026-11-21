package dev.timecapsule.app.data

import android.content.Context
import dev.timecapsule.app.core.rust.CoreAvailability
import dev.timecapsule.app.core.rust.NotIntegratedTimeCapsuleCore
import dev.timecapsule.app.core.rust.ProjectStorageGateway
import dev.timecapsule.app.core.rust.TimeCapsuleCore
import dev.timecapsule.app.core.storage.SafProjectDirectoryStore
import dev.timecapsule.app.core.storage.SafProjectStorageGateway
import dev.timecapsule.app.data.preferences.SelectedProjectStore
import dev.timecapsule.app.data.preferences.UserPreferencesStore
import dev.timecapsule.app.data.repository.InMemoryTimeCapsuleRepository
import dev.timecapsule.app.data.repository.TimeCapsuleRepository

/**
 * Manual dependency container.
 *
 * This is the only place that decides which repository implementation the app uses. When the
 * Rust core arrives, build a `UniffiTimeCapsuleRepository` here — no screen needs to change.
 * A DI framework would add a dependency for a handful of objects, so it is intentionally absent.
 */
class AppContainer(context: Context) {

    val repository: TimeCapsuleRepository = InMemoryTimeCapsuleRepository()

    val projectDirectories: SafProjectDirectoryStore = SafProjectDirectoryStore(context)

    val storageGateway: ProjectStorageGateway = SafProjectStorageGateway(context)

    val preferences: UserPreferencesStore = UserPreferencesStore(context)

    val selectedProject: SelectedProjectStore = SelectedProjectStore(context)

    /**
     * No UniFFI bindings are compiled in this build. `core` is an explicit failing stand-in,
     * never a fake native implementation.
     */
    val coreAvailability: CoreAvailability = CoreAvailability.NotIntegrated

    val core: TimeCapsuleCore = NotIntegratedTimeCapsuleCore
}
