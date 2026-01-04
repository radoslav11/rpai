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

### Option 1: Install from crates.io (Recommended)

```bash
cargo install rpai
```

### Option 2: Install from source (Latest dev version)

```bash
git clone https://github.com/radoslav11/rpai.git
cd rpai
cargo install --path .
```

This installs `rpai` to `~/.cargo/bin/rpai`.

### Option 3: Quick build script

```bash
git clone https://github.com/radoslav11/rpai.git
cd rpai
./install.sh
```

This builds the binary to `target/release/rpai` for local testing.

## Tmux Setup

Add this line to your `~/.tmux.conf`:

```bash
bind-key a display-popup -E "rpai"
```

Then reload tmux:

```bash
tmux source ~/.tmux.conf
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

## License

MIT
