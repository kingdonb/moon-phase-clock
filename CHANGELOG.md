# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
