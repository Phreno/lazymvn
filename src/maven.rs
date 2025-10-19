use crate::utils;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

/// Updates from async command execution
#[derive(Debug, Clone)]
pub enum CommandUpdate {
    Started(u32), // Process ID
    OutputLine(String),
    Completed,
    Error(String),
}

pub fn get_maven_command(project_root: &Path) -> String {
    // On Unix, check for mvnw
    #[cfg(unix)]
    {
        if project_root.join("mvnw").exists() {
            return "./mvnw".to_string();
        }
    }

    // On Windows, check for mvnw.bat, mvnw.cmd, or mvnw
    #[cfg(windows)]
    {
        if project_root.join("mvnw.bat").exists() {
            return "mvnw.bat".to_string();
        }
        if project_root.join("mvnw.cmd").exists() {
            return "mvnw.cmd".to_string();
        }
        if project_root.join("mvnw").exists() {
            return "mvnw".to_string();
        }
    }

    // On Windows, use mvn.cmd; on Unix, use mvn
    #[cfg(windows)]
    {
        "mvn.cmd".to_string()
    }
    #[cfg(not(windows))]
    {
        "mvn".to_string()
    }
}

/// Build the full command string for display
pub fn build_command_string(
    maven_command: &str,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> String {
    let mut parts = vec![maven_command.to_string()];

    if let Some(settings_path) = settings_path {
        parts.push("--settings".to_string());
        parts.push(settings_path.to_string());
    }

    if !profiles.is_empty() {
        parts.push("-P".to_string());
        parts.push(profiles.join(","));
    }

    if let Some(module) = module {
        if module != "." {
            parts.push("-pl".to_string());
            parts.push(module.to_string());
        }
    }

    for flag in flags {
        parts.push(flag.to_string());
    }

    for arg in args {
        parts.push(arg.to_string());
    }

    parts.join(" ")
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

pub fn check_maven_availability(project_root: &Path) -> Result<String, std::io::Error> {
    let maven_command = get_maven_command(project_root);

    let output = Command::new(&maven_command)
        .arg("--version")
        .current_dir(project_root)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Maven command '{}' failed", maven_command),
        ));
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let first_line = version_output
        .lines()
        .next()
        .unwrap_or("Unknown version")
        .to_string();

    Ok(first_line)
}

pub fn execute_maven_command(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> Result<Vec<String>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    log::debug!("execute_maven_command: Using command: {}", maven_command);
    log::debug!("  project_root: {:?}", project_root);
    log::debug!("  module: {:?}", module);
    log::debug!("  args: {:?}", args);
    log::debug!("  profiles: {:?}", profiles);
    log::debug!("  settings_path: {:?}", settings_path);
    log::debug!("  flags: {:?}", flags);

    let mut command = Command::new(&maven_command);
    if let Some(settings_path) = settings_path {
        command.arg("--settings").arg(settings_path);
        log::debug!("Added settings argument: {}", settings_path);
    }
    if !profiles.is_empty() {
        let profile_str = profiles.join(",");
        command.arg("-P").arg(&profile_str);
        log::debug!("Added profiles: {}", profile_str);
    }
    if let Some(module) = module {
        // Only use -pl flag if module is not "." (project root)
        if module != "." {
            command.arg("-pl").arg(module);
            log::debug!("Scoped to module: {}", module);
        } else {
            log::debug!("Running on project root, no -pl flag needed");
        }
    }
    // Add build flags
    for flag in flags {
        command.arg(flag);
        log::debug!("Added flag: {}", flag);
    }

    // Build the full command string for display
    let command_str = build_command_string(&maven_command, module, args, profiles, settings_path, flags);
    log::info!("Executing: {}", command_str);

    log::info!("Spawning Maven process...");
    let mut child = command
        .args(args)
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    log::debug!("Maven process spawned with PID: {:?}", child.id());
    
    // Start with the command string as the first line
    let mut output = vec![format!("$ {}", command_str), String::new()];
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line
                    && let Some(text) = utils::clean_log_line(&line)
                {
                    let _ = tx.send(text);
                }
            }
        }));
    }

    if let Some(stderr) = child.stderr.take() {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line
                    && let Some(text) = utils::clean_log_line(&line)
                {
                    let _ = tx.send(format!("[ERR] {text}"));
                }
            }
        }));
    }

    drop(tx);

    let mut line_count = 0;
    for line in rx {
        output.push(line);
        line_count += 1;
    }
    log::debug!("Received {} lines of output from Maven", line_count);

    for handle in handles {
        let _ = handle.join();
    }

    let exit_status = child.wait()?;
    log::info!("Maven process completed with status: {:?}", exit_status);
    if !exit_status.success() {
        log::warn!(
            "Maven command failed with exit code: {:?}",
            exit_status.code()
        );
    }

    Ok(output)
}

/// Async version that streams output line by line
/// Returns a receiver that will receive output lines as they arrive
pub fn execute_maven_command_async(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> Result<mpsc::Receiver<CommandUpdate>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    log::debug!(
        "execute_maven_command_async: Using command: {}",
        maven_command
    );
    log::debug!("  project_root: {:?}", project_root);
    log::debug!("  module: {:?}", module);
    log::debug!("  args: {:?}", args);

    // Build the full command string for display
    let command_str = build_command_string(&maven_command, module, args, profiles, settings_path, flags);
    log::info!("Executing: {}", command_str);

    let mut command = Command::new(maven_command);
    if let Some(settings_path) = settings_path {
        command.arg("--settings").arg(settings_path);
    }
    if !profiles.is_empty() {
        let profile_str = profiles.join(",");
        command.arg("-P").arg(&profile_str);
    }
    if let Some(module) = module
        && module != "."
    {
        command.arg("-pl").arg(module);
    }
    for flag in flags {
        command.arg(flag);
    }

    let project_root = project_root.to_path_buf();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let (tx, rx) = mpsc::channel();

    // Send the command string as the first output line
    let _ = tx.send(CommandUpdate::OutputLine(format!("$ {}", command_str)));
    let _ = tx.send(CommandUpdate::OutputLine(String::new()));

    // Spawn command execution in background thread
    thread::spawn(move || {
        let result = (|| -> Result<(), std::io::Error> {
            log::info!("Spawning Maven process asynchronously...");
            let mut child = command
                .args(&args)
                .current_dir(&project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let pid = child.id();
            log::debug!("Maven process spawned with PID: {}", pid);

            // Send the PID immediately so it can be stored for potential kill
            let _ = tx.send(CommandUpdate::Started(pid));

            let stdout_tx = tx.clone();
            let stderr_tx = tx.clone();
            let mut handles = Vec::new();

            if let Some(stdout) = child.stdout.take() {
                handles.push(thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line
                            && let Some(text) = utils::clean_log_line(&line)
                        {
                            let _ = stdout_tx.send(CommandUpdate::OutputLine(text));
                        }
                    }
                }));
            }

            if let Some(stderr) = child.stderr.take() {
                handles.push(thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line
                            && let Some(text) = utils::clean_log_line(&line)
                        {
                            let _ =
                                stderr_tx.send(CommandUpdate::OutputLine(format!("[ERR] {text}")));
                        }
                    }
                }));
            }

            for handle in handles {
                let _ = handle.join();
            }

            let exit_status = child.wait()?;
            log::info!("Maven process completed with status: {:?}", exit_status);

            if exit_status.success() {
                let _ = tx.send(CommandUpdate::Completed);
            } else {
                let _ = tx.send(CommandUpdate::Error(format!(
                    "Command failed with exit code: {:?}",
                    exit_status.code()
                )));
            }

            Ok(())
        })();

        if let Err(e) = result {
            log::error!("Command execution error: {}", e);
            let _ = tx.send(CommandUpdate::Error(format!("Execution error: {e}")));
        }
    });

    Ok(rx)
}

pub fn get_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error> {
    log::debug!(
        "get_profiles: Fetching Maven profiles from {:?}",
        project_root
    );
    // Try to load config and use settings if available
    let config = crate::config::load_config(project_root);
    // Run without -N flag to include profiles from all modules
    let output = execute_maven_command(
        project_root,
        None,
        &["help:all-profiles"],
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    // Use a HashSet to deduplicate profiles as they may appear multiple times
    // (once per module that inherits or defines them)
    let mut profile_set = std::collections::HashSet::new();

    for line in output.iter() {
        if line.contains("Profile Id:") {
            let parts: Vec<&str> = line.split("Profile Id:").collect();
            if parts.len() > 1 {
                // Extract just the profile name, stop at first space or parenthesis
                let profile_part = parts[1].trim();
                let profile_name = profile_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !profile_name.is_empty() {
                    log::debug!("Found profile: {}", profile_name);
                    profile_set.insert(profile_name.to_string());
                }
            }
        }
    }

    // Convert to sorted Vec for consistent ordering
    let mut profiles: Vec<String> = profile_set.into_iter().collect();
    profiles.sort();

    log::info!("Discovered {} unique Maven profiles", profiles.len());
    Ok(profiles)
}

/// Get profiles that are currently auto-activated by Maven
/// These are profiles activated by conditions like file existence, JDK version, OS, etc.
pub fn get_active_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error> {
    log::debug!(
        "get_active_profiles: Fetching auto-activated Maven profiles from {:?}",
        project_root
    );

    let config = crate::config::load_config(project_root);
    let output = execute_maven_command(
        project_root,
        None,
        &["help:active-profiles"],
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    let mut active_profiles = std::collections::HashSet::new();

    // Parse output looking for profile names after "- " lines
    for line in output.iter() {
        let trimmed = line.trim();
        // Lines with active profiles look like: " - dev (source: ...)"
        if let Some(stripped) = trimmed.strip_prefix("- ") {
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if let Some(profile_name) = parts.first() {
                log::debug!("Found active profile: {}", profile_name);
                active_profiles.insert(profile_name.to_string());
            }
        }
    }

    let mut profiles: Vec<String> = active_profiles.into_iter().collect();
    profiles.sort();

    log::info!("Discovered {} auto-activated profiles", profiles.len());
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn write_script(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
        // On Windows, batch files (.bat, .cmd) are executable by default
        // For tests, we create both the script and a .bat version
        #[cfg(windows)]
        {
            // Create a .bat file for Windows
            let bat_path = path.with_extension("bat");
            // Convert basic shell echo to batch echo
            let bat_content = content
                .replace("#!/bin/sh\n", "")
                .replace("echo $@", "echo %*");
            fs::write(&bat_path, bat_content).unwrap();
        }
    }

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn get_maven_command_returns_mvnw_if_present() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Test with mvnw present
        #[cfg(unix)]
        {
            let mvnw_path = project_root.join("mvnw");
            fs::File::create(&mvnw_path).unwrap();
            assert_eq!(get_maven_command(project_root), "./mvnw");
            std::fs::remove_file(&mvnw_path).unwrap();
        }

        #[cfg(windows)]
        {
            let mvnw_path = project_root.join("mvnw.bat");
            fs::File::create(&mvnw_path).unwrap();
            assert_eq!(get_maven_command(project_root), "mvnw.bat");
            std::fs::remove_file(&mvnw_path).unwrap();
        }

        // Test without mvnw present
        #[cfg(windows)]
        {
            assert_eq!(get_maven_command(project_root), "mvn.cmd");
        }
        #[cfg(not(windows))]
        {
            assert_eq!(get_maven_command(project_root), "mvn");
        }
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn execute_maven_command_captures_output() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho 'line 1'\necho 'line 2'\n");

        let output: Vec<String> =
            execute_maven_command(project_root, None, &["test"], &[], None, &[])
                .unwrap()
                .iter()
                .filter_map(|line| utils::clean_log_line(line))
                .collect();
        
        // Output now includes command line at the start
        // Skip the command line to check actual Maven output
        let maven_output: Vec<String> = output.iter().skip_while(|line| line.starts_with("$ ")).cloned().collect();
        assert_eq!(maven_output, vec!["line 1", "line 2"]);
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn execute_maven_command_captures_stderr() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(
            &mvnw_path,
            "#!/bin/sh\necho 'line 1'\n>&2 echo 'warn message'\n",
        );

        let output: Vec<String> =
            execute_maven_command(project_root, None, &["test"], &[], None, &[])
                .unwrap()
                .iter()
                .filter_map(|line| utils::clean_log_line(line))
                .collect();
        
        // Skip command line header
        let maven_output: Vec<String> = output.iter().skip_while(|line| line.starts_with("$ ")).cloned().collect();
        assert!(
            maven_output.contains(&"line 1".to_string()),
            "stdout line should be present"
        );
        assert!(
            maven_output.contains(&"[ERR] warn message".to_string()),
            "stderr line should be tagged"
        );
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn execute_maven_command_with_profiles() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let profiles = vec!["p1".to_string(), "p2".to_string()];
        let output: Vec<String> =
            execute_maven_command(project_root, None, &["test"], &profiles, None, &[])
                .unwrap()
                .iter()
                .filter_map(|line| utils::clean_log_line(line))
                .collect();
        
        // Skip command line header and check actual Maven output
        let maven_output: Vec<String> = output.iter().skip_while(|line| line.starts_with("$ ")).cloned().collect();
        assert_eq!(maven_output, vec!["-P p1,p2 test"]);
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn test_get_profiles() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script that simulates Maven's help:all-profiles output
        let mvnw_path = project_root.join("mvnw");
        write_script(
            &mvnw_path,
            "#!/bin/sh\necho '  Profile Id: profile-1 (Active: false, Source: pom)'\necho '  Profile Id: profile-2 (Active: true, Source: pom)'\n",
        );

        let profiles = get_profiles(project_root).unwrap();
        assert_eq!(profiles, vec!["profile-1", "profile-2"]);
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn test_get_profiles_deduplicates_and_sorts() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script that simulates Maven's help:all-profiles output
        // with duplicates (as would happen in multi-module projects without -N)
        let mvnw_path = project_root.join("mvnw");
        write_script(
            &mvnw_path,
            "#!/bin/sh\necho '  Profile Id: profile-2 (Active: false, Source: pom)'\necho '  Profile Id: profile-1 (Active: false, Source: pom)'\necho '  Profile Id: profile-2 (Active: false, Source: pom)'\necho '  Profile Id: child-profile (Active: false, Source: pom)'\n",
        );

        let profiles = get_profiles(project_root).unwrap();
        // Should be deduplicated and sorted
        assert_eq!(profiles, vec!["child-profile", "profile-1", "profile-2"]);
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn execute_maven_command_scopes_to_module() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let output: Vec<String> =
            execute_maven_command(project_root, Some("module-a"), &["test"], &[], None, &[])
                .unwrap()
                .iter()
                .filter_map(|line| utils::clean_log_line(line))
                .collect();
        
        // Skip command line header
        let maven_output: Vec<String> = output.iter().skip_while(|line| line.starts_with("$ ")).cloned().collect();
        assert_eq!(maven_output, vec!["-pl module-a test"]);
    }

    #[test]
    #[cfg(unix)] // Shell script execution not supported on Windows
    fn execute_maven_command_without_pl_for_root_module() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let output: Vec<String> =
            execute_maven_command(project_root, Some("."), &["test"], &[], None, &[])
                .unwrap()
                .iter()
                .filter_map(|line| utils::clean_log_line(line))
                .collect();
        
        // Skip command line header
        let maven_output: Vec<String> = output.iter().skip_while(|line| line.starts_with("$ ")).cloned().collect();
        assert_eq!(maven_output, vec!["test"]);
    }

    #[test]
    #[cfg(unix)]
    fn test_command_display_in_output() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho 'test output'\n");

        let profiles = vec!["dev".to_string()];
        let flags = vec!["--offline".to_string()];
        let output = execute_maven_command(
            project_root,
            Some("my-module"),
            &["clean", "install"],
            &profiles,
            None,
            &flags,
        )
        .unwrap();

        // First line should be the command
        assert!(
            output[0].starts_with("$ ./mvnw"),
            "First line should be the command: {}",
            output[0]
        );
        assert!(
            output[0].contains("-P dev"),
            "Command should include profiles: {}",
            output[0]
        );
        assert!(
            output[0].contains("-pl my-module"),
            "Command should include module: {}",
            output[0]
        );
        assert!(
            output[0].contains("--offline"),
            "Command should include flags: {}",
            output[0]
        );
        assert!(
            output[0].contains("clean install"),
            "Command should include goals: {}",
            output[0]
        );
    }
}
