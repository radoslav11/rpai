use sysinfo::{ProcessRefreshKind, System};
use chrono::Duration;

fn main() {
    let mut system = System::new_all();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything()
    );

    for (pid, process) in system.processes() {
        if pid.as_u32() == 71225 {
            let uptime = Duration::seconds(process.run_time() as i64);
            let cpu_usage = process.cpu_usage();
            println!("PID: {}", pid.as_u32());
            println!("Uptime: {}s", uptime.num_seconds());
            println!("CPU: {:.1}%", cpu_usage);
            println!("Minutes: {}", uptime.num_minutes());

            let status = if cpu_usage > 1.0 || uptime.num_minutes() < 1 {
                "Active"
            } else if uptime.num_minutes() < 30 {
                "Idle"
            } else {
                "Stale"
            };
            println!("Status: {}", status);
        }
    }
}
