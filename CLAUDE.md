# Moon Oracle (moon-phase-clock)

Rust WASM "brain" (`brain/`) plus one Android host (`android/`). Read `README.md`, then `docs/MOON_PHASE_ARCHITECTURE.md` for what is built and what is TODO.

## Start of a session

1. Read the newest file in `docs/sessions/`. It says where things were left and what was decided.
2. `gh issue list` and `gh pr list`. Work is planned in GitHub Issues; Dependabot PRs arrive monthly.
3. `git status`. Releases are cut from `main`.

## Rules agreed with Kingdon

- Releases follow `RELEASING.md` via `scripts/release.sh`. Never tag, sign, or bump a version without Kingdon's go-ahead in the current session. Never put the keystore or passphrase anywhere CI can see.
- Open PRs freely; Kingdon reviews and merges.
- Brain changes: edit `brain/src/lib.rs`, run `cargo test`, rebuild the WASM, and copy it to `android/app/src/main/assets/moon-phase.wasm`. The release script refuses a stale copy.
- Timestamps before 2000 are deliberately unsupported.
- Changes to what the brain reports at a given instant (for example true-moon timing, #10) are minor releases (0.x.0), not patches.
- Unbuilt ideas stay in the docs, marked 📝 TODO. Don't delete them, and don't build them unscheduled.
- Sign issues, PRs and session logs with the model name, e.g. "— Claude-Opus-5.5".

## End of a session

Write or extend `docs/sessions/YYYY-MM-DD.md`: what changed (with PR/issue numbers), what was decided, what was left and why, and the first thing to do next time.
