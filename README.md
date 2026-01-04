# rpai

<div align="center">
  <img src="assets/redpanda.png" width="120" alt="rpai logo">
</div>

A tool for managing multiple AI coding agent sessions (opencode, claude, codex, cursor) in tmux.

## Features

- Scan for running AI agent processes and ability to jump around.
- **Running/Waiting indicator** - shows if agent is active or stale (▶/⏸).
- Multiple color themes (gruvbox, nord, catppuccin, dracula, tokyo, solarized).
- Recommended workflow is to map to a tmux popup window.

## Installation

### Option 1: Install from crates.io (Recommended)

```bash
cargo install rpai
```

### Option 2: Install from source (Latest dev version)

```bash
git clone https://github.com/radoslav11/rpai.git
cd rpai
./install.sh
```

This builds and installs rpai in one step.

## Tmux Setup

Add this line to your `~/.tmux.conf` (or alternative mapping).

```bash
bind-key a display-popup -E "rpai"
```

Then reload tmux:

```bash
tmux source ~/.tmux.conf
```

**Now use it:**

- Press `prefix + a` to open rpai in a centered popup.

## Usage

```bash
rpai                # Interactive TUI (default)
rpai scan           # List sessions (non-interactive)
rpai jump <id|name> # Jump to session by ID or name
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

## Config Directory

`~/.config/rpai/`
- `theme` - Current theme name

## License

MIT
