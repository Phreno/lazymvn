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

        // Try to kill the entire process group (negative PID)
        // This ensures we kill Maven and all its child processes (like Spring Boot)
        log::info!("Attempting to kill process group for PID: {}", pid);

        // First try to kill the process group
        let group_result = Command::new("kill")
            .arg("-TERM")
            .arg(format!("-{}", pid)) // Negative PID kills the process group
            .output();

        match group_result {
            Ok(output) if output.status.success() => {
                log::info!("Successfully sent SIGTERM to process group {}", pid);

                // Wait a bit for graceful shutdown
                std::thread::sleep(std::time::Duration::from_millis(100));

                // Force kill if still running
                let _ = Command::new("kill")
                    .arg("-KILL")
                    .arg(format!("-{}", pid))
                    .output();

                return Ok(());
            }
            _ => {
                log::warn!("Failed to kill process group, trying individual process");
            }
        }

        // Fallback: kill individual process
        let output = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        if output.status.success() {
            log::info!("Successfully sent SIGTERM to process {}", pid);

            // Also try to kill with SIGKILL after a short delay
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = Command::new("kill")
                .arg("-KILL")
                .arg(pid.to_string())
                .output();

            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to kill process {}: {}", pid, error))
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;

        // On Windows, /T flag kills the process tree
        let output = Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .arg("/T") // Kill process tree (all child processes)
            .arg("/F") // Force kill
            .output()
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        if output.status.success() {
            log::info!("Successfully killed process tree {}", pid);
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to kill process {}: {}", pid, error))
        }
    }
}
