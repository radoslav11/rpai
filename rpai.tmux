#!/usr/bin/env bash
# rpai - tmux plugin for AI agent session management
#
# Installation:
#   1. Install rpai binary (cargo install --path . or copy to ~/bin)
#   2. Add to ~/.tmux.conf:
#      run-shell /path/to/rpai.tmux
#   3. Reload tmux: tmux source ~/.tmux.conf

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Find rpai binary
get_rpai_path() {
    # Check common locations
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
    tmux display-message "rpai not found! Please install: cargo install --path /path/to/rpai"
    exit 1
fi

# Get options from tmux config with defaults
get_tmux_option() {
    local option="$1"
    local default_value="$2"
    local option_value=$(tmux show-option -gqv "$option")
    if [[ -z "$option_value" ]]; then
        echo "$default_value"
    else
        echo "$option_value"
    fi
}

# Configuration options (set these in ~/.tmux.conf)
RPAI_KEY=$(get_tmux_option "@rpai-key" "a")
RPAI_WIDTH=$(get_tmux_option "@rpai-width" "80%")
RPAI_HEIGHT=$(get_tmux_option "@rpai-height" "60%")

# Bind key to open rpai in a popup
tmux bind-key "$RPAI_KEY" display-popup -E -w "$RPAI_WIDTH" -h "$RPAI_HEIGHT" "$RPAI_PATH"

tmux display-message "rpai loaded: prefix+$RPAI_KEY to open"
