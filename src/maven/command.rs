//! Maven command building and execution

use crate::config::LoggingConfig;
use crate::utils;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

use super::process::CommandUpdate;

/// Extract logging overrides from config
fn get_logging_overrides(logging_config: Option<&LoggingConfig>) -> Vec<(String, String)> {
    logging_config
        .map(|config| {
            config
                .packages
                .iter()
                .map(|pkg| (pkg.name.clone(), pkg.level.clone()))
                .collect()
        })
        .unwrap_or_default()
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

/// Build Maven command string for display purposes
#[allow(dead_code)]
pub fn build_command_string(
    maven_command: &str,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> String {
    build_command_string_with_options(
        maven_command,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false,
        Path::new("."),
        &[], // No logging overrides for backward compatibility
    )
}

/// Build the full command string for display with option to use -f
#[allow(clippy::too_many_arguments)]
pub fn build_command_string_with_options(
    maven_command: &str,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    project_root: &Path,
    _logging_overrides: &[(String, String)],
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

    if let Some(module) = module
        && module != "."
    {
        if use_file_flag {
            let module_pom = project_root.join(module).join("pom.xml");
            parts.push("-f".to_string());
            parts.push(module_pom.to_string_lossy().to_string());

            // Auto-add --also-make for exec:java to ensure dependencies are built
            if args.contains(&"exec:java") && !flags.iter().any(|f| f.contains("also-make")) {
                parts.push("--also-make".to_string());
            }
        } else {
            parts.push("-pl".to_string());
            parts.push(module.to_string());

            // Note: We don't auto-add --also-make for spring-boot:run because it would
            // try to execute the goal on all modules in the reactor (including parent POM).
        }
    }

    for flag in flags {
        parts.push(flag.to_string());
    }

    // Note: logging_overrides are handled by the caller and included in args
    // (either as spring-boot.run.jvmArguments or exec.args)
    // We don't add them here to avoid duplication

    for arg in args {
        parts.push(arg.to_string());
    }

    parts.join(" ")
}

/// Kill a running process by PID
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
    execute_maven_command_with_options(
        project_root,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false, // use_file_flag = false for backward compatibility
        None,  // No logging config for backward compatibility
    )
}

/// Execute Maven command with option to use -f instead of -pl
#[allow(clippy::too_many_arguments)]
pub fn execute_maven_command_with_options(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    logging_config: Option<&LoggingConfig>,
) -> Result<Vec<String>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    log::debug!("execute_maven_command: Using command: {}", maven_command);
    log::debug!("  project_root: {:?}", project_root);
    log::debug!("  module: {:?}", module);
    log::debug!("  args: {:?}", args);
    log::debug!("  profiles: {:?}", profiles);
    log::debug!("  settings_path: {:?}", settings_path);
    log::debug!("  flags: {:?}", flags);
    log::debug!("  use_file_flag: {}", use_file_flag);

    let logging_overrides = get_logging_overrides(logging_config);
    if !logging_overrides.is_empty() {
        log::debug!("  logging_overrides: {:?}", logging_overrides);
    }

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
            if use_file_flag {
                // Use -f to target the module's POM directly
                let module_pom = project_root.join(module).join("pom.xml");
                command.arg("-f").arg(&module_pom);
                log::debug!("Using -f flag with POM: {:?}", module_pom);

                // Auto-add --also-make for exec:java to ensure dependencies are built
                if args.contains(&"exec:java") && !flags.iter().any(|f| f.contains("also-make")) {
                    command.arg("--also-make");
                    log::debug!("Auto-adding --also-make for exec:java with -f flag");
                }
            } else {
                // Use -pl for reactor build
                command.arg("-pl").arg(module);
                log::debug!("Scoped to module: {}", module);

                // Note: We don't auto-add --also-make for spring-boot:run because it would
                // try to execute the goal on all modules in the reactor (including parent POM)
                // which fails. Dependencies should be built beforehand with a separate build command.
            }
        } else {
            log::debug!("Running on project root, no -pl/-f flag needed");
        }
    }
    // Add build flags
    for flag in flags {
        command.arg(flag);
        log::debug!("Added flag: {}", flag);
    }

    // Add logging overrides
    for (package, level) in &logging_overrides {
        command.arg(format!("-Dlog4j.logger.{}={}", package, level));
        command.arg(format!("-Dlogging.level.{}={}", package, level));
        log::debug!("Added logging override: {} = {}", package, level);
    }

    // Build the full command string for display
    let command_str = build_command_string_with_options(
        &maven_command,
        module,
        args,
        profiles,
        settings_path,
        flags,
        use_file_flag,
        project_root,
        &logging_overrides,
    );
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
#[allow(dead_code)]
pub fn execute_maven_command_async(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> Result<mpsc::Receiver<CommandUpdate>, std::io::Error> {
    execute_maven_command_async_with_options(
        project_root,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false, // use_file_flag = false for backward compatibility
        None,  // No logging config for backward compatibility
    )
}

/// Async version with option to use -f instead of -pl
#[allow(clippy::too_many_arguments)]
pub fn execute_maven_command_async_with_options(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    logging_config: Option<&LoggingConfig>,
) -> Result<mpsc::Receiver<CommandUpdate>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    log::debug!(
        "execute_maven_command_async: Using command: {}",
        maven_command
    );
    log::debug!("  project_root: {:?}", project_root);
    log::debug!("  module: {:?}", module);
    log::debug!("  args: {:?}", args);
    log::debug!("  use_file_flag: {}", use_file_flag);

    let logging_overrides = get_logging_overrides(logging_config);
    if !logging_overrides.is_empty() {
        log::debug!("  logging_overrides: {:?}", logging_overrides);
    }

    // Build the full command string for display
    let command_str = build_command_string_with_options(
        &maven_command,
        module,
        args,
        profiles,
        settings_path,
        flags,
        use_file_flag,
        project_root,
        &logging_overrides,
    );
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
        if use_file_flag {
            // Use -f to target the module's POM directly
            let module_pom = project_root.join(module).join("pom.xml");
            command.arg("-f").arg(&module_pom);
            log::debug!("Using -f flag with POM: {:?}", module_pom);

            // Auto-add --also-make for exec:java to ensure dependencies are built
            if args.contains(&"exec:java") && !flags.iter().any(|f| f.contains("also-make")) {
                command.arg("--also-make");
                log::debug!("Auto-adding --also-make for exec:java with -f flag");
            }
        } else {
            // Use -pl for reactor build
            command.arg("-pl").arg(module);

            // Note: We don't auto-add --also-make for spring-boot:run because it would
            // try to execute the goal on all modules in the reactor (including parent POM).
        }
    }
    for flag in flags {
        command.arg(flag);
    }

    // Add logging overrides
    // Note: For spring-boot:run, logging overrides are already included in
    // -Dspring-boot.run.jvmArguments= (see build_launch_command in detection.rs).
    // We only add them as Maven system properties for other launch strategies.
    let has_spring_boot_jvm_args = args.iter().any(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments="));
    if !has_spring_boot_jvm_args {
        for (package, level) in &logging_overrides {
            command.arg(format!("-Dlog4j.logger.{}={}", package, level));
            command.arg(format!("-Dlogging.level.{}={}", package, level));
            log::debug!("Added logging override: {} = {}", package, level);
        }
    } else {
        log::debug!("Skipping logging overrides as Maven properties (already in spring-boot.run.jvmArguments)");
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

            // Configure command to create a new process group
            // This allows us to kill all child processes (like Spring Boot) when needed
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0); // Create new process group
            }

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
