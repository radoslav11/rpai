use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// ============================================================================
// THEMES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ThemeName {
    Gruvbox,
    Nord,
    Catppuccin,
    Dracula,
    Tokyo,
    Solarized,
}

impl ThemeName {
    fn all() -> Vec<ThemeName> {
        vec![
            ThemeName::Gruvbox,
            ThemeName::Nord,
            ThemeName::Catppuccin,
            ThemeName::Dracula,
            ThemeName::Tokyo,
            ThemeName::Solarized,
        ]
    }

    fn next(&self) -> ThemeName {
        let all = Self::all();
        let idx = all.iter().position(|t| t == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    fn name(&self) -> &'static str {
        match self {
            ThemeName::Gruvbox => "gruvbox",
            ThemeName::Nord => "nord",
            ThemeName::Catppuccin => "catppuccin",
            ThemeName::Dracula => "dracula",
            ThemeName::Tokyo => "tokyo",
            ThemeName::Solarized => "solarized",
        }
    }

    fn from_str(s: &str) -> Option<ThemeName> {
        match s.to_lowercase().as_str() {
            "gruvbox" => Some(ThemeName::Gruvbox),
            "nord" => Some(ThemeName::Nord),
            "catppuccin" | "cat" => Some(ThemeName::Catppuccin),
            "dracula" => Some(ThemeName::Dracula),
            "tokyo" | "tokyonight" => Some(ThemeName::Tokyo),
            "solarized" | "solar" => Some(ThemeName::Solarized),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    fg: Color,
    dim: Color,
    accent: Color,
    green: Color,
    blue: Color,
    aqua: Color,
    orange: Color,
    selected_bg: Color,
    folder_icon: Color,
}

impl Theme {
    fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Gruvbox => Theme {
                fg: Color::Rgb(235, 219, 178),       // #ebdbb2
                dim: Color::Rgb(146, 131, 116),      // #928374
                accent: Color::Rgb(250, 189, 47),    // #fabd2f gold
                green: Color::Rgb(184, 187, 38),     // #b8bb26
                blue: Color::Rgb(131, 165, 152),     // #83a598
                aqua: Color::Rgb(142, 192, 124),     // #8ec07c
                orange: Color::Rgb(254, 128, 25),    // #fe8019
                selected_bg: Color::Rgb(80, 73, 69), // #504945
                folder_icon: Color::Rgb(250, 189, 47),
            },
            ThemeName::Nord => Theme {
                fg: Color::Rgb(236, 239, 244),       // #eceff4
                dim: Color::Rgb(76, 86, 106),        // #4c566a
                accent: Color::Rgb(136, 192, 208),   // #88c0d0 frost
                green: Color::Rgb(163, 190, 140),    // #a3be8c
                blue: Color::Rgb(129, 161, 193),     // #81a1c1
                aqua: Color::Rgb(143, 188, 187),     // #8fbcbb
                orange: Color::Rgb(208, 135, 112),   // #d08770
                selected_bg: Color::Rgb(67, 76, 94), // #434c5e
                folder_icon: Color::Rgb(235, 203, 139),
            },
            ThemeName::Catppuccin => Theme {
                fg: Color::Rgb(205, 214, 244),       // #cdd6f4 text
                dim: Color::Rgb(108, 112, 134),      // #6c7086 overlay0
                accent: Color::Rgb(245, 194, 231),   // #f5c2e7 pink
                green: Color::Rgb(166, 227, 161),    // #a6e3a1
                blue: Color::Rgb(137, 180, 250),     // #89b4fa
                aqua: Color::Rgb(148, 226, 213),     // #94e2d5 teal
                orange: Color::Rgb(250, 179, 135),   // #fab387 peach
                selected_bg: Color::Rgb(69, 71, 90), // #45475a surface0
                folder_icon: Color::Rgb(250, 179, 135),
            },
            ThemeName::Dracula => Theme {
                fg: Color::Rgb(248, 248, 242),       // #f8f8f2
                dim: Color::Rgb(98, 114, 164),       // #6272a4 comment
                accent: Color::Rgb(255, 121, 198),   // #ff79c6 pink
                green: Color::Rgb(80, 250, 123),     // #50fa7b
                blue: Color::Rgb(139, 233, 253),     // #8be9fd cyan
                aqua: Color::Rgb(139, 233, 253),     // #8be9fd
                orange: Color::Rgb(255, 184, 108),   // #ffb86c
                selected_bg: Color::Rgb(68, 71, 90), // #44475a
                folder_icon: Color::Rgb(255, 184, 108),
            },
            ThemeName::Tokyo => Theme {
                fg: Color::Rgb(192, 202, 245),       // #c0caf5
                dim: Color::Rgb(86, 95, 137),        // #565f89
                accent: Color::Rgb(187, 154, 247),   // #bb9af7 purple
                green: Color::Rgb(158, 206, 106),    // #9ece6a
                blue: Color::Rgb(125, 207, 255),     // #7dcfff
                aqua: Color::Rgb(115, 218, 202),     // #73daca
                orange: Color::Rgb(255, 158, 100),   // #ff9e64
                selected_bg: Color::Rgb(52, 59, 88), // #343b58
                folder_icon: Color::Rgb(224, 175, 104),
            },
            ThemeName::Solarized => Theme {
                fg: Color::Rgb(131, 148, 150),      // #839496 base0
                dim: Color::Rgb(88, 110, 117),      // #586e75 base01
                accent: Color::Rgb(181, 137, 0),    // #b58900 yellow
                green: Color::Rgb(133, 153, 0),     // #859900
                blue: Color::Rgb(38, 139, 210),     // #268bd2
                aqua: Color::Rgb(42, 161, 152),     // #2aa198 cyan
                orange: Color::Rgb(203, 75, 22),    // #cb4b16
                selected_bg: Color::Rgb(7, 54, 66), // #073642 base02
                folder_icon: Color::Rgb(38, 139, 210),
            },
        }
    }
}

// ============================================================================
// CONFIG
// ============================================================================

fn config_dir() -> PathBuf {
    env::var("HOME")
        .map(|h| PathBuf::from(h))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".config")
        .join("rpai")
}

fn ensure_config_dir() -> Result<PathBuf> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

// ============================================================================
// APP CONFIG
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    /// Theme name (default: gruvbox)
    #[serde(default = "default_theme")]
    theme: String,
    /// CPU percentage threshold below which a process is considered idle (default: 3.0)
    #[serde(default = "default_idle_threshold")]
    idle_threshold: f64,
    /// Refresh interval in milliseconds (default: 50)
    #[serde(default = "default_refresh_ms")]
    refresh_ms: u64,
    /// Use ASCII symbols instead of unicode (default: false)
    #[serde(default = "default_ascii_symbols")]
    ascii_symbols: bool,
}

fn default_theme() -> String {
    "gruvbox".to_string()
}

fn default_idle_threshold() -> f64 {
    3.0
}

fn default_refresh_ms() -> u64 {
    50
}

fn default_ascii_symbols() -> bool {
    false
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            idle_threshold: default_idle_threshold(),
            refresh_ms: default_refresh_ms(),
            ascii_symbols: default_ascii_symbols(),
        }
    }
}

fn load_config() -> AppConfig {
    let path = config_dir().join("config.json");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

fn save_config(config: &AppConfig) -> Result<()> {
    let dir = ensure_config_dir()?;
    let path = dir.join("config.json");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn load_theme() -> ThemeName {
    let config = load_config();
    ThemeName::from_str(&config.theme).unwrap_or(ThemeName::Gruvbox)
}

fn save_theme(theme: ThemeName) -> Result<()> {
    let mut config = load_config();
    config.theme = theme.name().to_string();
    save_config(&config)
}

// ============================================================================
// SESSION DATA
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SessionState {
    Running,
    Waiting,
}

impl SessionState {
    fn symbol(&self, ascii: bool) -> &str {
        match self {
            SessionState::Running => {
                if ascii {
                    ">>"
                } else {
                    "▶"
                }
            }
            SessionState::Waiting => {
                if ascii {
                    "||"
                } else {
                    "⏸"
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiSession {
    pid: u32,
    agent_type: String,
    working_dir: String,
    name: Option<String>,
    pane_id: Option<String>,
    session_name: Option<String>,
    window_index: Option<u32>,
    pane_width: Option<u32>,
    pane_height: Option<u32>,
    uptime_seconds: i64,
    memory_mb: u64,
    cpu_percent: f64,
    state: SessionState,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    ppid: u32,
    comm: String,
    cmd: Option<String>,
}

fn get_process_info_via_ps() -> Result<Vec<ProcessInfo>> {
    let output = Command::new("ps")
        .args(["-axo", "pid,ppid,comm,command"])
        .output()
        .map_err(|e| format!("Failed to execute ps command: {}", e))?;

    let mut processes = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let pid: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
                let ppid: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let comm = parts.get(2).unwrap_or(&"").to_string();
                let cmd = parts.get(3).map(|s| s.to_string());

                processes.push(ProcessInfo {
                    pid,
                    ppid,
                    comm,
                    cmd,
                });
            }
        }
    }

    Ok(processes)
}

fn get_descendant_pids(pid: u32, ps_output: &str) -> Vec<u32> {
    // Parse ps output to build parent->children map
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for line in ps_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(child_pid), Ok(parent_pid)) =
                (parts[0].parse::<u32>(), parts[1].parse::<u32>())
            {
                children.entry(parent_pid).or_default().push(child_pid);
            }
        }
    }

    // BFS to collect all descendants
    let mut result = vec![pid];
    let mut queue = vec![pid];
    while let Some(current) = queue.pop() {
        if let Some(kids) = children.get(&current) {
            for &kid in kids {
                result.push(kid);
                queue.push(kid);
            }
        }
    }
    result
}

/// LSP server patterns to exclude from CPU calculation.
/// These run in the background and don't indicate the AI agent is actively working.
const LSP_PATTERNS: &[&str] = &[
    "pyright-langserver",
    "pylsp",
    "python-lsp-server",
    "clangd",
    "typescript-language-server",
    "tsserver",
    "gopls",
    "rust-analyzer",
    "lua-language-server",
    "bash-language-server",
    "vscode-json-language",
    "yaml-language-server",
    "tailwindcss-language-server",
];

fn is_lsp_process(command: &str) -> bool {
    let cmd_lower = command.to_lowercase();
    LSP_PATTERNS
        .iter()
        .any(|pattern| cmd_lower.contains(pattern))
}

fn get_process_tree_cpu_usage(pid: u32) -> Option<f64> {
    // Get all processes with their parent PIDs
    let ps_output = Command::new("ps")
        .args(["-axo", "pid,ppid"])
        .output()
        .ok()?;

    if !ps_output.status.success() {
        return None;
    }

    let ps_str = String::from_utf8_lossy(&ps_output.stdout);
    let descendant_pids = get_descendant_pids(pid, &ps_str);

    // Get CPU usage and command for all descendant PIDs, filtering out LSP servers
    let pid_args: Vec<String> = descendant_pids.iter().map(|p| p.to_string()).collect();
    let output = Command::new("ps")
        .args(["-p", &pid_args.join(","), "-o", "pcpu=,command="])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let total: f64 = stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                // Parse "CPU COMMAND" format - CPU is first space-separated field
                let mut parts = line.splitn(2, char::is_whitespace);
                let cpu_str = parts.next()?.trim();
                let command = parts.next().unwrap_or("");

                // Skip LSP processes
                if is_lsp_process(command) {
                    return None;
                }

                cpu_str.parse::<f64>().ok()
            })
            .sum();
        Some(total)
    } else {
        None
    }
}

fn get_session_state_and_cpu(pid: u32, idle_threshold: f64) -> (SessionState, f64) {
    // Check CPU usage of the AI agent process and all its descendants
    // (LSP servers are filtered out in get_process_tree_cpu_usage)
    let cpu_pct = get_process_tree_cpu_usage(pid).unwrap_or(0.0);

    // Use CPU as the primary signal for determining state
    if cpu_pct > idle_threshold {
        (SessionState::Running, cpu_pct)
    } else {
        (SessionState::Waiting, cpu_pct)
    }
}

fn get_cwd_via_lsof(pid: u32) -> Option<String> {
    let output = Command::new("lsof")
        .args(["-p", &pid.to_string(), "-a"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(" cwd ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(path) = parts.last() {
                    return Some(path.to_string());
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
struct TmuxPaneInfo {
    pane_id: String,
    session_name: String,
    window_index: u32,
    pane_width: u32,
    pane_height: u32,
}

fn get_tmux_pane_info() -> Result<HashMap<u32, TmuxPaneInfo>> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_pid}\t#{pane_id}\t#{session_name}\t#{window_index}\t#{pane_width}\t#{pane_height}",
        ])
        .output()
        .map_err(|e| format!("Failed to execute tmux command: {}", e))?;

    let mut pane_map: HashMap<u32, TmuxPaneInfo> = HashMap::new();

    if output.status.success() {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 6 {
                let pid: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);

                if pid > 0 {
                    let pane_info = TmuxPaneInfo {
                        pane_id: parts.get(1).unwrap_or(&"").to_string(),
                        session_name: parts.get(2).unwrap_or(&"").to_string(),
                        window_index: parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
                        pane_width: parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0),
                        pane_height: parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0),
                    };
                    pane_map.insert(pid, pane_info);
                }
            }
        }
    }

    Ok(pane_map)
}

fn find_tmux_pane_for_pid(
    pid: u32,
    process_map: &HashMap<u32, ProcessInfo>,
    tmux_panes: &HashMap<u32, TmuxPaneInfo>,
) -> Option<(u32, TmuxPaneInfo)> {
    let mut current_pid = pid;
    let max_steps = 25;

    for _ in 0..max_steps {
        if let Some(pane_info) = tmux_panes.get(&current_pid) {
            return Some((current_pid, pane_info.clone()));
        }

        if let Some(process_info) = process_map.get(&current_pid) {
            current_pid = process_info.ppid;

            if current_pid == 0 || current_pid == 1 {
                break;
            }
        } else {
            break;
        }
    }

    None
}

fn scan_ai_processes() -> Result<Vec<AiSession>> {
    let config = load_config();
    let tmux_panes = get_tmux_pane_info().unwrap_or_default();

    let ps_processes = get_process_info_via_ps()?;
    let ps_map: HashMap<u32, ProcessInfo> = ps_processes
        .clone()
        .into_iter()
        .map(|p| (p.pid, p))
        .collect();

    let agent_pattern = Regex::new(r"(?i)(opencode|claude|codex|cursor|gemini)")?;

    // First pass: find all matching PIDs from ps (fast)
    let mut matched_pids: Vec<(u32, ProcessInfo)> = Vec::new();
    for process_info in ps_processes {
        let comm_lower = process_info.comm.to_lowercase();
        let cmd_lower = process_info
            .cmd
            .as_ref()
            .map(|c| c.to_lowercase())
            .unwrap_or_default();

        // Filter out system services
        if comm_lower.contains("cursoruiviewservice") || cmd_lower.contains("cursoruiviewservice") {
            continue;
        }

        let match_comm = agent_pattern.is_match(&comm_lower);
        let match_cmd = agent_pattern.is_match(&cmd_lower);

        if match_comm || match_cmd {
            matched_pids.push((process_info.pid, process_info));
        }
    }

    // Filter out subprocesses - only keep processes whose parent is not also an AI agent
    let matched_pid_set: std::collections::HashSet<u32> =
        matched_pids.iter().map(|(pid, _)| *pid).collect();
    let mut sessions = Vec::new();

    // Now load only matched PIDs into sysinfo (much faster than loading all)
    let mut system = System::new();
    let pid_list: Vec<sysinfo::Pid> = matched_pids
        .iter()
        .map(|(pid, _)| sysinfo::Pid::from_u32(*pid))
        .collect();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pid_list),
        true,
        ProcessRefreshKind::everything(),
    );

    for (pid, process_info) in matched_pids {
        // Skip if parent is also an AI agent
        if matched_pid_set.contains(&process_info.ppid) {
            continue;
        }

        let comm_lower = process_info.comm.to_lowercase();
        let cmd_lower = process_info
            .cmd
            .as_ref()
            .map(|c| c.to_lowercase())
            .unwrap_or_default();

        // Determine agent type
        let agent_type = if cmd_lower.contains("opencode") {
            "opencode"
        } else if cmd_lower.contains("claude") {
            "claude"
        } else if cmd_lower.contains("codex") {
            "codex"
        } else if cmd_lower.contains("cursor") {
            "cursor"
        } else if cmd_lower.contains("gemini") {
            "gemini"
        } else if comm_lower.contains("opencode") {
            "opencode"
        } else if comm_lower.contains("claude") {
            "claude"
        } else if comm_lower.contains("codex") {
            "codex"
        } else if comm_lower.contains("cursor") {
            "cursor"
        } else if comm_lower.contains("gemini") {
            "gemini"
        } else {
            "unknown"
        };

        let sysinfo_pid = sysinfo::Pid::from_u32(pid);
        if let Some(process) = system.process(sysinfo_pid) {
            let working_dir = process
                .cwd()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| get_cwd_via_lsof(pid).unwrap_or_else(|| "unknown".to_string()));

            let uptime = Duration::from_secs(process.run_time() as u64);
            let memory_mb = process.memory() / 1024 / 1024;

            let tmux_info = find_tmux_pane_for_pid(pid, &ps_map, &tmux_panes);

            let (pane_id, session_name, window_index, pane_width, pane_height) =
                if let Some((_pane_pid, info)) = tmux_info {
                    (
                        Some(info.pane_id.clone()),
                        Some(info.session_name.clone()),
                        Some(info.window_index),
                        Some(info.pane_width),
                        Some(info.pane_height),
                    )
                } else {
                    (None, None, None, None, None)
                };

            let (state, cpu_percent) = get_session_state_and_cpu(pid, config.idle_threshold);

            sessions.push(AiSession {
                pid,
                agent_type: agent_type.to_string(),
                working_dir,
                name: None,
                pane_id,
                session_name,
                window_index,
                pane_width,
                pane_height,
                uptime_seconds: uptime.as_secs() as i64,
                memory_mb,
                cpu_percent,
                state,
            });
        }
    }

    sessions.sort_by(|a, b| match a.agent_type.cmp(&b.agent_type) {
        std::cmp::Ordering::Equal => a.pid.cmp(&b.pid),
        other => other,
    });

    Ok(sessions)
}

fn format_duration(seconds: i64) -> String {
    let minutes = seconds / 60;
    let hours = minutes / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

// Format path with visual hierarchy
fn format_path_visual(path: &str, max_len: usize, theme: &Theme) -> Vec<Span<'static>> {
    // Handle home directory
    let home = env::var("HOME").ok();
    let display_path = if let Some(ref home_path) = home {
        if path.starts_with(home_path) {
            format!("~{}", &path[home_path.len()..])
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };

    let parts: Vec<&str> = display_path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return vec![Span::styled(
            path.to_string(),
            Style::default().fg(theme.dim),
        )];
    }

    // Calculate if we need to truncate
    let total_len: usize = parts.iter().map(|p| p.len() + 3).sum(); // +3 for " / "

    let mut spans = Vec::new();

    // Folder icon
    spans.push(Span::styled(" ", Style::default().fg(theme.folder_icon)));

    if total_len > max_len && parts.len() > 2 {
        // Show first part, ..., last two parts
        spans.push(Span::styled(
            format!(" {}", parts[0]),
            Style::default().fg(theme.fg),
        ));
        spans.push(Span::styled(" / ... / ", Style::default().fg(theme.dim)));

        let last_idx = parts.len() - 1;
        if parts.len() > 2 {
            spans.push(Span::styled(
                parts[last_idx - 1].to_string(),
                Style::default().fg(theme.fg),
            ));
            spans.push(Span::styled(" / ", Style::default().fg(theme.dim)));
        }
        spans.push(Span::styled(
            parts[last_idx].to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        // Show full path with styled separators
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" / ", Style::default().fg(theme.dim)));
            } else {
                spans.push(Span::styled(" ", Style::default()));
            }

            // Last part is brighter/accented
            let style = if i == parts.len() - 1 {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            spans.push(Span::styled(part.to_string(), style));
        }
    }

    spans
}

// ============================================================================
// TUI APP
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Normal,
    Command,
}

struct App {
    sessions: Vec<AiSession>,
    list_state: ListState,
    should_quit: bool,
    selected_session: Option<usize>,
    theme_name: ThemeName,
    theme: Theme,
    mode: AppMode,
    command_input: String,
    status_message: Option<String>,
    last_refresh: Instant,
    config: AppConfig,
}

impl App {
    fn new(sessions: Vec<AiSession>) -> Self {
        let config = load_config();
        let theme_name = load_theme();
        let theme = Theme::from_name(theme_name);
        let mut list_state = ListState::default();
        if !sessions.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            sessions,
            list_state,
            should_quit: false,
            selected_session: None,
            theme_name,
            theme,
            mode: AppMode::Normal,
            command_input: String::new(),
            status_message: None,
            last_refresh: Instant::now(),
            config,
        }
    }

    fn next(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.sessions.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.sessions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select(&mut self) {
        self.selected_session = self.list_state.selected();
        self.should_quit = true;
    }

    fn set_theme(&mut self, name: ThemeName) {
        self.theme_name = name;
        self.theme = Theme::from_name(name);
        let _ = save_theme(name);
        self.status_message = Some(format!("Theme set to: {}", name.name()));
    }

    fn cycle_theme(&mut self) {
        self.set_theme(self.theme_name.next());
    }

    fn execute_command(&mut self) {
        let cmd = self.command_input.trim().to_lowercase();

        if cmd.starts_with("theme") {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.len() > 1 {
                if let Some(theme) = ThemeName::from_str(parts[1]) {
                    self.set_theme(theme);
                } else {
                    self.status_message = Some(format!(
                        "Unknown theme. Available: {}",
                        ThemeName::all()
                            .iter()
                            .map(|t| t.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            } else {
                self.cycle_theme();
            }
        } else if cmd == "themes" || cmd == "list" {
            self.status_message = Some(format!(
                "Themes: {}",
                ThemeName::all()
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        } else if cmd == "q" || cmd == "quit" {
            self.should_quit = true;
        } else if !cmd.is_empty() {
            self.status_message = Some(format!("Unknown command: {}", cmd));
        }

        self.command_input.clear();
        self.mode = AppMode::Normal;
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let theme = &app.theme;

    // Main layout: header, list, status/command, help bar
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(5),    // List
        Constraint::Length(1), // Status/command line
        Constraint::Length(2), // Help bar
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " rpai ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("- AI Agent Sessions", Style::default().fg(theme.fg)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.dim)),
    );
    frame.render_widget(header, chunks[0]);

    // Session list
    if app.sessions.is_empty() {
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            "  No AI agent processes detected",
            Style::default().fg(theme.orange),
        )]))
        .block(Block::default());
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = app.list_state.selected() == Some(i);
                create_session_list_item(
                    session,
                    i,
                    is_selected,
                    chunks[1].width,
                    theme,
                    &app.config,
                )
            })
            .collect();

        let list = List::new(items)
            .block(Block::default())
            .highlight_style(Style::default().bg(theme.selected_bg));

        frame.render_stateful_widget(list, chunks[1], &mut app.list_state);
    }

    // Status/command line
    let status_line = match &app.mode {
        AppMode::Command => Paragraph::new(Line::from(vec![
            Span::styled(":", Style::default().fg(theme.accent)),
            Span::styled(app.command_input.clone(), Style::default().fg(theme.fg)),
            Span::styled("_", Style::default().fg(theme.accent)),
        ])),
        AppMode::Normal => {
            if let Some(msg) = &app.status_message {
                Paragraph::new(Line::from(vec![Span::styled(
                    format!(" {}", msg),
                    Style::default().fg(theme.aqua),
                )]))
            } else {
                Paragraph::new("")
            }
        }
    };
    frame.render_widget(status_line, chunks[2]);

    // Help bar
    let help_spans = if app.mode == AppMode::Command {
        vec![
            Span::styled(" Enter", Style::default().fg(theme.green)),
            Span::styled(" execute  ", Style::default().fg(theme.dim)),
            Span::styled("Esc", Style::default().fg(theme.green)),
            Span::styled(" cancel", Style::default().fg(theme.dim)),
        ]
    } else {
        vec![
            Span::styled(" j/k", Style::default().fg(theme.green)),
            Span::styled(" nav  ", Style::default().fg(theme.dim)),
            Span::styled("Enter", Style::default().fg(theme.green)),
            Span::styled(" jump  ", Style::default().fg(theme.dim)),
            Span::styled("/", Style::default().fg(theme.green)),
            Span::styled(" cmd  ", Style::default().fg(theme.dim)),
            Span::styled("t", Style::default().fg(theme.green)),
            Span::styled(" theme  ", Style::default().fg(theme.dim)),
            Span::styled("q", Style::default().fg(theme.green)),
            Span::styled(" quit", Style::default().fg(theme.dim)),
        ]
    };

    let help = Paragraph::new(Line::from(help_spans)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.dim)),
    );
    frame.render_widget(help, chunks[3]);
}

fn create_session_list_item(
    session: &AiSession,
    idx: usize,
    is_selected: bool,
    width: u16,
    theme: &Theme,
    config: &AppConfig,
) -> ListItem<'static> {
    let prefix = if is_selected { " " } else { "  " };
    let prefix_style = if is_selected {
        Style::default()
            .fg(theme.green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };

    // First line: agent type and uptime
    let state_color = if session.state == SessionState::Running {
        theme.green
    } else {
        theme.orange
    };
    let line1 = Line::from(vec![
        Span::styled(prefix, prefix_style),
        Span::styled(format!("[{}] ", idx + 1), Style::default().fg(theme.dim)),
        Span::styled(
            format!("{:<10}", session.agent_type),
            Style::default().fg(theme.aqua).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | ", Style::default().fg(theme.dim)),
        Span::styled(
            session.state.symbol(config.ascii_symbols).to_string(),
            Style::default().fg(state_color),
        ),
        Span::styled(" ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("PID: {}", session.pid),
            Style::default().fg(theme.green),
        ),
        Span::styled(" | ", Style::default().fg(theme.dim)),
        Span::styled(" ", Style::default().fg(theme.dim)),
        Span::styled(
            format_duration(session.uptime_seconds),
            Style::default().fg(theme.fg),
        ),
        Span::styled(" | ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("CPU: {:.1}%", session.cpu_percent),
            Style::default().fg(state_color),
        ),
        Span::styled(" | ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("MEM: {}MB", session.memory_mb),
            Style::default().fg(theme.fg),
        ),
    ]);

    // Second line: PID and tmux info
    let line2 = if let (Some(session_name), Some(window_index), Some(pane_id)) = (
        &session.session_name,
        &session.window_index,
        &session.pane_id,
    ) {
        Line::from(vec![
            Span::styled("     ", Style::default()),
            Span::styled(" ", Style::default().fg(theme.blue)),
            Span::styled(" ", Style::default()),
            Span::styled(
                format!("{}:{} {}", session_name, window_index, pane_id,),
                Style::default().fg(theme.blue),
            ),
            Span::styled(
                format!(
                    " [{}x{}]",
                    session.pane_width.unwrap_or(0),
                    session.pane_height.unwrap_or(0)
                ),
                Style::default().fg(theme.dim),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("     ", Style::default()),
            Span::styled(" ", Style::default().fg(theme.dim)),
            Span::styled(" not in tmux", Style::default().fg(theme.dim)),
        ])
    };

    // Third line: working directory with visual formatting
    let max_cwd_len = (width as usize).saturating_sub(10);
    let mut path_spans = vec![Span::styled("     ", Style::default())];
    path_spans.extend(format_path_visual(&session.working_dir, max_cwd_len, theme));
    let line3 = Line::from(path_spans);

    // Empty line for spacing
    let line4 = Line::from("");

    ListItem::new(vec![line1, line2, line3, line4])
}

fn run_tui(sessions: Vec<AiSession>, refresh_ms: u64) -> Result<Option<AiSession>> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new(sessions);

    // Calculate lines per session item (4 lines each)
    let lines_per_item = 4;

    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        // Check for events with configurable timeout
        if event::poll(Duration::from_millis(refresh_ms))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        // Clear status message on any key
                        app.status_message = None;

                        match &app.mode {
                            AppMode::Command => match key.code {
                                KeyCode::Enter => {
                                    app.execute_command();
                                }
                                KeyCode::Esc => {
                                    app.command_input.clear();
                                    app.mode = AppMode::Normal;
                                }
                                KeyCode::Backspace => {
                                    app.command_input.pop();
                                }
                                KeyCode::Char(c) => {
                                    app.command_input.push(c);
                                }
                                _ => {}
                            },
                            AppMode::Normal => {
                                // Ctrl-C handling
                                if key.modifiers.contains(KeyModifiers::CONTROL)
                                    && key.code == KeyCode::Char('c')
                                {
                                    app.should_quit = true;
                                } else {
                                    match key.code {
                                        KeyCode::Char('q') | KeyCode::Esc => {
                                            app.should_quit = true;
                                        }
                                        KeyCode::Char('/') | KeyCode::Char(':') => {
                                            app.mode = AppMode::Command;
                                        }
                                        KeyCode::Char('t') => {
                                            app.cycle_theme();
                                        }
                                        KeyCode::Down | KeyCode::Char('j') => {
                                            app.next();
                                        }
                                        KeyCode::Up | KeyCode::Char('k') => {
                                            app.previous();
                                        }
                                        KeyCode::Enter => {
                                            app.select();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if app.mode == AppMode::Normal {
                        match mouse.kind {
                            MouseEventKind::Down(_) => {
                                // Calculate which session was clicked
                                // Header is 3 lines, so list starts at row 3
                                let list_start_row = 3u16;
                                if mouse.row >= list_start_row && !app.sessions.is_empty() {
                                    let clicked_row = (mouse.row - list_start_row) as usize;
                                    let clicked_index = clicked_row / lines_per_item;
                                    if clicked_index < app.sessions.len() {
                                        app.list_state.select(Some(clicked_index));
                                    }
                                }
                            }
                            MouseEventKind::ScrollDown => {
                                app.next();
                            }
                            MouseEventKind::ScrollUp => {
                                app.previous();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        } else {
            // Timeout - refresh sessions
            if let Ok(new_sessions) = scan_ai_processes() {
                // Preserve selected session by PID
                let selected_pid = app
                    .list_state
                    .selected()
                    .and_then(|i| app.sessions.get(i))
                    .map(|s| s.pid);

                app.sessions = new_sessions;

                // Restore selection
                if let Some(pid) = selected_pid {
                    let new_index = app.sessions.iter().position(|s| s.pid == pid);
                    app.list_state.select(new_index);
                }
            }
            app.last_refresh = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(app
        .selected_session
        .and_then(|i| app.sessions.get(i).cloned()))
}

fn jump_to_session(session: &AiSession) -> Result<()> {
    if let (Some(session_name), Some(window_index), Some(pane_id)) = (
        &session.session_name,
        &session.window_index,
        &session.pane_id,
    ) {
        let pane_target = format!("{}:{}.{}", session_name, window_index, pane_id);

        // Check if we're inside a tmux session
        let in_tmux = std::env::var("TMUX").is_ok();

        if in_tmux {
            // Use switch-client when inside tmux
            let output = Command::new("tmux")
                .args(["switch-client", "-t", &pane_target])
                .output()
                .map_err(|e| format!("Failed to execute tmux switch-client command: {}", e))?;

            if output.status.success() {
                println!(
                    "Switched to session: {} (Window: {}, Pane: {})",
                    session_name, window_index, pane_id
                );
            } else {
                println!("Failed to switch to session");
            }
        } else {
            // Use attach-session when outside tmux - must exec to take over terminal
            use std::os::unix::process::CommandExt;
            let err = Command::new("tmux")
                .args(["attach-session", "-t", &pane_target])
                .exec();
            // exec only returns on error
            println!("Failed to attach to session: {}", err);
        }
    } else {
        println!("No tmux session info available for this process");
    }

    Ok(())
}

fn display_sessions(sessions: &[AiSession], config: &AppConfig) {
    if sessions.is_empty() {
        println!("No AI agent processes detected");
        return;
    }

    println!("AI Agent Sessions:");
    println!();

    for (i, session) in sessions.iter().enumerate() {
        println!(
            "[{}] {} {} | {} | PID: {} | CPU: {:.1}% | MEM: {}MB",
            i + 1,
            session.agent_type,
            session.state.symbol(config.ascii_symbols),
            format_duration(session.uptime_seconds),
            session.pid,
            session.cpu_percent,
            session.memory_mb
        );

        if let (Some(session_name), Some(window_index), Some(pane_id)) = (
            &session.session_name,
            &session.window_index,
            &session.pane_id,
        ) {
            println!(
                "     {}:{} {} [{}x{}]",
                session_name,
                window_index,
                pane_id,
                session.pane_width.unwrap_or(0),
                session.pane_height.unwrap_or(0)
            );
        }

        println!("     {}", session.working_dir);

        if i < sessions.len() - 1 {
            println!();
        }
    }
}

fn kill_session(id: usize) -> Result<()> {
    let sessions = scan_ai_processes()?;

    if id == 0 || id > sessions.len() {
        println!("Invalid session ID: {}", id);
        println!("Use 'rpai scan' to see available sessions");
        return Ok(());
    }

    let session = &sessions[id - 1];
    let pid = session.pid;

    let output = Command::new("kill")
        .args([pid.to_string().as_str()])
        .output()
        .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
    if output.status.success() {
        println!("Killed session [{}] (PID: {})", id, pid);
    } else {
        println!("Failed to kill session [{}] (PID: {})", id, pid);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !stderr.is_empty() {
            eprintln!("Error: {}", stderr);
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("scan") => {
            let config = load_config();
            let sessions = scan_ai_processes()?;
            display_sessions(&sessions, &config);
        }
        Some("kill") => {
            if let Some(id_str) = args.get(2) {
                if let Ok(id) = id_str.parse::<usize>() {
                    kill_session(id)?;
                } else {
                    println!("Invalid ID: {}", id_str);
                    println!("Use 'rpai kill <id>' where <id> is a number");
                }
            } else {
                println!("Usage: rpai kill <id>");
                println!("Use 'rpai scan' to see available sessions");
            }
        }
        Some("jump") => {
            if let Some(id_str) = args.get(2) {
                let sessions = scan_ai_processes()?;
                // Try parsing as numeric ID first
                if let Ok(id) = id_str.parse::<usize>() {
                    if let Some(session) = sessions.get(id.saturating_sub(1)) {
                        jump_to_session(session)?;
                    } else {
                        println!("Invalid ID: {}", id);
                        println!("Use 'rpai scan' to see available sessions");
                    }
                } else {
                    // Try matching by session name
                    let matching: Vec<_> = sessions
                        .iter()
                        .filter(|s| {
                            s.session_name
                                .as_ref()
                                .map(|n| n == id_str || n.contains(id_str))
                                .unwrap_or(false)
                        })
                        .collect();

                    match matching.len() {
                        0 => {
                            println!("No session found matching: {}", id_str);
                            println!("Use 'rpai scan' to see available sessions");
                        }
                        1 => {
                            jump_to_session(matching[0])?;
                        }
                        _ => {
                            println!("Multiple sessions match '{}'. Be more specific:", id_str);
                            for s in matching {
                                if let Some(name) = &s.session_name {
                                    println!("  - {}", name);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Usage: rpai jump <id|name>");
                println!("Use 'rpai scan' to see available sessions");
            }
        }
        Some("theme") => {
            if let Some(theme_name) = args.get(2) {
                if let Some(theme) = ThemeName::from_str(theme_name) {
                    save_theme(theme)?;
                    println!("Theme set to: {}", theme.name());
                } else {
                    println!("Unknown theme: {}", theme_name);
                    println!(
                        "Available themes: {}",
                        ThemeName::all()
                            .iter()
                            .map(|t| t.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            } else {
                println!("Current theme: {}", load_theme().name());
                println!(
                    "Available themes: {}",
                    ThemeName::all()
                        .iter()
                        .map(|t| t.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        Some("help") | Some("-h") | Some("--help") => {
            println!("rpai - AI Agent Session Manager for tmux");
            println!();
            println!("Usage:");
            println!("  rpai                - Interactive TUI (default)");
            println!("  rpai scan           - Scan and display AI agent sessions");
            println!("  rpai jump <id|name> - Jump to session by ID or name");
            println!("  rpai kill <id>      - Terminate a session");
            println!("  rpai theme [name]   - Show/set theme");
            println!("  rpai help           - Show this help message");
            println!();
            println!("Keyboard shortcuts (TUI mode):");
            println!("  j/k or Up/Down      - Navigate sessions");
            println!("  Enter               - Jump to selected session");
            println!("  t                   - Cycle through themes");
            println!("  / or :              - Enter command mode");
            println!("  q, Esc, Ctrl-C      - Quit");
            println!();
            println!("Commands (type after /):");
            println!("  theme [name]        - Switch theme");
            println!("  themes              - List available themes");
            println!();
            println!(
                "Available themes: {}",
                ThemeName::all()
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!();
            println!("Config: ~/.config/rpai/");
        }
        _ => {
            let config = load_config();
            let sessions = scan_ai_processes()?;
            if let Some(selected) = run_tui(sessions, config.refresh_ms)? {
                jump_to_session(&selected)?;
            }
        }
    }

    Ok(())
}
