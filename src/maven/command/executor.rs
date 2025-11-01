//! Maven command execution utilities

use crate::core::config::LoggingConfig;
use crate::maven::process::CommandUpdate;
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

use super::builder::get_maven_command;
use super::log4j_config::{extract_log4j_config_url, get_logging_overrides};

/// Execute Maven command synchronously
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
    
    // CRITICAL: Set JAVA_TOOL_OPTIONS environment variable to inject Log4j configuration
    // This ensures Log4j properties are set BEFORE any application code runs
    // (including custom factories like Log4jJbossLoggerFactory that initialize in constructors)
    // Check for Log4j config URL in args (always present for Spring Boot, regardless of logging_config)
    if let Some(log4j_config_url) = extract_log4j_config_url(args) {
        let opts_str = format!(
            "-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration={}",
            log4j_config_url
        );
        log::info!("Setting JAVA_TOOL_OPTIONS with Log4j configuration: {}", log4j_config_url);
        log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
        command.env("JAVA_TOOL_OPTIONS", &opts_str);
    } else {
        log::debug!("No Log4j configuration URL found in args");
    }
    
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
    
    // Filter incompatible flags for spring-boot:run
    // --also-make and --also-make-dependents cause spring-boot:run to execute on ALL modules
    // in the reactor (including parent POMs), which fails with "Unable to find main class"
    let is_spring_boot_run = args.iter().any(|arg| {
        arg.contains("spring-boot:run") || arg.contains("spring-boot-maven-plugin") && arg.contains(":run")
    });
    
    let filtered_flags: Vec<&String> = if is_spring_boot_run {
        let original_count = flags.len();
        let filtered: Vec<&String> = flags.iter()
            .filter(|flag| {
                let flag_lower = flag.to_lowercase();
                !flag_lower.contains("also-make")
            })
            .collect();
        
        if filtered.len() < original_count {
            log::warn!(
                "Filtered out --also-make flags for spring-boot:run (would execute on all reactor modules including parent POM)"
            );
            log::debug!("Original flags: {:?}", flags);
            log::debug!("Filtered flags: {:?}", filtered);
        }
        filtered
    } else {
        flags.iter().collect()
    };
    
    // Add build flags (split on spaces if needed, skip commas and aliases)
    for flag in &filtered_flags {
        // Split flags like "-U, --update-snapshots" into individual flags
        // Take only the first part before comma to skip aliases
        let flag_parts: Vec<&str> = flag
            .split(',')
            .next()
            .unwrap_or(flag)
            .split_whitespace()
            .collect();
        
        for part in flag_parts {
            if !part.is_empty() {
                command.arg(part);
                log::debug!("Added flag: {}", part);
            }
        }
    }

    // Add logging config via Maven properties if logging_config is provided
    // and no JVM args are already present
    let has_spring_boot_jvm_args = args
        .iter()
        .any(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments=") || arg.starts_with("-Drun.jvmArguments="));

    if let Some(logging_config) = logging_config
        && !has_spring_boot_jvm_args
    {
        if let Some(log_format) = &logging_config.log_format {
            command.arg(format!("-Dlog4j.conversionPattern={}", log_format));
            command.arg(format!("-Dlogging.pattern.console={}", log_format));
            log::debug!("Added log format: {}", log_format);
        }
        for (package, level) in &logging_overrides {
            command.arg(format!("-Dlog4j.logger.{}={}", package, level));
            command.arg(format!("-Dlogging.level.{}={}", package, level));
            log::debug!("Added logging override: {}={}", package, level);
        }
    }

    // Add the actual Maven goal/command arguments
    for arg in args {
        command.arg(arg);
        log::debug!("Added arg: {}", arg);
    }

    log::debug!("Final command: {:?}", command);

    // Format command display string
    let mut command_display = format!("$ {}", maven_command);
    if let Some(m) = module
        && m != "."
    {
        if use_file_flag {
            command_display.push_str(&format!(" -f {}/pom.xml", m));
        } else {
            command_display.push_str(&format!(" -pl {}", m));
        }
    }
    if !profiles.is_empty() {
        command_display.push_str(&format!(" -P {}", profiles.join(",")));
    }
    if let Some(s) = settings_path {
        command_display.push_str(&format!(" -s {}", s));
    }
    for flag in filtered_flags {
        command_display.push_str(&format!(" {}", flag));
    }
    for arg in args {
        command_display.push_str(&format!(" {}", arg));
    }

    let output = command.current_dir(project_root).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut lines: Vec<String> = vec![command_display];
    lines.extend(stdout.lines().map(|s| s.to_string()));
    lines.extend(stderr.lines().map(|s| format!("[ERR] {}", s)));

    log::debug!("Command completed with status: {}", output.status);
    log::debug!("Output lines: {}", lines.len());

    if !output.status.success() {
        log::warn!(
            "Maven command failed with exit code: {:?}",
            output.status.code()
        );
    }

    Ok(lines)
}

/// Execute Maven command asynchronously with default options
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

/// Execute Maven command asynchronously with streaming output and option to use -f
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
    let (tx, rx) = mpsc::channel();
    let maven_command = get_maven_command(project_root);
    let project_root = project_root.to_path_buf();
    let module = module.map(|s| s.to_string());
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let profiles = profiles.to_vec();
    let settings_path = settings_path.map(|s| s.to_string());
    let flags = flags.to_vec();
    let logging_config = logging_config.cloned();

    thread::spawn(move || {
        log::info!("Starting Maven command in background thread");
        log::debug!("  command: {}", maven_command);
        log::debug!("  project_root: {:?}", project_root);
        log::debug!("  module: {:?}", module);
        log::debug!("  args: {:?}", args);
        log::debug!("  profiles: {:?}", profiles);
        log::debug!("  settings_path: {:?}", settings_path);
        log::debug!("  flags: {:?}", flags);
        log::debug!("  use_file_flag: {}", use_file_flag);

        let logging_overrides = get_logging_overrides(logging_config.as_ref());
        if !logging_overrides.is_empty() {
            log::debug!("  logging_overrides: {:?}", logging_overrides);
        }

        let mut command = Command::new(&maven_command);

        // CRITICAL: Set JAVA_TOOL_OPTIONS environment variable to inject Log4j configuration
        // This ensures Log4j properties are set BEFORE any application code runs
        if let Some(log4j_config_url) = extract_log4j_config_url(
            &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        ) {
            let opts_str = format!(
                "-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration={}",
                log4j_config_url
            );
            log::info!(
                "Setting JAVA_TOOL_OPTIONS with Log4j configuration: {}",
                log4j_config_url
            );
            log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
            command.env("JAVA_TOOL_OPTIONS", &opts_str);
        } else {
            log::debug!("No Log4j configuration URL found in args");
        }

        if let Some(ref settings_path) = settings_path {
            command.arg("--settings").arg(settings_path);
        }
        if !profiles.is_empty() {
            let profile_str = profiles.join(",");
            command.arg("-P").arg(&profile_str);
        }
        if let Some(ref module) = module {
            // Only use -pl flag if module is not "." (project root)
            if module != "." {
                if use_file_flag {
                    // Use -f to target the module's POM directly
                    let module_pom = project_root.join(module).join("pom.xml");
                    command.arg("-f").arg(&module_pom);
                    log::debug!("Using -f flag with POM: {:?}", module_pom);

                    // Auto-add --also-make for exec:java to ensure dependencies are built
                    if args.contains(&"exec:java".to_string())
                        && !flags.iter().any(|f| f.contains("also-make"))
                    {
                        command.arg("--also-make");
                        log::debug!("Auto-adding --also-make for exec:java with -f flag");
                    }
                } else {
                    // Use -pl for reactor build
                    command.arg("-pl").arg(module);
                    log::debug!("Scoped to module: {}", module);
                }
            }
        }

        // Filter incompatible flags for spring-boot:run
        let is_spring_boot_run = args.iter().any(|arg| {
            arg.contains("spring-boot:run")
                || arg.contains("spring-boot-maven-plugin") && arg.contains(":run")
        });

        let filtered_flags: Vec<&String> = if is_spring_boot_run {
            let original_count = flags.len();
            let filtered: Vec<&String> = flags
                .iter()
                .filter(|flag| {
                    let flag_lower = flag.to_lowercase();
                    !flag_lower.contains("also-make")
                })
                .collect();

            if filtered.len() < original_count {
                log::warn!(
                    "Filtered out --also-make flags for spring-boot:run (would execute on all reactor modules)"
                );
            }
            filtered
        } else {
            flags.iter().collect()
        };

        // Add build flags (split on spaces if needed, skip commas and aliases)
        for flag in filtered_flags {
            let flag_parts: Vec<&str> = flag
                .split(',')
                .next()
                .unwrap_or(flag)
                .split_whitespace()
                .collect();

            for part in flag_parts {
                if !part.is_empty() {
                    command.arg(part);
                }
            }
        }

        // Add logging config via Maven properties if logging_config is provided
        // and no JVM args are already present
        let has_spring_boot_jvm_args = args.iter().any(|arg| {
            arg.starts_with("-Dspring-boot.run.jvmArguments=")
                || arg.starts_with("-Drun.jvmArguments=")
        });

        if let Some(ref logging_config) = logging_config
            && !has_spring_boot_jvm_args
        {
            if let Some(log_format) = &logging_config.log_format {
                command.arg(format!("-Dlog4j.conversionPattern={}", log_format));
                command.arg(format!("-Dlogging.pattern.console={}", log_format));
            }
            for (package, level) in &logging_overrides {
                command.arg(format!("-Dlog4j.logger.{}={}", package, level));
                command.arg(format!("-Dlogging.level.{}={}", package, level));
            }
        }

        // Add the actual Maven goal/command arguments
        for arg in &args {
            command.arg(arg);
        }

        command
            .current_dir(&project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log::debug!("Spawning Maven process...");
        let mut child = match command.spawn() {
            Ok(child) => {
                log::info!("Maven process spawned successfully (PID: {:?})", child.id());
                child
            }
            Err(e) => {
                log::error!("Failed to spawn Maven process: {}", e);
                let _ = tx.send(CommandUpdate::Error(format!("Failed to start: {}", e)));
                return;
            }
        };

        let child_id = child.id();
        log::debug!("Maven child process ID: {}", child_id);

        let _ = tx.send(CommandUpdate::Started(child_id));

        let stdout = child.stdout.take().expect("Failed to get stdout");
        let stderr = child.stderr.take().expect("Failed to get stderr");

        let tx_clone = tx.clone();
        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        log::trace!("[STDOUT] {}", line);
                        if tx_clone.send(CommandUpdate::OutputLine(line)).is_err() {
                            log::warn!("Failed to send stdout line (receiver closed)");
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            log::debug!("Stdout reader thread finished");
        });

        let tx_clone = tx.clone();
        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        log::trace!("[STDERR] {}", line);
                        if tx_clone.send(CommandUpdate::OutputLine(line)).is_err() {
                            log::warn!("Failed to send stderr line (receiver closed)");
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading stderr: {}", e);
                        break;
                    }
                }
            }
            log::debug!("Stderr reader thread finished");
        });

        log::debug!("Waiting for output threads to complete...");
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        log::debug!("Output threads completed");

        log::debug!("Waiting for Maven process to exit...");
        match child.wait() {
            Ok(status) => {
                log::info!("Maven process exited with status: {}", status);
                let _ = tx.send(CommandUpdate::Completed);
            }
            Err(e) => {
                log::error!("Error waiting for Maven process: {}", e);
                let _ = tx.send(CommandUpdate::Error(format!("Process error: {}", e)));
            }
        }
        log::info!("Maven command thread finished");
    });

    Ok(rx)
}
