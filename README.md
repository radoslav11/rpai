# rpai

A TUI for managing multiple AI coding agent sessions (opencode, claude, codex, cursor) in tmux.

## Features

- Interactive TUI with mouse support
- Scan for running AI agent processes
- **Auto-refreshes every second** - stays up-to-date with session state
- **Running/Waiting indicator** - shows if agent is active or waiting for input (▶/⏸)
- Jump to any AI session with Enter
- Multiple color themes (gruvbox, nord, catppuccin, dracula, tokyo, solarized)
- Opens as a tmux popup window

## Installation

### Option 1: Quick Install Script (Recommended)

```bash
# Clone the repo and run install script
git clone <repo-url>
cd rpai
./install.sh
```

This script:
- Builds the binary with `cargo build --release`
- Installs `rpai` and `rpai.tmux` to `~/.local/bin/`
- Shows you how to add it to tmux

### Option 2: Install with Cargo

```bash
# Install from this directory
cargo install --path .

# After install, copy the tmux plugin to ~/.cargo/bin/
cp rpai.tmux ~/.cargo/bin/

# Or if published to crates.io
cargo install rpai
# Then download rpai.tmux from the repository
```

Make sure `~/.cargo/bin` is in your PATH:

```bash
# Add to your shell config (~/.zshrc or ~/.bashrc)
export PATH="$HOME/.cargo/bin:$PATH"
```

### Option 3: Manual Build

```bash
git clone <repo-url>
cd rpai
cargo build --release

# Copy binary and tmux plugin
cp target/release/rpai ~/.local/bin/
cp rpai.tmux ~/.local/bin/
```

### Verify Installation

```bash
# Check binary is in PATH
which rpai

# Test it works
rpai --help
```

## Tmux Setup

Add this to your `~/.tmux.conf`:

```bash
# Source the rpai tmux plugin
# If you installed with cargo:
run-shell ~/.cargo/bin/rpai.tmux

# If you installed manually:
# run-shell ~/.local/bin/rpai.tmux
# run-shell /path/to/rpai/rpai.tmux
```

**Optional customization:**

```bash
set -g @rpai-key "a"           # prefix + a to open (default: a)
set -g @rpai-width "80%"       # popup width (default: 80%)
set -g @rpai-height "60%"      # popup height (default: 60%)
```

**Reload tmux:**

```bash
tmux source ~/.tmux.conf
# or
prefix + :source-file ~/.tmux.conf
```

**Now use it:**

- Press `prefix + a` to open rpai in a centered popup

## Usage

```bash
rpai                # Interactive TUI (default)
rpai scan           # List sessions (non-interactive)
rpai kill <id>      # Terminate a session
rpai theme [name]   # Show/set theme
rpai help           # Show help
```

## Keyboard Shortcuts (TUI)

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate sessions |
| `Enter` | Jump to selected session |
| `t` | Cycle through themes |
| `/` or `:` | Enter command mode |
| `q` / `Esc` / `Ctrl-C` | Quit |
| Mouse click | Select session |
| Mouse scroll | Navigate sessions |

## Commands (type after `/`)

- `theme [name]` - Switch theme (gruvbox, nord, catppuccin, dracula, tokyo, solarized)
- `themes` - List available themes

## Themes

- **gruvbox** (default) - Warm retro colors
- **nord** - Cool arctic blues  
- **catppuccin** - Pastel mocha vibes
- **dracula** - Purple vampire aesthetic
- **tokyo** - Tokyo Night purple/blue
- **solarized** - Classic solarized dark

Theme is persisted to `~/.config/rpai/theme`

## How It Works

1. Scans all processes for AI agent patterns (opencode, claude, codex, cursor)
2. Walks the process tree to find the tmux pane containing each agent
3. Displays sessions with agent type, PID, state (▶/⏸), uptime, memory usage, tmux location, and working directory
4. When you select a session, jumps directly to that tmux pane
5. Auto-refreshes every second to show current CPU usage and session state

## Config Directory

`~/.config/rpai/`
- `theme` - Current theme name
