#!/usr/bin/env bash
# maid-cafe-se macOS版(.app と .dmg)を作る。Apple SiliconとIntel両対応のuniversal。 / Build the macOS .app and .dmg (universal).
#
# 使い方 / Usage:  installer/build-macos.sh
# 必要なもの / Requires: macOS、Rust (cargo、rustup)、python3、Xcodeのコマンドラインツール(hdiutil/iconutil/sips/lipo/codesign)
# 出力 / Output: installer/dist/maid-cafe-se-<version>-macos.dmg と SHA256SUMS-macos.txt
# 注意: Secretsが無い時はad-hoc署名のみ(初回は右クリック→「開く」)。Developer ID署名+公証は上記の分岐(未検証)。
# 未検証: 開発機がWindowsのため、実機のMacでの動作は未確認(CIでビルドのみ)。
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$root/crates/maid-cafe-desktop/Cargo.toml" | head -1)"
cd "$root"
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  rustup target add "$t"
  cargo build --release -p maid-cafe-desktop --target "$t"
done

dist="$root/installer/dist"; work="$root/installer/build/macos"
rm -rf "$work"; mkdir -p "$dist" "$work"
app="$work/maid-cafe-se.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
lipo -create -output "$app/Contents/MacOS/maid-cafe-se" \
  target/aarch64-apple-darwin/release/maid-cafe-se target/x86_64-apple-darwin/release/maid-cafe-se

# アイコン(失敗してもアイコンなしで続ける)
if python3 installer/make-png.py "$work/icon.png" 1024 && mkdir "$work/icon.iconset"; then
  for s in 16 32 64 128 256 512; do
    sips -z $s $s "$work/icon.png" --out "$work/icon.iconset/icon_${s}x${s}.png" >/dev/null || true
    sips -z $((s*2)) $((s*2)) "$work/icon.png" --out "$work/icon.iconset/icon_${s}x${s}@2x.png" >/dev/null || true
  done
  iconutil -c icns "$work/icon.iconset" -o "$app/Contents/Resources/maid-cafe-se.icns" || echo "icns作成に失敗(アイコンなしで続行)"
fi

cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>maid-cafe-se</string>
  <key>CFBundleDisplayName</key><string>maid-cafe-se</string>
  <key>CFBundleIdentifier</key><string>jp.co.aon.maid-cafe-se</string>
  <key>CFBundleExecutable</key><string>maid-cafe-se</string>
  <key>CFBundleIconFile</key><string>maid-cafe-se</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
# 署名: 環境変数 MACOS_CERT_P12_BASE64 等が全て揃っている時だけ Developer ID 署名+公証。無ければ今まで通りad-hoc署名。
# 必要な値(GitHub Secrets): MACOS_CERT_P12_BASE64, MACOS_CERT_PASSWORD, MACOS_SIGN_IDENTITY,
#   APPLE_ID, APPLE_TEAM_ID, APPLE_APP_PASSWORD(詳細はPORTING.md)。値は出力しない。 未検証。
sign_real=0
if [ -n "${MACOS_CERT_P12_BASE64:-}" ] && [ -n "${MACOS_CERT_PASSWORD:-}" ] && [ -n "${MACOS_SIGN_IDENTITY:-}" ] \
   && [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ] && [ -n "${APPLE_APP_PASSWORD:-}" ]; then
  sign_real=1
  kc="$work/signing.keychain-db"; kcpw="$(uuidgen)"
  echo "$MACOS_CERT_P12_BASE64" | base64 --decode > "$work/cert.p12"
  security create-keychain -p "$kcpw" "$kc"
  security set-keychain-settings -lut 3600 "$kc"
  security unlock-keychain -p "$kcpw" "$kc"
  security import "$work/cert.p12" -k "$kc" -P "$MACOS_CERT_PASSWORD" -T /usr/bin/codesign
  security set-key-partition-list -S apple-tool:,apple: -s -k "$kcpw" "$kc" >/dev/null
  security list-keychains -d user -s "$kc" login.keychain-db
  rm -f "$work/cert.p12"
  codesign --force --deep --options runtime --timestamp -s "$MACOS_SIGN_IDENTITY" "$app"
  codesign --verify --deep --strict "$app"
else
  echo "署名用の値が揃っていないため、ad-hoc署名(未署名扱い)で続行 / signing secrets not set: ad-hoc signing"
  codesign --force --deep -s - "$app"
fi

stage="$work/dmg"; mkdir -p "$stage"
cp -R "$app" "$stage/"; ln -s /Applications "$stage/Applications"
dmg="$dist/maid-cafe-se-$version-macos.dmg"
rm -f "$dmg"
hdiutil create -volname "maid-cafe-se $version" -srcfolder "$stage" -ov -format UDZO "$dmg"
if [ "$sign_real" = 1 ]; then
  codesign --force --timestamp -s "$MACOS_SIGN_IDENTITY" "$dmg"
  xcrun notarytool submit "$dmg" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD" --wait
  xcrun stapler staple "$dmg"
  xcrun stapler validate "$dmg"
fi
(cd "$dist" && shasum -a 256 "$(basename "$dmg")" > SHA256SUMS-macos.txt)
echo "done:"; ls -l "$dmg"
