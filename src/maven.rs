use crate::utils;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

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

    "mvn".to_string()
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

    let mut command = Command::new(maven_command);
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

    log::info!("Spawning Maven process...");
    let mut child = command
        .args(args)
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    log::debug!("Maven process spawned with PID: {:?}", child.id());
    let mut output = Vec::new();
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
        assert_eq!(get_maven_command(project_root), "mvn");
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
                .map(|line| utils::clean_log_line(line).unwrap())
                .collect();
        assert_eq!(output, vec!["line 1", "line 2"]);
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
                .map(|line| utils::clean_log_line(line).unwrap())
                .collect();
        assert!(
            output.contains(&"line 1".to_string()),
            "stdout line should be present"
        );
        assert!(
            output.contains(&"[ERR] warn message".to_string()),
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
                .map(|line| utils::clean_log_line(line).unwrap())
                .collect();
        assert_eq!(output, vec!["-P p1,p2 test"]);
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
                .map(|line| utils::clean_log_line(line).unwrap())
                .collect();
        assert_eq!(output, vec!["-pl module-a test"]);
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
                .map(|line| utils::clean_log_line(line).unwrap())
                .collect();
        assert_eq!(output, vec!["test"]);
    }
}
