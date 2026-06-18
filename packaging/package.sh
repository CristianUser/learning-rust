#!/bin/bash

# Pronesoft Print Agent Packaging Script

set -e

echo "--- Building Pronesoft Print Agent ---"

# 0. Navigate to project root
cd "$(dirname "$0")/.."

# 1. Install cargo-deb if missing
if ! command -v cargo-deb &> /dev/null; then
    echo "Installing cargo-deb..."
    cargo install cargo-deb
fi

# 2. Build binaries in release mode
echo "Building release binaries..."
cargo build --release
cargo build --release --bin win

# 3. Create Linux package (.deb)
echo "Generating Debian package..."
cargo deb

# Summary
echo ""
echo "--- Build Complete ---"
echo "Linux (.deb): target/debian/printing-service_*.deb"
echo "Windows binaries: target/release/printing-service.exe, target/release/win.exe"
echo ""
echo "To generate the Windows installer (.exe):"
echo "1. Move the project to a Windows machine."
echo "2. Ensure Inno Setup is installed."
echo "3. Run: iscc packaging/windows/inno-file.iss"
