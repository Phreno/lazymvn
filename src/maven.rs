use crate::utils;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

pub fn get_maven_command(project_root: &Path) -> String {
    if project_root.join("mvnw").exists() {
        "./mvnw".to_string()
    } else {
        "mvn".to_string()
    }
}

pub fn execute_maven_command(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
) -> Result<Vec<String>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    let mut command = Command::new(maven_command);
    if let Some(settings_path) = settings_path {
        command.arg("--settings").arg(settings_path);
    }
    if !profiles.is_empty() {
        command.arg("-P").arg(profiles.join(","));
    }
    if let Some(module) = module {
        command.arg("-pl").arg(module);
    }
    let mut child = command
        .args(args)
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut output = Vec::new();
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(text) = utils::clean_log_line(&line) {
                        let _ = tx.send(text);
                    }
                }
            }
        }));
    }

    if let Some(stderr) = child.stderr.take() {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(text) = utils::clean_log_line(&line) {
                        let _ = tx.send(format!("[ERR] {text}"));
                    }
                }
            }
        }));
    }

    drop(tx);

    for line in rx {
        output.push(line);
    }

    for handle in handles {
        let _ = handle.join();
    }

    child.wait()?;
    Ok(output)
}

pub fn get_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error> {
    let output = execute_maven_command(project_root, None, &["help:all-profiles", "-N"], &[], None)?;
    let profiles = output
        .iter()
        .filter_map(|line| {
            if line.contains("Profile Id:") {
                line.split("Profile Id:")
                    .last()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn write_script(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
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
        let mvnw_path = project_root.join("mvnw");
        fs::File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "./mvnw");

        // Test without mvnw present
        std::fs::remove_file(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "mvn");
    }

    #[test]
    fn execute_maven_command_captures_output() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
                    write_script(&mvnw_path, "#!/bin/sh\necho 'line 1'\necho 'line 2'\n");
        
                            let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &[], None)
        
                                .unwrap()
        
                                .iter()
        
                                .map(|line| utils::clean_log_line(line).unwrap())
        
                                .collect();        assert_eq!(output, vec!["line 1", "line 2"]);
    }

    #[test]
    fn execute_maven_command_captures_stderr() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(
            &mvnw_path,
            "#!/bin/sh\necho 'line 1'\n>&2 echo 'warn message'\n",
        );

        let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &[], None)
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
    fn execute_maven_command_with_profiles() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let profiles = vec!["p1".to_string(), "p2".to_string()];
        let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &profiles, None)
            .unwrap()
            .iter()
            .map(|line| utils::clean_log_line(line).unwrap())
            .collect();
        assert_eq!(output, vec!["-P p1,p2 test"]);
    }

    #[test]
    fn test_get_profiles() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        write_script(
            &mvnw_path,
            "#!/bin/sh\necho '[INFO]   Profile Id: profile-1'\necho '[INFO]   Profile Id: profile-2'\n",
        );

        let profiles: Vec<String> = get_profiles(project_root)
            .unwrap()
            .iter()
            .map(|line| utils::clean_log_line(line).unwrap())
            .collect();
        assert_eq!(profiles, vec!["profile-1", "profile-2"]);
    }

    #[test]
    fn execute_maven_command_scopes_to_module() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let output: Vec<String> = execute_maven_command(project_root, Some("module-a"), &["test"], &[], None)
            .unwrap()
            .iter()
            .map(|line| utils::clean_log_line(line).unwrap())
            .collect();
        assert_eq!(output, vec!["-pl module-a test"]);
    }
}
