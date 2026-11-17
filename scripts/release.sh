#!/usr/bin/env bash
# Builds, verifies and signs a Moon Oracle release, then (only if a human
# says yes) tags it. See RELEASING.md.
#
# Usage: scripts/release.sh            # build + sign + verify, then ask before tagging
#        scripts/release.sh --dry-run  # all checks and tests, no signing, no tag
#
# Environment (all optional):
#   MOON_KEYSTORE           keystore path   (default ~/Documents/UrmanacAndroidDeveloper.jks)
#   MOON_KEY_ALIAS          key alias       (default key0)
#   MOON_KEYSTORE_PASS_OP   1Password reference for the passphrase, e.g. op://Private/Moon Oracle upload key/password
#                           If unset, or if `op` cannot reach the 1Password app, the passphrase is prompted for.
set -euo pipefail

# Upload certificate of the Play Store listing (public; from the 0.0.7 bundle).
EXPECTED_CERT_SHA256="CE:E2:8B:48:F3:DA:62:95:50:96:B3:CF:4B:F7:17:78:D3:37:A3:00:3B:9C:F4:81:35:4F:51:6F:92:45:D4:30"
PACKAGE="com.urmanac.moonphase"

DRY_RUN=0
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=1

cd "$(git rev-parse --show-toplevel)"
step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
die()  { printf '\033[31mrelease: %s\033[0m\n' "$*" >&2; exit 1; }
ask()  { local a; read -r -p "$1 [y/N] " a </dev/tty; [[ "$a" == [yY]* ]]; }

GRADLE_FILE=android/app/build.gradle.kts
VERSION=$(sed -n 's/^ *versionName = "\(.*\)"/\1/p' "$GRADLE_FILE")
CODE=$(sed -n 's/^ *versionCode = \([0-9]*\)/\1/p' "$GRADLE_FILE")
[[ -n "$VERSION" && -n "$CODE" ]] || die "could not read versionName/versionCode from $GRADLE_FILE"

step "Preflight for $VERSION (versionCode $CODE)"
CHANGELOG_TOP=$(sed -n 's/^## \[\([^]]*\)\].*/\1/p' CHANGELOG.md | head -1)
[[ "$CHANGELOG_TOP" == "$VERSION" ]] || die "CHANGELOG.md's first section is [$CHANGELOG_TOP], expected [$VERSION]"
git rev-parse -q --verify "refs/tags/$VERSION" >/dev/null && die "tag $VERSION already exists"
LAST_TAG=$(git tag --sort=-v:refname | head -1)
if [[ -n "$LAST_TAG" ]]; then
  LAST_CODE=$(git show "$LAST_TAG:$GRADLE_FILE" | sed -n 's/^ *versionCode = \([0-9]*\)/\1/p')
  (( CODE > LAST_CODE )) || die "versionCode $CODE must be greater than $LAST_TAG's $LAST_CODE"
fi
if (( ! DRY_RUN )); then
  [[ "$(git branch --show-current)" == "main" ]] || die "release from main (on $(git branch --show-current))"
  git diff --quiet && git diff --cached --quiet || die "working tree has uncommitted changes"
  git fetch -q origin main
  [[ "$(git rev-parse HEAD)" == "$(git rev-parse origin/main)" ]] || die "main is not in sync with origin/main"
fi
echo "ok: version, changelog, tag, branch"

step "Brain: test and rebuild"
(cd brain && cargo test --quiet && cargo build --quiet --target wasm32-wasip1 --release)
cmp -s brain/target/wasm32-wasip1/release/moon_phase_brain.wasm android/app/src/main/assets/moon-phase.wasm \
  || die "the committed android/app/src/main/assets/moon-phase.wasm is not the brain built from source; copy it in and commit"
echo "ok: committed WASM matches a fresh build"

step "Android: unit tests"
./gradlew -q :app:testDebugUnitTest

if (( DRY_RUN )); then
  step "Dry run: building an unsigned minified release"
  ./gradlew -q :app:assembleRelease
  echo "Dry run complete. Nothing signed, nothing tagged."
  exit 0
fi

step "Signing"
MOON_KEYSTORE="${MOON_KEYSTORE:-$HOME/Documents/UrmanacAndroidDeveloper.jks}"
[[ -f "$MOON_KEYSTORE" ]] || die "keystore not found: $MOON_KEYSTORE"
PASS=""
if [[ -n "${MOON_KEYSTORE_PASS_OP:-}" ]] && command -v op >/dev/null; then
  echo "Reading passphrase from 1Password (approve the prompt)..."
  PASS=$(op read "$MOON_KEYSTORE_PASS_OP" 2>/dev/null) || { echo "1Password read failed; falling back to a prompt."; PASS=""; }
fi
if [[ -z "$PASS" ]]; then
  read -r -s -p "Keystore passphrase for $(basename "$MOON_KEYSTORE"): " PASS </dev/tty; echo
fi
export MOON_SIGNING_STORE_FILE="$MOON_KEYSTORE"
export MOON_SIGNING_KEY_ALIAS="${MOON_KEY_ALIAS:-key0}"
export MOON_SIGNING_STORE_PASSWORD="$PASS"
export MOON_SIGNING_KEY_PASSWORD="$PASS"
unset PASS
./gradlew -q --no-configuration-cache :app:bundleRelease :app:assembleRelease
unset MOON_SIGNING_STORE_PASSWORD MOON_SIGNING_KEY_PASSWORD

AAB=android/app/build/outputs/bundle/release/app-release.aab
APK=android/app/build/outputs/apk/release/app-release.apk
[[ -f "$AAB" && -f "$APK" ]] || die "signed outputs missing (is the passphrase right?)"

step "Verify signatures"
BUILD_TOOLS=$(ls -d "${ANDROID_HOME:-$HOME/Library/Android/sdk}"/build-tools/* | sort -V | tail -1)
AAB_CERT=$(keytool -printcert -jarfile "$AAB" | sed -n 's/^[[:space:]]*SHA256: //p' | head -1)
[[ "$AAB_CERT" == "$EXPECTED_CERT_SHA256" ]] || die "bundle signed with an unexpected certificate: $AAB_CERT"
"$BUILD_TOOLS/apksigner" verify "$APK" || die "apksigner rejected $APK"
APK_CERT=$("$BUILD_TOOLS/apksigner" verify --print-certs "$APK" | sed -n 's/.*SHA-256 digest: //p' | head -1 | tr 'a-f' 'A-F' | sed 's/../&:/g; s/:$//')
[[ "$APK_CERT" == "$EXPECTED_CERT_SHA256" ]] || die "APK signed with an unexpected certificate: $APK_CERT"
BADGING=$("$BUILD_TOOLS/aapt2" dump badging "$APK" | head -1)
grep -q "versionCode='$CODE'" <<<"$BADGING" && grep -q "versionName='$VERSION'" <<<"$BADGING" \
  || die "APK version does not match: $BADGING"
echo "ok: upload certificate, APK signature, version $VERSION ($CODE)"

DIST="dist/$VERSION"
mkdir -p "$DIST"
cp "$AAB" "$DIST/moon-oracle-$VERSION.aab"
cp "$APK" "$DIST/moon-oracle-$VERSION.apk"
(cd "$DIST" && shasum -a 256 moon-oracle-* > SHA256SUMS)
echo "Signed artifacts in $DIST:"; cat "$DIST/SHA256SUMS"

step "Device check (human)"
ADB="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
if "$ADB" get-state >/dev/null 2>&1; then
  if ask "Install the signed release APK on $("$ADB" shell getprop ro.product.model | tr -d '\r') and launch it?"; then
    "$ADB" install -r "$DIST/moon-oracle-$VERSION.apk" \
      || echo "Install failed. A Play-installed copy is signed by Google's app signing key; uninstall it first to test this build."
    "$ADB" shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1 || true
    sleep 5
    if "$ADB" shell pidof "$PACKAGE" >/dev/null; then echo "ok: $PACKAGE is running"; else echo "WARNING: $PACKAGE is not running; check: $ADB logcat -b crash -d"; fi
  fi
else
  echo "No device attached. Install $DIST/moon-oracle-$VERSION.apk on a device and confirm it starts before uploading."
fi
ask "Has the release build been checked on a device?" || die "stopping before the tag. Re-run once it has been checked."

step "Tag (human)"
if ask "Create and push tag $VERSION at $(git rev-parse --short HEAD)? CI will publish the GitHub release."; then
  git tag -a "$VERSION" -m "Moon Oracle $VERSION"
  git push origin "$VERSION"
  echo "Tagged. Watch CI: gh run watch"
else
  echo "Not tagged. Later: git tag -a $VERSION -m 'Moon Oracle $VERSION' && git push origin $VERSION"
fi

cat <<NEXT

Remaining manual step: upload $DIST/moon-oracle-$VERSION.aab to Play Console
(Moon Oracle > Test and release > Production > Create new release),
paste the $VERSION section of CHANGELOG.md as release notes, and roll out.
NEXT
