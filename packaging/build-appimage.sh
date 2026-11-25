#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|aarch64) ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

BUILD_DIR="$ROOT/target/appimage"
APPDIR="$BUILD_DIR/AppDir"
TOOLS_DIR="$BUILD_DIR/tools"
DIST_DIR="$ROOT/dist"
OUTPUT="$DIST_DIR/AimRust-${VERSION}-${ARCH}.AppImage"
OUTPUT_TMP="$OUTPUT.tmp"

BIN="$ROOT/target/release/AimRust"

echo "==> Building release binary"
cargo build --release --bin AimRust
if [ ! -x "$BIN" ]; then
  echo "Missing release binary: $BIN" >&2
  exit 1
fi

echo "==> Preparing AppDir"
rm -rf "$APPDIR"
mkdir -p \
  "$APPDIR/usr/bin" \
  "$APPDIR/usr/share/icons/hicolor/256x256/apps" \
  "$APPDIR/usr/share/icons/hicolor/512x512/apps" \
  "$APPDIR/usr/share/metainfo" \
  "$APPDIR/usr/share/applications"

install -m 755 "$BIN" "$APPDIR/usr/bin/AimRust"

# Assets bundling (Bevy expects assets/ directory alongside the executable or working dir)
if [ -d "$ROOT/assets" ]; then
  cp -r "$ROOT/assets" "$APPDIR/usr/bin/assets"
fi

chmod -R a+rX "$APPDIR"

ICON_SRC="$ROOT/packaging/aim-rust.png"
if [ -f "$ICON_SRC" ]; then
  install -m 644 "$ICON_SRC" "$APPDIR/aim-rust.png"
  for sz in 256 512; do
    DEST="$APPDIR/usr/share/icons/hicolor/${sz}x${sz}/apps/aim-rust.png"
    if command -v magick >/dev/null 2>&1; then
      magick "$ICON_SRC" -resize "${sz}x${sz}" "$DEST"
    elif command -v convert >/dev/null 2>&1; then
      convert "$ICON_SRC" -resize "${sz}x${sz}" "$DEST"
    else
      install -m 644 "$ICON_SRC" "$DEST"
    fi
  done
  ln -sf aim-rust.png "$APPDIR/.DirIcon"
fi

sed "s/^X-AppImage-Version=.*/X-AppImage-Version=${VERSION}/" \
  "$ROOT/packaging/io.github.michaelx17.AimRust.desktop" > "$APPDIR/io.github.michaelx17.AimRust.desktop"
chmod 644 "$APPDIR/io.github.michaelx17.AimRust.desktop"
install -m 644 "$APPDIR/io.github.michaelx17.AimRust.desktop" "$APPDIR/usr/share/applications/io.github.michaelx17.AimRust.desktop"
install -m 644 "$ROOT/packaging/io.github.michaelx17.AimRust.metainfo.xml" \
  "$APPDIR/usr/share/metainfo/io.github.michaelx17.AimRust.metainfo.xml"
if [ -f "$ROOT/packaging/.appimage-hub.json" ]; then
  install -m 644 "$ROOT/packaging/.appimage-hub.json" "$APPDIR/.appimage-hub.json"
fi

cat > "$APPDIR/AppRun" <<'APPRUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
cd "$HERE/usr/bin"
exec "$HERE/usr/bin/AimRust" "$@"
APPRUN
chmod 755 "$APPDIR/AppRun"

mkdir -p "$DIST_DIR" "$TOOLS_DIR"
APPIMAGETOOL="${APPIMAGETOOL:-}"
if [ -z "$APPIMAGETOOL" ] && command -v appimagetool >/dev/null 2>&1; then
  APPIMAGETOOL="$(command -v appimagetool)"
fi
if [ -z "$APPIMAGETOOL" ]; then
  APPIMAGETOOL="$TOOLS_DIR/appimagetool-${ARCH}.AppImage"
  if [ ! -x "$APPIMAGETOOL" ]; then
    echo "==> Downloading appimagetool"
    curl -fL --retry 3 \
      "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage" \
      -o "$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
  fi
fi

echo "==> Creating $OUTPUT"
rm -f "$OUTPUT_TMP"
ARCH="$ARCH" "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" "$OUTPUT_TMP"
chmod +x "$OUTPUT_TMP"
mv -f "$OUTPUT_TMP" "$OUTPUT"

echo "==> Built: $OUTPUT"
sha256sum "$OUTPUT"
