use sysinfo::{Pid, ProcessRefreshKind, System, SystemExt};
use std::os::unix::ffi::OsStrExt;

fn main() {
    let mut system = System::new_all();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything()
    );

    for (pid, process) in system.processes() {
        let cmd = process.name();
        let cmd_str = match cmd.to_str() {
            Some(s) => s.to_string(),
            None => String::from_utf8_lossy(cmd.as_bytes()).to_string(),
        };
        
        if cmd_str.to_lowercase().contains("claude") || pid.as_u32() == 70413 {
            println!("PID: {}, Name: '{}', CWD: {:?}",
                pid.as_u32(),
                cmd_str,
                process.cwd()
            );
        }
    }
}
