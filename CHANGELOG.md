# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
