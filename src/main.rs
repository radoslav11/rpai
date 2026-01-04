use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use sysinfo::{ProcessRefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiSession {
    pid: u32,
    agent_type: String,
    working_dir: String,
    name: Option<String>,
    pane_id: Option<String>,
    status: SessionStatus,
    uptime_seconds: i64,
    cpu_usage: f32,
    memory_mb: u64,
    last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SessionStatus {
    Active,
    Idle,
    Stale,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    comm: String,
    cmd: Option<String>,
    cwd: Option<String>,
    parent_pid: Option<u32>,
}

fn get_process_info_via_ps() -> Result<Vec<ProcessInfo>> {
    let output = Command::new("ps")
        .args(["-axo", "pid,comm,command,pid,ppid"])
        .output()
        .context("Failed to execute ps command")?;

    let mut processes = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let pid: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
                let comm = parts.get(1).unwrap_or(&"").to_string();
                let cmd = parts.get(2).map(|s| s.to_string());
                let ppid: Option<u32> = parts.get(3).and_then(|s| s.parse().ok());

                processes.push(ProcessInfo {
                    pid,
                    comm,
                    cmd,
                    cwd: None,
                    parent_pid: ppid,
                });
            }
        }
    }

    Ok(processes)
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

fn scan_ai_processes() -> Result<Vec<AiSession>> {
    let mut system = System::new_all();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );

    let ps_processes = get_process_info_via_ps()?;
    let ps_map: HashMap<u32, ProcessInfo> = ps_processes.into_iter().map(|p| (p.pid, p)).collect();

    let agent_pattern = Regex::new(r"(?i)(opencode|claude|codex|cursor)")?;
    let mut sessions = Vec::new();

    for (pid, process) in system.processes() {
        let process_info = ps_map.get(&pid.as_u32());

        let (agent_type, is_match) = if let Some(info) = process_info {
            let comm_lower = info.comm.to_lowercase();
            let cmd_lower = info
                .cmd
                .as_ref()
                .map(|c| c.to_lowercase())
                .unwrap_or_default();

            // Filter out system services that happen to match agent names
            if comm_lower.contains("cursoruiviewservice")
                || cmd_lower.contains("cursoruiviewservice")
            {
                continue;
            }

            let match_comm = agent_pattern.is_match(&comm_lower);
            let match_cmd = agent_pattern.is_match(&cmd_lower);

            let matched = match_comm || match_cmd;
            let agent_type = if match_cmd {
                if cmd_lower.contains("opencode") {
                    "opencode"
                } else if cmd_lower.contains("claude") {
                    "claude"
                } else if cmd_lower.contains("codex") {
                    "codex"
                } else if cmd_lower.contains("cursor") {
                    "cursor"
                } else if comm_lower.contains("opencode") {
                    "opencode"
                } else if comm_lower.contains("claude") {
                    "claude"
                } else if comm_lower.contains("codex") {
                    "codex"
                } else if comm_lower.contains("cursor") {
                    "cursor"
                } else {
                    "unknown"
                }
            } else {
                if comm_lower.contains("opencode") {
                    "opencode"
                } else if comm_lower.contains("claude") {
                    "claude"
                } else if comm_lower.contains("codex") {
                    "codex"
                } else if comm_lower.contains("cursor") {
                    "cursor"
                } else {
                    "unknown"
                }
            };

            (agent_type.to_string(), matched)
        } else {
            let cmd = process.name();
            let cmd_str = match cmd.to_str() {
                Some(s) => s.to_string(),
                None => String::from_utf8_lossy(cmd.as_bytes()).to_string(),
            };
            let matched = agent_pattern.is_match(&cmd_str.to_lowercase());

            let agent_type = if matched {
                agent_pattern
                    .find(&cmd_str.to_lowercase())
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                continue;
            };

            (agent_type, matched)
        };

        if !is_match {
            continue;
        }

        let working_dir = process
            .cwd()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| {
                get_cwd_via_lsof(pid.as_u32()).unwrap_or_else(|| "unknown".to_string())
            });

        let uptime = Duration::seconds(process.run_time() as i64);
        let memory_mb = process.memory() / 1024 / 1024;
        let cpu_usage = process.cpu_usage();

        let status = AiSession::determine_status(uptime, cpu_usage);

        sessions.push(AiSession {
            pid: pid.as_u32(),
            agent_type,
            working_dir,
            name: None,
            pane_id: None,
            status,
            uptime_seconds: uptime.num_seconds(),
            cpu_usage,
            memory_mb,
            last_activity: None,
        });
    }

    Ok(sessions)
}

impl AiSession {
    fn determine_status(uptime: Duration, cpu_usage: f32) -> SessionStatus {
        if cpu_usage > 1.0 || uptime.num_minutes() < 1 {
            SessionStatus::Active
        } else if uptime.num_minutes() < 30 {
            SessionStatus::Idle
        } else {
            SessionStatus::Stale
        }
    }
}

fn display_sessions(sessions: &[AiSession]) {
    if sessions.is_empty() {
        println!("No AI agent processes detected");
        return;
    }

    println!("AI Agent Sessions:");
    println!();

    for (i, session) in sessions.iter().enumerate() {
        let status_indicator = match session.status {
            SessionStatus::Active => "●",
            SessionStatus::Idle => "○",
            SessionStatus::Stale => "○",
        };

        println!(
            "{} [{}] {} | {} | {}",
            status_indicator,
            i + 1,
            session.agent_type,
            format_status(&session.status),
            format_duration(session.uptime_seconds)
        );

        println!(
            "    PID: {} | Mem: {}MB | CPU: {:.1}%",
            session.pid, session.memory_mb, session.cpu_usage
        );

        let cwd = if session.working_dir.len() > 60 {
            format!(
                "...{}",
                &session.working_dir[session.working_dir.len() - 57..]
            )
        } else {
            session.working_dir.clone()
        };
        println!("    {}", cwd);

        if i < sessions.len() - 1 {
            println!();
        }
    }
}

fn format_status(status: &SessionStatus) -> String {
    match status {
        SessionStatus::Active => "Active".to_string(),
        SessionStatus::Idle => "Idle".to_string(),
        SessionStatus::Stale => "Stale".to_string(),
    }
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

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("scan") => {
            let sessions = scan_ai_processes()?;
            display_sessions(&sessions);
        }
        Some("help") | Some("-h") | Some("--help") => {
            println!("rpai - Tmux plugin for AI agent session management");
            println!();
            println!("Usage:");
            println!("  rpai scan          - Scan and display AI agent sessions");
            println!("  rpai kill <id>     - Terminate a session");
            println!("  rpai help           - Show this help message");
        }
        _ => {
            println!("Use 'rpai help' for usage information");
        }
    }

    Ok(())
}
