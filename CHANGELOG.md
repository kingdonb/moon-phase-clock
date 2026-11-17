# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.8] - 2026-11-17

### Changed
- **Build Toolchain**: Android Gradle Plugin 9.4.0 → 9.5.1 and Gradle 9.7.1 → 9.9.1, following the Android Studio 2026.2 update. Includes the earlier, unreleased move to AGP 9.4 with built-in Kotlin.
- **Cosine Illumination**: The Oracle Brain now computes the illuminated fraction from the phase angle, `(1 - cos θ) / 2`, instead of a linear ramp. A mid-crescent reads about 15% rather than 25%; the quarters and the full moon are unchanged.
- **Finer Illumination**: The brain reports illumination to four decimal places, correctly rounded. Previously one truncated digit meant the app only ever showed multiples of 10% and the drawn terminator moved in ten visible steps.
- **Rounded Percentage**: The app rounds the displayed illumination percentage instead of truncating it.

### Fixed
- **Stale Phase While Open**: The app calculated the phase once at launch. It now recalculates every time it returns to the foreground and once a minute while visible, and asks the brain again before updating the launcher icon on leaving.
- **Background Sync Retries**: `LunarSyncWorker` no longer asks WorkManager to retry with backoff after a failure. The calculation is local and deterministic, so a failed run is reported as failed and the next 6-hourly run tries again.

### Added
- **Brain Test Suite**: `cargo test` covers the cosine approximation, illumination, phase windows, number formatting, the JSON contract and the exported entry point.
- **Tests in CI**: CI now runs the Rust tests, and runs the Android unit tests against the brain it has just built.
- **Release Script**: `scripts/release.sh` and `RELEASING.md` take a release from preflight checks through tests, signing, signature verification and a device check, and ask before tagging. Signing secrets are passed only through the environment and never leave the laptop.
- **Dependabot**: Monthly grouped update pull requests for Gradle, GitHub Actions and Cargo.
- **Minified Release Build in CI**: CI also builds the R8-minified release, so missing keep rules fail the build rather than the app.
- **Session Logs**: `docs/sessions/` records what each working session changed and decided; `CLAUDE.md` points new sessions at it.
- **Documented Status**: The README roadmap and the architecture blueprint now mark what is built and what is still TODO, describe the brain's real interface, and explain what latitude/longitude would (and would not) change.

### Removed
- **Committed Release Bundle**: `android/app/release/app-release.aab` is no longer tracked; signed builds go to the ignored `dist/` directory.
- **Duplicate WASM Binary**: The copy of `moon-phase.wasm` under test resources is gone; unit tests load the binary from `src/main/assets`, the one that ships.

## [0.0.7] - 2026-08-15

### Fixed
- **WorkManager Proguard Keep Rules**: Added explicit R8/Proguard rules for `androidx.work.**` and `androidx.room.**` to prevent startup crash (`Failed to create an instance of class androidx.work.impl.WorkDatabase`) in release builds with code minification enabled.

## [0.0.6] - 2026-08-15

### Added
- **Passive Background Icon Sync**: Added `LunarSyncWorker` via Android `WorkManager` (scheduled every 6 hours / 4x daily) to calculate current moon phase from the WASM brain and update the launcher app icon automatically without requiring manual app launch.
- **Application Startup Enqueue**: Added `MoonPhaseApplication` to arm background periodic sync automatically on process start.

## [0.0.5] - 2026-05-29

### Changed
- **Package Renaming**: Renamed app package and application ID to `com.urmanac.moonphase` to reflect domain ownership.
- **SDK Upgrade**: Upgraded target and compile SDK to API level 35 to meet Google Play Store requirements.
- **R8 Minification**: Enabled code/resource minification for release builds (with obfuscation disabled via Proguard/R8 rules).

## [0.0.4] - 2026-05-16

### Fixed
- **New Moon Icon**: Fixed the default app icon (used for New Moon) which was incorrectly showing a Full Moon.

## [0.0.3] - 2026-05-11

### Fixed
- **App Icon Swap**: Fixed a critical race condition where the app was killed before it could finish disabling old activity aliases, leading to the default icon persisting indefinitely.
- **Icon Refresh**: Improved cleanup logic to ensure only one icon alias is active at a time.

## [0.0.2] - 2026-05-11

### Fixed
- **Moon Graphics**: Corrected drawing logic for waning phases in the UI. Waning crescents and gibbouses now display their correct concave/convex curves.
- **App Icon Shapes**: Fixed inverted sweep flags in vector drawables that caused crescents to appear as gibbouses.
- **First-Run Exit**: Resolved an issue where the app would quit immediately on the first run. Dynamic icon swapping is now deferred until the application is moved to the background (`onStop`), preventing the system from killing the active foreground process.
- **App Icon Visibility**: Enhanced all moon icons with crater details and set the default app icon to a detailed Full Moon for better visibility in system settings and the app drawer.

## [0.0.1] - 2026-05-10


### Added
- **Moon Oracle App**: Initial release of the Android client.
- **Zero-Split-Brain Architecture**: Ported lunar math to a Rust WASM core for cross-platform consistency.
- **Phased Icon System**: The app icon now updates dynamically to reflect the current moon phase.
- **Lunar Visualization**: Custom `MoonView` in Jetpack Compose with crater textures and spherical shadow rendering.
- **The Oracle Brain**: A `no_std` Rust library for precise moon phase and "Torment Multiplier" calculations.
- **WASM Integration**: Embedded WASM execution on Android via the Chicory interpreter.
- **CI/CD**: Initial GitHub Actions workflow for building the WASM brain and Android APK.

### Changed
- Renamed project from "Moon Phase" to **Moon Oracle**.
- Updated documentation to reflect the Oracle's role within the broader ecosystem.

### Fixed
- UI text clipping in the Torment Gauge.
- Improved terminator line math for more realistic lunar crescents.
