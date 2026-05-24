#!/usr/bin/env bash
set -euo pipefail

BINARY="MeowSimulatorRust"
RELEASE="target/release"
DIST="dist/windows"

echo "==> Building release..."
cargo build --release

echo "==> Preparing dist directory..."
rm -rf "$DIST"
mkdir -p "$DIST"

echo "==> Copying binary..."
cp "$RELEASE/$BINARY.exe" "$DIST/"

echo "==> Copying DLLs..."
ldd "$RELEASE/$BINARY.exe" \
  | grep -i ucrt64 \
  | awk '{print $3}' \
  | xargs -I{} cp {} "$DIST/"
cp /ucrt64/bin/vulkan-1.dll "$DIST/"

echo "==> Copying GTK runtime data..."
mkdir -p "$DIST/share/glib-2.0"
cp -r /ucrt64/share/glib-2.0/schemas "$DIST/share/glib-2.0/"
cp -r /ucrt64/share/icons "$DIST/share/"
glib-compile-schemas "$DIST/share/glib-2.0/schemas/"

echo "==> Copying assets..."
cp -r "$RELEASE/assets" "$DIST/"

echo "==> Creating zip..."
(cd dist && zip -r "meow-simulator-windows.zip" windows/)

echo "==> Done: dist/meow-simulator-windows.zip"
