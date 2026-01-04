use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use sysinfo::{Pid, ProcessRefreshKind, System};

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

impl AiSession {
    fn from_process(pid: Pid, agent_type: &str, system: &System) -> Result<Self> {
        let process = system.process(pid).context("Process not found")?;

        let working_dir = process
            .cwd()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let uptime = Duration::seconds(process.run_time() as i64);
        let memory_mb = process.memory() / 1024 / 1024;
        let cpu_usage = process.cpu_usage();

        let status = Self::determine_status(uptime, cpu_usage);

        Ok(AiSession {
            pid: pid.as_u32(),
            agent_type: agent_type.to_string(),
            working_dir,
            name: None,
            pane_id: None,
            status,
            uptime_seconds: uptime.num_seconds(),
            cpu_usage,
            memory_mb,
            last_activity: None,
        })
    }

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

fn os_str_to_str(s: &std::ffi::OsStr) -> String {
    match s.to_str() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(s.as_bytes()).to_string(),
    }
}

fn scan_ai_processes() -> Result<Vec<AiSession>> {
    let mut system = System::new_all();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );

    let agent_pattern = Regex::new(r"(?i)(opencode|claude|codex|aider|cursor)")?;
    let mut sessions = Vec::new();

    for (pid, process) in system.processes() {
        let cmd = os_str_to_str(process.name());
        if agent_pattern.is_match(&cmd) {
            let agent_type = agent_pattern
                .find(&cmd)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            if let Ok(session) = AiSession::from_process(*pid, &agent_type, &system) {
                sessions.push(session);
            }
        }
    }

    Ok(sessions)
}

fn get_tmux_session_info() -> Result<HashMap<String, String>> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_pid}\t#{pane_id}\t#{pane_current_path}",
        ])
        .output()
        .context("Failed to execute tmux command")?;

    let mut pane_info = HashMap::new();

    if output.status.success() {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let pid = parts[0].to_string();
                let pane_id = parts[1].to_string();
                pane_info.insert(pid, pane_id);
            }
        }
    }

    Ok(pane_info)
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
            println!("  rpai rename <id>   - Rename a session");
            println!("  rpai kill <id>      - Terminate a session");
            println!("  rpai help           - Show this help message");
        }
        _ => {
            println!("Use 'rpai help' for usage information");
        }
    }

    Ok(())
}
