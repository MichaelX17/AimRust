#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-xwin &>/dev/null && ! cargo xwin --version &>/dev/null; then
    echo "Error: cargo-xwin is not installed."
    echo "Please install it by running: cargo install cargo-xwin"
    exit 1
fi

echo "Building AimRust for Windows (x86_64-pc-windows-msvc) in release mode..."
cargo xwin build --target x86_64-pc-windows-msvc --release
