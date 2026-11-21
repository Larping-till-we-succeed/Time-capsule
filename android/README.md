# TimeCapsule for Android

Native Android client for TimeCapsule, built with Kotlin, Jetpack Compose, Material 3,
Gradle Kotlin DSL and an Android Studio-compatible project layout.

The Android app is intentionally **not** connected to the Rust engine yet. It ships a
documented in-memory repository so the UI, navigation and data flow can be developed and
tested independently. The Rust core remains untouched in `src/`.

## Open in Android Studio

1. Open Android Studio.
2. Choose **Open** and select the `android/` directory of this repository.
3. Wait for Gradle sync to finish. The project uses the Gradle wrapper; no separate
   Gradle installation is required.
4. Run the `app` configuration on an emulator or device (API 26+).

The Android SDK must be installed. This project targets `compileSdk 36` and `minSdk 26`.

## Build from the command line

From the `android/` directory:

```sh
./gradlew :app:assembleDebug
```

The debug APK is written to:

```
android/app/build/outputs/apk/debug/app-debug.apk
```

Run the unit tests with:

```sh
./gradlew :app:testDebugUnitTest
```

Run both together with:

```sh
./gradlew clean testDebugUnitTest assembleDebug
```

## Architecture

```
android/app/src/main/java/dev/timecapsule/app/
├── TimeCapsuleApplication.kt   # owns AppContainer
├── MainActivity.kt            # edge-to-edge host
├── TimeCapsuleApp.kt          # navigation shell (bottom bar / rail)
├── data/
│   ├── AppContainer.kt        # single place where implementations are chosen
│   ├── model/                 # pure-Kotlin domain models
│   ├── repository/            # TimeCapsuleRepository + in-memory implementation
│   └── preferences/           # SharedPreferences-backed stores
├── core/
│   ├── rust/                  # proposed TimeCapsuleCore contract (not implemented)
│   └── storage/               # SAF directory grants and storage gateway
└── ui/
    ├── theme/                 # Material 3 dark scheme, typography, semantic colors
    ├── common/                # reusable components, selectors, formatters
    ├── navigation/            # destinations
    └── screens/               # Dashboard, Timeline, Compare, Statistics, Settings
```

Key design points:

- The UI depends only on `TimeCapsuleRepository` and `AppContainer`. Swapping the in-memory
  repository for a UniFFI-backed one happens in `AppContainer` and nowhere else.
- Models are pure Kotlin so they can be unit-tested on the JVM.
- `TimeCapsuleCore` is a documented Kotlin interface that mirrors the proposed Rust
  operations. `NotIntegratedTimeCapsuleCore` throws `CoreNotIntegratedException` instead of
  returning fake native results.
- No `System.loadLibrary` call exists anywhere in the app.

## Implemented screens

- **Dashboard** — project overview, snapshot/storage/branch/last-capture metrics, recent
  snapshots, project folder link, engine status, demo-data banner.
- **Timeline** — chronological snapshot list grouped by day, branch filter pills, expandable
  snapshot details, empty and error states.
- **Compare** — two snapshot selectors, added/modified/deleted summaries, changed-file list,
  explicit "textual diff not available" placeholder.
- **Statistics** — totals, 14-day activity chart drawn with Compose `Canvas`, history span,
  branch breakdown.
- **Settings** — automatic-snapshot preferences, retention, SAF folder grants with
  add/remove/link, engine status, appearance, about.

Navigation uses a `NavHost` with a bottom navigation bar on compact screens and a
navigation rail on wide screens.

## Mock repository limitations

`InMemoryTimeCapsuleRepository` is clearly identified as `DataSource.MOCK`:

- Projects, snapshots and file manifests live only in RAM.
- Nothing is read from or written to the device's project directories.
- No snapshot is ever created, restored or compared by a real engine.
- Diff file lists are derived from synthetic manifests so the UI can be exercised honestly,
  but they are not real diffs and no patch text is produced.
- Automatic-snapshot preferences are persisted with SharedPreferences but are not consumed
  by any engine.

The UI marks this state with a "Demo data" banner and per-screen notices. Do not present
demo snapshots as real project history.

## Planned UniFFI integration

See `docs/android-rust-api.md` for the proposed Kotlin/Rust contract and the unresolved
decisions that need coordination with the Rust contributor.

High-level plan:

1. Add UniFFI-generated Kotlin bindings for the Rust core to the app module.
2. Implement `TimeCapsuleRepository` and `TimeCapsuleCore` on top of the generated bindings
   in a new `UniffiTimeCapsuleRepository`.
3. Construct that implementation in `AppContainer` instead of `InMemoryTimeCapsuleRepository`.
4. Bridge SAF `content://` URIs to the core through `ProjectStorageGateway` (the core cannot
   open Android content URIs itself).
5. Remove or gate the demo banner once a real core is linked and verified.

## Android storage access (SAF)

The app never requests broad storage permissions and never constructs filesystem paths:

- Folder access is obtained exclusively through `ActivityResultContracts.OpenDocumentTree()`.
- Grants are persisted with `ContentResolver.takePersistableUriPermission(...)`.
- Persisted grants are reconciled against `ContentResolver.getPersistedUriPermissions()` on
  read, so revoked grants are pruned.
- Granted directories are described through `DocumentFile` (top-level entry counts, read/write
  capability) for diagnostics.
- Project-to-folder links are stored as device-local associations; they do not index anything.

This matches Android's scoped-storage model and keeps the door open for passing content to the
Rust core through a storage abstraction rather than through paths.
EOF
echo written