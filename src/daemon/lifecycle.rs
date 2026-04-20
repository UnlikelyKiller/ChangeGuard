use miette::{IntoDiagnostic, Result, miette};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use sysinfo::{Pid, System};
use tracing::{info, warn};

pub struct DaemonLifecycle {
    pid_file: PathBuf,
}

impl DaemonLifecycle {
    pub fn new(root: &Path) -> Self {
        let pid_file = root.join(".changeguard").join("daemon.pid");
        Self { pid_file }
    }

    pub fn setup(&self) -> Result<()> {
        if let Some(parent) = self.pid_file.parent() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }

        if self.pid_file.exists() {
            let content = fs::read_to_string(&self.pid_file).into_diagnostic()?;
            if let Ok(pid) = content.trim().parse::<u32>() {
                if self.is_process_alive(pid) {
                    return Err(miette!(
                        "Daemon is already running (PID: {}). Stop it first or delete {}",
                        pid,
                        self.pid_file.display()
                    ));
                } else {
                    warn!("Stale PID file found (PID: {}). Cleaning up.", pid);
                    self.cleanup()?;
                }
            }
        }

        let current_pid = process::id();
        fs::write(&self.pid_file, current_pid.to_string()).into_diagnostic()?;
        info!("Daemon started with PID: {}", current_pid);
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file).into_diagnostic()?;
            info!("Cleaned up PID file: {}", self.pid_file.display());
        }
        Ok(())
    }

    fn is_process_alive(&self, pid: u32) -> bool {
        let mut sys = System::new_all();
        sys.refresh_all();
        sys.process(Pid::from(pid as usize)).is_some()
    }
}

pub fn check_stdin_alive() -> bool {
    // In a real LSP, we might want to check if stdin is still open.
    // For now, tower-lsp handles the main loop, but we can provide this for the server to check.
    true
}
