#!/usr/bin/env bash
# Installation script for rpai

set -e

echo "Installing rpai..."

# Build the binary
echo "Building rpai..."
cargo build --release

# Install binary
echo "Installing binary to ~/.local/bin/..."
mkdir -p ~/.local/bin
cp target/release/rpai ~/.local/bin/

# Install tmux plugin
echo "Installing tmux plugin to ~/.local/bin/..."
cp rpai.tmux ~/.local/bin/
chmod +x ~/.local/bin/rpai.tmux

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "⚠️  WARNING: ~/.local/bin is not in your PATH"
    echo "Add this to your shell config (~/.zshrc or ~/.bashrc):"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "Add this to your ~/.tmux.conf:"
echo ""
echo "  run-shell ~/.local/bin/rpai.tmux"
echo ""
echo "Then reload tmux: tmux source ~/.tmux.conf"
echo ""
echo "Use 'prefix + a' to open rpai in a popup"
