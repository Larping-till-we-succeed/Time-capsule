package dev.timecapsule.app.ui.common

/**
 * Loading / content / failure state for a single screen section.
 *
 * "Empty" is not a separate variant: a successful load of an empty list is content, and the
 * screen decides which empty state to render. That keeps empty-state copy next to the layout
 * that needs it.
 */
sealed interface UiState<out T> {
    data object Loading : UiState<Nothing>

    data class Content<T>(val value: T) : UiState<T>

    data class Failure(val message: String) : UiState<Nothing>
}

inline fun <T, R> UiState<T>.map(transform: (T) -> R): UiState<R> = when (this) {
    is UiState.Loading -> UiState.Loading
    is UiState.Content -> UiState.Content(transform(value))
    is UiState.Failure -> this
}
