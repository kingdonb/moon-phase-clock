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
# Prompts on the terminal itself; with no terminal, the answer is "no".
ask() {
  local a=""
  { exec 3<>/dev/tty; } 2>/dev/null || return 1
  printf '%s [y/N] ' "$1" >&3
  read -r a <&3 || true
  exec 3>&-
  [[ "$a" == [yY]* ]]
}

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
# Google Play only accepts releases targeting a recent API level (roughly
# within a year of the newest Android). Proxy: the newest stable platform in the
# local SDK, minus one. Install new platforms via Android Studio's SDK Manager.
TARGET_SDK=$(sed -n 's/^ *targetSdk = \([0-9]*\)/\1/p' "$GRADLE_FILE")
NEWEST_PLATFORM=$(ls "${ANDROID_HOME:-$HOME/Library/Android/sdk}/platforms" | sed -n 's/^android-\([0-9]*\)\(\.[0-9]*\)\{0,1\}$/\1/p' | sort -n | tail -1)
if [[ -n "$NEWEST_PLATFORM" ]] && (( TARGET_SDK < NEWEST_PLATFORM - 1 )); then
  die "targetSdk $TARGET_SDK is likely too old for Google Play (newest installed platform is $NEWEST_PLATFORM); raise it to at least $((NEWEST_PLATFORM - 1))"
fi
echo "ok: version, changelog, tag, branch, targetSdk $TARGET_SDK"

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

DIST="dist/$VERSION"
HEAD_SHA=$(git rev-parse HEAD)
BUILD_TOOLS=$(ls -d "${ANDROID_HOME:-$HOME/Library/Android/sdk}"/build-tools/* | sort -V | tail -1)

# Checks a signed APK/AAB pair: upload certificate and version.
verify_signed() {
  local aab=$1 apk=$2 aab_cert apk_cert badging
  aab_cert=$(keytool -printcert -jarfile "$aab" | sed -n 's/^[[:space:]]*SHA256: //p' | head -1)
  [[ "$aab_cert" == "$EXPECTED_CERT_SHA256" ]] || die "bundle signed with an unexpected certificate: $aab_cert"
  "$BUILD_TOOLS/apksigner" verify "$apk" || die "apksigner rejected $apk"
  apk_cert=$("$BUILD_TOOLS/apksigner" verify --print-certs "$apk" | sed -n 's/.*SHA-256 digest: //p' | head -1 | tr 'a-f' 'A-F' | sed 's/../&:/g; s/:$//')
  [[ "$apk_cert" == "$EXPECTED_CERT_SHA256" ]] || die "APK signed with an unexpected certificate: $apk_cert"
  badging=$("$BUILD_TOOLS/aapt2" dump badging "$apk" | head -1)
  grep -q "versionCode='$CODE'" <<<"$badging" && grep -q "versionName='$VERSION'" <<<"$badging" \
    || die "APK version does not match: $badging"
  echo "ok: upload certificate, APK signature, version $VERSION ($CODE)"
}

# A signed build of this exact commit already in dist/ is reused, so stopping
# at the device check (or the tag) never means unlocking the key again.
if [[ -f "$DIST/COMMIT" && "$(cat "$DIST/COMMIT")" == "$HEAD_SHA" ]] \
   && (cd "$DIST" && shasum -a 256 -c --status SHA256SUMS); then
  step "Reusing the signed build of $(git rev-parse --short HEAD) in $DIST"
  verify_signed "$DIST/moon-oracle-$VERSION.aab" "$DIST/moon-oracle-$VERSION.apk"
else

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
verify_signed "$AAB" "$APK"

rm -rf "$DIST"
mkdir -p "$DIST"
cp "$AAB" "$DIST/moon-oracle-$VERSION.aab"
cp "$APK" "$DIST/moon-oracle-$VERSION.apk"
(cd "$DIST" && shasum -a 256 moon-oracle-* > SHA256SUMS)
echo "$HEAD_SHA" > "$DIST/COMMIT"
fi
echo "Signed artifacts in $DIST:"; cat "$DIST/SHA256SUMS"

step "Device check (human)"
ADB="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools/adb"
if "$ADB" get-state >/dev/null 2>&1; then
  if ask "Install the signed release APK on $("$ADB" shell getprop ro.product.model | tr -d '\r') and check it?"; then
    "$ADB" install -r "$DIST/moon-oracle-$VERSION.apk" \
      || die "install failed. A Play-installed copy is signed by Google's app signing key; uninstall it first, then re-run (the signed build is reused)."
    "$ADB" logcat -c
    # Launch the activity itself; monkey does not reliably bring it to the front.
    "$ADB" shell am start -W -n "$PACKAGE/.MainActivity" >/dev/null
    sleep 6
    "$ADB" shell dumpsys activity activities | grep -q "topResumedActivity=.*$PACKAGE/.MainActivity" \
      && echo "ok: app is in the foreground" || echo "WARNING: app is not in the foreground"
    "$ADB" logcat -d 'MoonPhase:*' '*:S' | grep -v '^-' || true
    # Leaving the app triggers the icon swap
    "$ADB" shell input keyevent KEYCODE_HOME
    sleep 4
    echo "Launcher icon now: $("$ADB" shell cmd package query-activities --components -a android.intent.action.MAIN -c android.intent.category.LAUNCHER | grep "$PACKAGE" | tr -d '\r')"
    if "$ADB" shell pidof "$PACKAGE" >/dev/null && [[ -z "$("$ADB" logcat -b crash -d)" ]]; then
      echo "ok: still running, no crashes"
    else
      echo "WARNING: $PACKAGE crashed or stopped; check: $ADB logcat -b crash -d"
    fi
  fi
else
  echo "No device attached. Install $DIST/moon-oracle-$VERSION.apk on a device and confirm it starts before uploading."
fi
ask "Has the release build been checked on a device?" || die "stopping before the tag. Re-run once it has been checked; the signed build in $DIST is reused."

step "Tag (human)"
if ask "Create and push tag $VERSION at $(git rev-parse --short HEAD)? CI will publish the GitHub release."; then
  git tag -a "$VERSION" -m "Moon Oracle $VERSION" "$HEAD_SHA"
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
