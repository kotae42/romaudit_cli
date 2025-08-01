#!/bin/bash

# Build script for romaudit_cli

set -e

echo "Building romaudit_cli..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build in release mode
cargo build --release

echo "Build complete!"
echo ""
echo "Binary location: target/release/romaudit_cli"
echo ""
echo "To install system-wide (requires sudo):"
echo "  sudo cp target/release/romaudit_cli /usr/local/bin/"
echo ""
echo "Or add to your PATH:"
echo "  export PATH=\"\$PATH:$(pwd)/target/release\""