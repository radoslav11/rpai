#!/usr/bin/env bash
# Build script for rpai (for local testing)

set -e

echo "Building rpai for local testing..."
cargo build --release

echo ""
echo "✅ Build complete! Binary is at: target/release/rpai"
echo ""
echo "For production installation, use: cargo install --path ."
echo "This will install to ~/.cargo/bin/ automatically."
echo ""
echo "To enable tmux integration, add this line to your ~/.tmux.conf:"
echo ""
echo "  bind-key a display-popup -E \"rpai\""
echo ""
echo "Then reload tmux: tmux source ~/.tmux.conf"
echo ""
echo "Use 'prefix + a' to open rpai in a popup"
