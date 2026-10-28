#!/usr/bin/env bash
# maid-cafe-se Linux版のパッケージ(.deb と .tar.gz)を作る。 / Build the Linux packages (.deb and .tar.gz).
#
# 使い方 / Usage:  installer/build-linux.sh [--skip-build]
# 必要なもの / Requires: Rust (cargo)、dpkg-deb、python3、ビルド用に libgtk-3-dev libxdo-dev libayatana-appindicator3-dev
# 出力 / Output: installer/dist/maid-cafe-se_<version>_<arch>.deb と maid-cafe-se-<version>-linux-<arch>.tar.gz と SHA256SUMS-linux.txt
# 未検証: 開発機がWindowsのため、実機のLinuxでの動作は未確認(CIでビルド・テストのみ)。
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$root/crates/maid-cafe-desktop/Cargo.toml" | head -1)"
case "$(uname -m)" in x86_64) debarch=amd64 ;; aarch64) debarch=arm64 ;; *) debarch="$(uname -m)" ;; esac
[ "${1:-}" = "--skip-build" ] || (cd "$root" && cargo build --release -p maid-cafe-desktop)
bin="$root/target/release/maid-cafe-se"
[ -x "$bin" ] || { echo "ビルド済みの実行ファイルがありません: $bin" >&2; exit 1; }

dist="$root/installer/dist"; work="$root/installer/build/linux"
rm -rf "$work"; mkdir -p "$dist" "$work"
python3 "$root/installer/make-png.py" "$work/maid-cafe-se.png" 512

desktop='[Desktop Entry]
Type=Application
Name=maid-cafe-se
Comment=Maid cafe alarm / めいどかふぇ目覚まし
Exec=maid-cafe-se
Icon=maid-cafe-se
Terminal=false
Categories=Utility;Clock;
'

# ---- .deb ----
pkg="$work/deb"
install -Dm755 "$bin" "$pkg/usr/bin/maid-cafe-se"
install -Dm644 "$work/maid-cafe-se.png" "$pkg/usr/share/icons/hicolor/512x512/apps/maid-cafe-se.png"
mkdir -p "$pkg/usr/share/applications"; printf '%s' "$desktop" > "$pkg/usr/share/applications/maid-cafe-se.desktop"
install -Dm644 "$root/README.md" "$pkg/usr/share/doc/maid-cafe-se/README.md"
mkdir -p "$pkg/DEBIAN"
size="$(du -sk "$pkg/usr" | cut -f1)"
cat > "$pkg/DEBIAN/control" <<EOF
Package: maid-cafe-se
Version: $version
Section: utils
Priority: optional
Architecture: $debarch
Depends: libgtk-3-0, libxdo3, libayatana-appindicator3-1 | libappindicator3-1, espeak-ng, alsa-utils | pulseaudio-utils | pipewire-bin
Installed-Size: $size
Maintainer: aon-co-jp <noreply@users.noreply.github.com>
Homepage: https://github.com/aon-co-jp/maid-cafe-se
Description: Maid cafe alarm clock with voice (めいどかふぇ目覚まし)
 Alarm clock that speaks in a maid's voice. Local web UI with a tray icon.
 Uses espeak-ng to synthesize speech.
EOF
deb="$dist/maid-cafe-se_${version}_${debarch}.deb"
dpkg-deb --root-owner-group --build "$pkg" "$deb"

# ---- .tar.gz(展開してinstall.shでも、そのまま実行でも) ----
tdir="$work/maid-cafe-se-$version"
mkdir -p "$tdir"
cp "$bin" "$tdir/maid-cafe-se"; cp "$work/maid-cafe-se.png" "$tdir/"; printf '%s' "$desktop" > "$tdir/maid-cafe-se.desktop"
cp "$root/README.md" "$tdir/"
cat > "$tdir/install.sh" <<'EOF'
#!/bin/sh
# 利用者のホーム(~/.local)へ入れる。 / Installs into ~/.local for the current user.
set -e
d="$(cd "$(dirname "$0")" && pwd)"
install -Dm755 "$d/maid-cafe-se" "$HOME/.local/bin/maid-cafe-se"
install -Dm644 "$d/maid-cafe-se.png" "$HOME/.local/share/icons/hicolor/512x512/apps/maid-cafe-se.png"
install -Dm644 "$d/maid-cafe-se.desktop" "$HOME/.local/share/applications/maid-cafe-se.desktop"
echo "installed. 読み上げには espeak-ng が必要です (e.g. sudo apt install espeak-ng)"
EOF
chmod +x "$tdir/install.sh"
tgz="$dist/maid-cafe-se-$version-linux-$(uname -m).tar.gz"
tar -C "$work" -czf "$tgz" "maid-cafe-se-$version"

(cd "$dist" && sha256sum "$(basename "$deb")" "$(basename "$tgz")" > SHA256SUMS-linux.txt)
echo "done:"; ls -l "$deb" "$tgz"
