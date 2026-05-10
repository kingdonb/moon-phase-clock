# 🌕 Moon Oracle

*A Zero-Split-Brain implementation of lunar awareness.*

The Moon Oracle is a mathematically precise moon phase tracker. It exists to provide a true "Write Once, Run Everywhere" business logic layer using WebAssembly (WASM), ensuring that the lunar context is consistent across all platforms—from Android devices to cloud gateways.

## 🧠 The Philosophy: Zero-Split-Brain

The cardinal rule of this architecture is that **Host Environments are ignorant**. The hosts (Android, Go, Python, etc.) do not know how to calculate a moon phase. They only handle I/O and pass primitive data to the WASM "Brain."

If a bug is found in the lunar calculation, it is fixed in exactly one place: the Rust WASM source in the `brain/` directory.

## 🏗️ Project Structure

- `brain/`: The core logic written in Rust, compiled to WASM.
- `android/`: The **Moon Oracle** mobile application, built with Jetpack Compose.
- `docs/`: Detailed architectural blueprints.

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

## 🗺️ Roadmap

- **Phase 2**: Iron Town Gateway (Spin API) and Terminal Interface (Go CLI).
- **Phase 3**: AI Bridge (MCP Server) and K8s Reconciler.
- **Phase 4**: Smooth animations and deeper OS integration.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

---
*Status: Initialized | Architecture: Zero-Split-Brain*
