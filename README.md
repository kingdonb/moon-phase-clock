# 🌕 The Useless Moon Phase Clock

*An implementation of the "Zero-Split-Brain" Vind-Box Architecture, infused with the lore of the Mecris Torment Matrix.*

This project implements a highly over-engineered, mathematically precise Moon Phase Clock. It exists to prove the viability of a true "Write Once, Run Everywhere" business logic layer using WebAssembly (WASM), while providing environmental context to the Mecris accountability ecosystem.

## 🧠 The Core Philosophy: Zero-Split-Brain

The cardinal rule of this architecture is that **Host Environments are ignorant**. The hosts (Android, Go, Python, etc.) do not know how to calculate a moon phase. They do not hold state logic. They only handle I/O (Network, Time, Disk) and pass primitive data to the "Brain."

If a bug is found in the lunar calculation, it is fixed in exactly one place: the Rust WASM source in the `brain/` directory.

## 🌌 Lore Integration: The Lunar Torment

The Moon Phase Clock is not just an aesthetic toy; it actively influences the severity of the Mecris accountability system via the **Torment Multiplier**.

- **🌑 New Moon**: The system is quiet. Multiplier is 1.0.
- **🌕 Full Moon**: The Torment Multiplier maxes out at 3.0. The system demands perfection.

## 🏗️ Project Structure

- `brain/`: The core logic written in Rust, compiled to WASM.
- `android/`: A Jetpack Compose mobile application that embeds the WASM brain using the [Chicory](https://github.com/dylibso/chicory) interpreter.
- `docs/`: Detailed architectural blueprints and lore.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-wasip1` target.
- [Android Studio](https://developer.android.com/studio) (Koala or newer).
- JDK 17+.

### Building the Brain

```bash
cd brain
cargo build --target wasm32-wasip1 --release
# Copy the binary to the Android assets
cp target/wasm32-wasip1/release/moon_phase_brain.wasm ../android/app/src/main/assets/moon-phase.wasm
```

### Running the Android App

1. Open the root directory in Android Studio.
2. Sync Gradle.
3. Run the `app` module on an emulator or physical device.

## 📝 Changelog

### [0.0.1] - 2026-05-10
#### Added
- **The Brain**: Core lunar logic implemented in Rust (`no_std`) and compiled to WASM.
- **Android App**: Jetpack Compose mobile client with dark mode support.
- **WASM Integration**: Pure-Java WASM interpretation via Chicory.
- **Lunar Visualization**: Custom `MoonView` with dynamic phase rendering and crater textures.
- **Mecris Integration**: Implementation of the "Torment Multiplier" based on lunar phases.
- **Documentation**: Comprehensive architecture guide, LICENSE (MIT), and README.

#### Fixed
- Resolved text clipping in the "Torment Gauge" UI.
- Corrected lunar terminator math for proper spherical rendering.

## 🗺️ Roadmap

### Phase 2: The Infrastructure Expansion
- **Iron Town Gateway (Spin API)**: Deploy a Rust-based serverless API using Fermyon Spin to provide global lunar context.
- **The Terminal Interface (Go CLI)**: Implement `moonctl`, a Go-based CLI that embeds the WASM brain for offline calculations.

### Phase 3: The Ecosystem Integration
- **The Reconciler (K8s Operator)**: A Go-based Kubernetes controller to make clusters "lunar-aware."
- **AI Bridge (MCP Server)**: A Python FastMCP server to provide lunar context to LLMs during Mecris sessions.

### Phase 4: Lore Depth
- **Torment Escalation**: Link the Android app's Torment Multiplier to local notification severity.
- **Visual Polish**: Add smooth animations for phase transitions in the `MoonView`.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

---
*Status: Initialized | Architecture: Zero-Split-Brain / Useless*
