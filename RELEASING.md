# Releasing Moon Oracle

One script does the mechanical work; a human approves the three things only a human should: unlocking the signing key, confirming the build works on a phone, and publishing.

## Before you start

- Everything for the release is merged to `main`, and CI on `main` is green.
- `CHANGELOG.md` has a `## [x.y.z] - YYYY-MM-DD` section at the top (rename `[Unreleased]`).
- `android/app/build.gradle.kts` has the matching `versionName` and a `versionCode` one higher than the last release.

The usual way is a `release/x.y.z` pull request containing only those two edits; see #13 for 0.0.8.

## Run it

```bash
git checkout main && git pull
scripts/release.sh --dry-run   # optional: every check and test, nothing signed
scripts/release.sh
```

The script:

1. **Preflight.** On `main`, clean, in sync with `origin`; the version, versionCode and changelog agree; the tag doesn't exist yet. `targetSdk` must be no more than one below the newest Android platform installed in the SDK, because Play rejects releases that target an old API level (0.0.8 was rejected for targeting 35). Keep the SDK current through Android Studio's SDK Manager so the check stays meaningful.
2. **Tests.** Runs the brain's Rust tests and rebuilds it. Fails if the committed `moon-phase.wasm` is not what the source builds. Runs the Android unit tests.
3. **Signs** (human: passphrase). Reads it with `op read "$MOON_KEYSTORE_PASS_OP"` if that is set and 1Password is reachable, otherwise prompts. It reaches Gradle only through environment variables, never a file.
4. **Verifies.** The bundle and APK must be signed with the Play upload certificate (fingerprint pinned in the script) and carry the right version. Writes `dist/x.y.z/` with the `.aab`, `.apk` and `SHA256SUMS`.
5. **Device check** (human). If a phone is attached over adb (USB or wireless), offers to install the signed APK. It then launches the app, sends the phone Home to trigger the icon swap, reports which launcher icon is active, and checks for crashes. Either way, it asks you to confirm before going on. The 0.0.7 crash only showed up in minified release builds, which is why this step exists.

**Stopping is cheap.** `dist/x.y.z/COMMIT` records which commit was signed. Re-running the script on that same commit re-verifies and reuses the signed build instead of asking for the passphrase again. So answer "No" freely, check the phone, and run it again.
6. **Tag** (human). Asks before creating and pushing the annotated tag. CI then publishes the GitHub release, with the changelog section as notes.
7. **Play Console** (human, manual). Upload `dist/x.y.z/moon-oracle-x.y.z.aab`.

## Signing details

| | |
|---|---|
| Keystore | `~/Documents/UrmanacAndroidDeveloper.jks` (override with `MOON_KEYSTORE`) |
| Alias | `key0` (override with `MOON_KEY_ALIAS`) |
| Passphrase | 1Password. Set `MOON_KEYSTORE_PASS_OP=op://<vault>/<item>/<field>` to read it from there |
| Upload certificate SHA-256 | `CE:E2:8B:48:F3:DA:62:95:50:96:B3:CF:4B:F7:17:78:D3:37:A3:00:3B:9C:F4:81:35:4F:51:6F:92:45:D4:30` |

The key and passphrase never go to CI. CI builds and tests everything, including an unsigned minified release, but cannot sign.

A Play-installed copy of the app is re-signed by Google, so `adb install` of the upload-signed APK over it fails. Uninstall it first to test.

## Keeping up between releases

- Dependabot opens grouped update PRs monthly for Gradle/AndroidX, GitHub Actions and the brain's Cargo manifest. CI tests each one; merge the green ones.
- Session logs live in `docs/sessions/`. Read the latest before starting.
