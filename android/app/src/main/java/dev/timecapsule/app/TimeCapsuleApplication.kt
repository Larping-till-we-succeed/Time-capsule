package dev.timecapsule.app

import android.app.Application
import dev.timecapsule.app.data.AppContainer

/**
 * Application entry point. Owns the single [AppContainer] instance so screens never
 * construct repositories themselves — swapping the fake repository for a UniFFI-backed
 * implementation happens here and nowhere else.
 */
class TimeCapsuleApplication : Application() {

    lateinit var container: AppContainer
        private set

    override fun onCreate() {
        super.onCreate()
        container = AppContainer(applicationContext)
    }
}
