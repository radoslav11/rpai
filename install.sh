#!/usr/bin/env bash
# Installation script for rpai

set -e

echo "Installing rpai..."
cargo install --path .

echo ""
echo "✅ Installation complete!"
echo "Binary installed to: ~/.cargo/bin/rpai"
echo ""
echo "To enable tmux integration, add this line to your ~/.tmux.conf:"
echo ""
echo "  bind-key a display-popup -E \"rpai\""
echo ""
echo "Then reload tmux: tmux source ~/.tmux.conf"
echo ""
echo "Use 'prefix + a' to open rpai in a popup"
