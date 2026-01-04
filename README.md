# rpai

A tmux plugin for managing multiple AI coding agent sessions (opencode, claude, codex, aider, cursor).

## Features

- Scan for running AI agent processes
- Walks process tree to find tmux sessions by matching TTY
- Display session details: PID, agent type, working directory, status, CPU/memory usage
- Status detection: Active, Idle, Stale
- Jump to tmux sessions: `rpai jump <session_name>`
- Kill sessions by ID
- Consistent session ordering (sorted by agent type, then PID)

## Installation

```bash
# Clone and build
git clone <repo-url>
cd redpandai
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## Usage

```bash
rpai scan          # Scan and display AI agent sessions
rpai jump <name>   # Jump to tmux session by name
rpai kill <id>     # Terminate a session by ID
rpai help           # Show help message
```

## Output Example

```
AI Agent Sessions:

○ [1] claude | Idle | 2m
    PID: 70351 | Mem: 256MB | CPU: 0.0%
    /Users/rado

○ [2] opencode | Idle | 1m
    PID: 71225 | Mem: 256MB | CPU: 0.0%
    /Users/rado/Programming/Projects/redpandai

○ [3] opencode | Idle | 1m
    PID: 73401 | Mem: 1472MB | CPU: 0.0%
    /Users/rado/Programming/Projects/redpandai
```

## Status Definitions

- **Active**: High CPU usage or very recent start (less than 1 minute)
- **Idle**: Running but with low activity (1-30 minutes)
- **Stale**: Running but inactive for 30+ minutes

## Tmux Integration

The tool walks the process tree to find tmux sessions by matching TTY. When an AI agent is running in a tmux pane, it will display:
- Session name (e.g., "main", "work")
- Window index (e.g., 1, 2)
- Pane ID (e.g., %28, %57)

This works even when tmux doesn't directly track the process - by finding the parent process that shares the same TTY.

## Future Enhancements

- Interactive TUI mode
- Token usage tracking (tokens used, context window, cost estimation)
- Session activity monitoring (last message, current task)
- Session renaming via tmux
- Session persistence across restarts
- Key bindings for quick actions
