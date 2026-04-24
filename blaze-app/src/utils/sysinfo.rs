pub fn log_memory_usage(label: &str) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            let get = |key: &str| -> String {
                status.lines()
                    .find(|l| l.starts_with(key))
                    .map(|l| l.split_whitespace().nth(1).unwrap_or("?").to_string())
                    .unwrap_or("?".to_string())
            };

        }
    }
}


use std::time::Duration;

use sysinfo::{System};

pub fn get_stats() -> (f32, f64) {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let pid = sysinfo::get_current_pid().unwrap();
    if let Some(proc) = sys.process(pid) {
        let cpu = proc.cpu_usage();
        let mem_mb = proc.memory() as f64 / 1024.0 / 1024.0;
        (cpu, mem_mb)
    } else {
        (0.0, 0.0)
    }
}