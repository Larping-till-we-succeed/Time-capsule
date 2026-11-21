package dev.timecapsule.app.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import dev.timecapsule.app.data.AppContainer

/** Provides the dependency container to the composition without a DI framework. */
val LocalAppContainer = staticCompositionLocalOf<AppContainer> {
    error("No AppContainer provided. Wrap the UI in TimeCapsuleApp(container = ...).")
}

/**
 * Creates a screen ViewModel wired to [AppContainer].
 *
 * The factory is remembered per container instance, so recomposition never rebuilds the
 * ViewModel and the container stays the single place where implementations are chosen.
 */
@Composable
inline fun <reified VM : ViewModel> appViewModel(crossinline create: (AppContainer) -> VM): VM {
    val container = LocalAppContainer.current
    val factory = remember(container) {
        viewModelFactory {
            initializer { create(container) }
        }
    }
    return viewModel<VM>(factory = factory)
}
