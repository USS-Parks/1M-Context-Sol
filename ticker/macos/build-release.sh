#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: build-release.sh OUTPUT_DIRECTORY" >&2
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
output_dir="$1"
mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"

artifact_name="1M-Context-Ticker-macOS-universal.dmg"
checksum_name="$artifact_name.sha256"
manifest_name="1M-Context-Ticker-macOS-universal.manifest.json"
for name in "$artifact_name" "$checksum_name" "$manifest_name"; do
  if [[ -e "$output_dir/$name" ]]; then
    echo "refusing to overwrite existing output: $output_dir/$name" >&2
    exit 1
  fi
done

temp_base="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
work="$(mktemp -d "$temp_base/1m-context-ticker-macos.XXXXXX")"
mount_point="$work/mount"
mounted=0
cleanup() {
  if [[ $mounted -eq 1 ]]; then
    hdiutil detach "$mount_point" -quiet || true
  fi
  rm -rf "$work"
}
trap cleanup EXIT

sdk="$(xcrun --sdk macosx --show-sdk-path)"
build_architecture() {
  local triple="$1"
  swift build \
    --package-path "$script_dir" \
    --configuration release \
    --triple "$triple" \
    --sdk "$sdk" \
    -Xswiftc -warnings-as-errors
  swift build \
    --package-path "$script_dir" \
    --configuration release \
    --triple "$triple" \
    --sdk "$sdk" \
    --show-bin-path
}

arm_bin_dir="$(build_architecture arm64-apple-macosx13.0 | tail -n 1)"
intel_bin_dir="$(build_architecture x86_64-apple-macosx13.0 | tail -n 1)"
arm_binary="$arm_bin_dir/OneMContextTicker"
intel_binary="$intel_bin_dir/OneMContextTicker"
test -x "$arm_binary"
test -x "$intel_binary"

app="$work/1M Context Ticker.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
lipo -create "$arm_binary" "$intel_binary" -output "$app/Contents/MacOS/OneMContextTicker"
chmod 755 "$app/Contents/MacOS/OneMContextTicker"
lipo -verify_arch arm64 x86_64 "$app/Contents/MacOS/OneMContextTicker"
cp "$script_dir/Info.plist" "$app/Contents/Info.plist"
cp "$repo_root/overlay/sol-1m-models.json" "$app/Contents/Resources/sol-1m-models.json"
plutil -lint "$app/Contents/Info.plist"

stage="$work/stage"
mkdir -p "$stage"
ditto --norsrc "$app" "$stage/1M Context Ticker.app"
ln -s /Applications "$stage/Applications"

temporary_dmg="$work/$artifact_name"
COPYFILE_DISABLE=1 hdiutil create \
  -quiet \
  -format UDZO \
  -fs HFS+ \
  -volname "1M Context Ticker" \
  -srcfolder "$stage" \
  "$temporary_dmg"

mkdir -p "$mount_point"
hdiutil attach -quiet -readonly -nobrowse -mountpoint "$mount_point" "$temporary_dmg"
mounted=1
test -d "$mount_point/1M Context Ticker.app"
test -L "$mount_point/Applications"
test "$(readlink "$mount_point/Applications")" = "/Applications"

logical_count=0
for item in "$mount_point"/*; do
  name="$(basename "$item")"
  case "$name" in
    "1M Context Ticker.app"|"Applications") logical_count=$((logical_count + 1)) ;;
    *) echo "unexpected DMG content: $name" >&2; exit 1 ;;
  esac
done
test "$logical_count" -eq 2
lipo -verify_arch arm64 x86_64 "$mount_point/1M Context Ticker.app/Contents/MacOS/OneMContextTicker"
cmp "$script_dir/Info.plist" "$mount_point/1M Context Ticker.app/Contents/Info.plist"
cmp "$repo_root/overlay/sol-1m-models.json" \
  "$mount_point/1M Context Ticker.app/Contents/Resources/sol-1m-models.json"
hdiutil detach "$mount_point" -quiet
mounted=0

cp "$temporary_dmg" "$output_dir/$artifact_name"
sha256="$(shasum -a 256 "$output_dir/$artifact_name" | awk '{print $1}')"
printf '%s  %s\n' "$sha256" "$artifact_name" > "$output_dir/$checksum_name"
source_commit="$(git -C "$repo_root" rev-parse HEAD)"
python3 "$script_dir/create-manifest.py" \
  --artifact "$output_dir/$artifact_name" \
  --sha256 "$sha256" \
  --source-commit "$source_commit" \
  --output "$output_dir/$manifest_name"

echo "DMG=$output_dir/$artifact_name"
echo "SHA256=$sha256"
echo "CHECKSUM=$output_dir/$checksum_name"
echo "MANIFEST=$output_dir/$manifest_name"
