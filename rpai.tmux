#!/usr/bin/env bash
# rpai - tmux integration script
#
# Installation: Add this line to ~/.tmux.conf:
#   run-shell /path/to/rpai/rpai.tmux
#
# Then reload tmux: tmux source ~/.tmux.conf

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Find rpai binary
get_rpai_path() {
    if command -v rpai &> /dev/null; then
        echo "rpai"
    elif [[ -x "$CURRENT_DIR/target/release/rpai" ]]; then
        echo "$CURRENT_DIR/target/release/rpai"
    elif [[ -x "$HOME/.cargo/bin/rpai" ]]; then
        echo "$HOME/.cargo/bin/rpai"
    elif [[ -x "$HOME/bin/rpai" ]]; then
        echo "$HOME/bin/rpai"
    elif [[ -x "/usr/local/bin/rpai" ]]; then
        echo "/usr/local/bin/rpai"
    else
        echo ""
    fi
}

RPAI_PATH=$(get_rpai_path)

if [[ -z "$RPAI_PATH" ]]; then
    tmux display-message "rpai not found! Install with: cargo install rpai"
    exit 1
fi

# Bind prefix + a to open rpai popup
tmux bind-key "a" display-popup -E -w "80%" -h "60%" "$RPAI_PATH"

tmux display-message "rpai loaded: prefix + a to open"
