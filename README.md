# 🌕 Moon Oracle

*A Zero-Split-Brain implementation of lunar awareness.*

The Moon Oracle is a mathematically precise moon phase tracker. It exists to provide a true "Write Once, Run Everywhere" business logic layer using WebAssembly (WASM), ensuring that the lunar context is consistent across all platforms—from Android devices to cloud gateways.

## 🧠 The Philosophy: Zero-Split-Brain

The cardinal rule of this architecture is that **Host Environments are ignorant**. The hosts (Android, Go, Python, etc.) do not know how to calculate a moon phase. They only handle I/O and pass primitive data to the WASM "Brain."

If a bug is found in the lunar calculation, it is fixed in exactly one place: the Rust WASM source in the `brain/` directory.

## 🏗️ Project Structure

- `brain/`: The core logic written in Rust, compiled to WASM.
- `android/`: The **Moon Oracle** mobile application, built with Jetpack Compose. The only host so far.
- `docs/`: Detailed architectural blueprints, including the hosts that are still to be built.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-wasip1` target.
- [Android Studio](https://developer.android.com/studio) (Koala or newer).
- JDK 17+.

### Building the Brain

```bash
cd brain
cargo test
cargo build --target wasm32-wasip1 --release
# Copy the binary to the Android assets
cp target/wasm32-wasip1/release/moon_phase_brain.wasm ../android/app/src/main/assets/moon-phase.wasm
```

### Running the Android App

1. Open the root directory in Android Studio.
2. Sync Gradle.
3. Run the `app` module on an emulator or physical device.

## 🗺️ Roadmap

- **Phase 1** ✅ Shipped: the Rust WASM Brain and the Android app, with a launcher icon that follows the moon.
- **0.1.0** 🚧 In review: true-moon phase timing using real orbital correction terms ([#10](https://github.com/kingdonb/moon-phase-clock/issues/10)). New and full moons land within about half an hour of the published instants, where the mean lunar month could be up to ~14 hours off.
- **Phase 2** 📝 TODO: Iron Town Gateway (Spin API) and Terminal Interface (Go CLI).
- **Phase 3** 📝 TODO: AI Bridge (MCP Server) and K8s Reconciler.
- **Phase 4** 📝 TODO: Smooth animations and deeper OS integration.
- **Someday** 📝 TODO: latitude/longitude inputs for hemisphere-correct orientation, moonrise/moonset, and sunrise/sunset.

See [docs/MOON_PHASE_ARCHITECTURE.md](docs/MOON_PHASE_ARCHITECTURE.md) for what each piece is and which parts exist.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

---
*Status: Brain and Android host shipped; other hosts TODO | Architecture: Zero-Split-Brain*
