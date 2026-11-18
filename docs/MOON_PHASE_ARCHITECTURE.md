# 🌕 The Useless Moon Phase Clock: Architecture Blueprint

*An implementation of the "Zero-Split-Brain" Vind-Box Architecture, infused with the lore of the Mecris Torment Matrix.*

This project implements a highly over-engineered, mathematically precise Moon Phase Clock. It exists to prove the viability of a true "Write Once, Run Everywhere" business logic layer using WebAssembly (WASM) Component Models, while providing environmental context to the Mecris accountability ecosystem.

## 1. The Core Philosophy: Zero-Split-Brain
The cardinal rule of this architecture is that **Host Environments are ignorant**. The hosts (Go, Python, Kotlin, Spin) do not know how to calculate a moon phase. They do not hold state logic. They only handle I/O (Network, Time, Disk) and pass primitive data to the Brain.

If a bug is found in the lunar calculation, it is fixed in exactly one place: the Rust WASM source. 

## 2. The Components

This blueprint describes where the project is going as well as where it is. Each component carries a status:

| Component | Status |
|---|---|
| 🧠 The Oracle (WASM Brain) | ✅ Built: phase, illumination, Torment Multiplier. Location inputs and next full moon are 📝 TODO |
| 📱 The Mobile Scent (Android) | ✅ Built: embeds the Brain; dashboard and phased launcher icon. Home-screen widget is 📝 TODO |
| ☁️ The Iron Town Gateway (Spin) | 📝 TODO |
| 💻 The Terminal Interface (Go CLI) | 📝 TODO |
| 🚜 The Reconciler (Kubernetes) | 📝 TODO |
| 🤖 The AI Bridge (MCP Server) | 📝 TODO |
| Lunar Torment wired into Mecris | 📝 TODO: the multiplier is calculated and displayed, but nothing in Mecris consumes it yet |

### 🧠 1. The Oracle (The WASM Brain)
- **Language**: Rust
- **Target**: `wasm32-wasip1`. The Brain imports nothing from WASI, so any core-WASM runtime can host it.
- **Function**: Pure, deterministic math, `no_std`, no dependencies.

**Built today.** One export, `calculate_moon_phase(timestamp: f64)`, takes a UNIX timestamp in seconds and returns a pointer to a NUL-terminated JSON string in the Brain's memory:

```json
{"phase_name": "First Quarter", "torment_multiplier": 1.5, "illumination": 0.5336}
```

  - `phase_name`: New Moon, Waxing Crescent, First Quarter, Waxing Gibbous, Full Moon, Waning Gibbous, Last Quarter, Waning Crescent
  - `illumination`: lit fraction of the disc, 0.0000 to 1.0000, from the Sun–Moon elongation θ: `(1 - cos θ) / 2`
  - `torment_multiplier`: **The Torment Multiplier**, an intensity factor (1.0 to 3.0) that peaks during the Full Moon

The position in the lunar cycle is carried at full `f64` precision; only the printed illumination is rounded, to four decimals.

**Known limits.**
  - The position in the cycle is the Moon's true elongation from the Sun. It uses the mean elongation plus the six largest periodic correction terms (Meeus, *Astronomical Algorithms*, ch. 48). Tested against 2026's published new moons, full moons and eclipses, it is within about half an hour, and usually within minutes. A full lunar theory would be closer still; that is not planned.
  - Time is treated as UTC. The ~70 s offset from Terrestrial Time is far below the model's accuracy.
  - Pre-2000 timestamps are not a goal, and will not be until time travel ships. They happen to work now, because nothing depends on a reference new moon any more.

**📝 TODO.**
  - **Next Full Moon timestamp** in the output.
  - **Geographic coordinates (Lat/Lon) as input.** Location does *not* change the phase: the whole Earth sees the same phase and the same lit fraction at the same instant. What it does change, and what these inputs are for:
    - which way up the moon appears. The Southern Hemisphere sees it rotated, with a waxing moon lit on the left; the app currently draws the Northern Hemisphere view.
    - moonrise and moonset times.
    - sunrise and sunset times. These depend only on the date and the observer's position, so the same pure-math Brain can calculate them. It would need a few more trigonometric functions, written in-house or by admitting a first dependency.
  - **Component Model interface.** `brain/wit/world.wit` is a placeholder for this and is not used by the build.

### ☁️ 2. The Iron Town Gateway (Spin API Service) — 📝 TODO
- **Language**: Rust (Fermyon Spin framework)
- **Function**: Serverless cloud gateway. It provides the REST API endpoints (`/api/moon/phase`). It handles the HTTP request, fetches the current system time, calls the WASM Brain, and returns the response.

### 🚜 3. The Reconciler (Kubernetes Controller / Operator) — 📝 TODO
- **Language**: Go (Kubebuilder)
- **Function**: Watches a Custom Resource Definition (CRD) called `LunarTracker`. It runs a reconciliation loop where it loads the WASM Brain (via `wasmtime-go`), calculates the phase, and writes it back into the CRD's `Status.Conditions`. It allows the Kubernetes cluster to be "aware" of the moon phase for scaling or triggering other cron jobs.

### 💻 4. The Terminal Interface (Go CLI) — 📝 TODO
- **Language**: Go (Cobra)
- **Function**: A command-line tool (`moonctl`). It embeds the exact same WASM binary. You can run `moonctl phase --time "2026-10-31"` completely offline to query the Oracle locally. 

### 📱 5. The Mobile Scent (Android Client)
- **Language**: Kotlin / Jetpack Compose
- **Function**: A dark-themed dashboard that visualizes the current phase and clearly displays the active "Torment Multiplier."

**Built today.**
  - Embeds the Brain and runs it on-device with [Chicory](https://github.com/dylibso/chicory), a pure-Java WASM interpreter. No network, no native libraries.
  - The dashboard recalculates each time it returns to the foreground and once a minute while it is open.
  - The launcher icon follows the phase. It is updated when the user leaves the app and by a background job every 6 hours.

**📝 TODO.**
  - A home-screen widget.
  - Polling the Spin API as an alternative to the embedded Brain, once the Gateway exists.
  - Chicory's `wasi` module is on the classpath but unused; it is there for the day the Brain needs a WASI import.

### 🤖 6. The AI Bridge (Python MCP Server) — 📝 TODO
- **Language**: Python (FastMCP)
- **Function**: Exposes the `get_lunar_context` tool. When Claude or Gemini are orienting themselves in a Mecris session, they query this MCP. 

## 3. Lore Integration: The Lunar Torment
The Moon Phase Clock is not just an aesthetic toy; it is meant to influence the severity of the Mecris accountability system.

> **Status:** the Brain calculates the multiplier and the Android app displays it. The effects on Mecris described below (Nag Ladder tiers, Review Pump scaling) are 📝 TODO; they arrive with the AI Bridge.

- **🌑 New Moon (The Silent Forest)**: The system is quiet. The Nag Ladder starts at Tier 1. The Review Pump multiplier is 1.0. Forgiveness is possible.
- **🌓 Waxing (The Building Pressure)**: The ghost archivist becomes more active. The Review Pump multiplier scales up (1.2 - 1.8).
- **🌕 Full Moon (The Boar God's Charge)**: The Torment Multiplier maxes out at 3.0. Any missed Beeminder goals bypass Tier 1 and immediately trigger Tier 3 escalated interventions. The system demands perfection.
- **🌗 Waning (The Receding Tide)**: Pressure slowly releases back to baseline.

## 4. Development Workflow
1. Write and test the Rust math in the `brain/` directory.
2. Compile to `.wasm`.
3. Distribute the `.wasm` binary to each host. Today that is one copy, `android/app/src/main/assets/moon-phase.wasm`; the `api/`, `controller/`, and `cli/` directories are 📝 TODO.
4. Each host implements a minimal wrapper to execute the binary.

CI runs the Brain's tests, builds the `.wasm`, and runs the Android unit tests against that fresh build.

---
*Status: Brain and Android host shipped; other hosts TODO | Architecture: Zero-Split-Brain / Useless*
