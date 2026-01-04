#!/usr/bin/env bash
# Installation script for rpai

set -e

echo "Building rpai..."
cargo build --release

echo ""
echo "✅ Build complete! Binary is at: target/release/rpai"
echo ""
echo "To install globally with cargo (recommended):"
echo "  cargo install --path ."
echo ""
echo "To enable tmux integration, add this to your ~/.tmux.conf:"
echo ""
echo "  run-shell $(pwd)/rpai.tmux"
echo ""
echo "Then reload tmux: tmux source ~/.tmux.conf"
echo ""
echo "Use 'prefix + a' to open rpai in a popup"
