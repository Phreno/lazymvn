//! Process management for Maven commands

/// Updates from async command execution
#[derive(Debug, Clone)]
pub enum CommandUpdate {
    Started(u32), // Process ID
    OutputLine(String),
    Completed,
    Error(String),
}

/// Kill a running process by PID
pub fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        if output.status.success() {
            log::info!("Successfully sent SIGTERM to process {}", pid);
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to kill process {}: {}", pid, error))
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .arg("/F")
            .output()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        if output.status.success() {
            log::info!("Successfully killed process {}", pid);
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to kill process {}: {}", pid, error))
        }
    }
}
