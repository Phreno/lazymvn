//! Maven command execution utilities

use crate::core::config::LoggingConfig;
use crate::maven::process::CommandUpdate;
use maven_java_agent::AgentBuilder;
use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

use super::builder::get_maven_command;
use super::helpers::{filter_spring_boot_incompatible_flags, is_spring_boot_run_command, parse_flag_parts};
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
        // Use the new maven-java-agent library to configure environment
        match AgentBuilder::new()
            .with_log4j_config(&log4j_config_url)
            .build()
        {
            Ok(deployment) => {
                // Set environment variables from the deployment
                for (key, value) in deployment.env_vars {
                    log::info!("Setting {}: {}", key, value);
                    command.env(key, value);
                }
            }
            Err(e) => {
                // Fallback to manual configuration if agent setup fails
                log::warn!("Failed to use maven-java-agent library: {}, using fallback", e);
                let opts_str = format!(
                    "-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration={}",
                    log4j_config_url
                );
                log::info!("Setting JAVA_TOOL_OPTIONS with Log4j configuration: {}", log4j_config_url);
                log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
                command.env("JAVA_TOOL_OPTIONS", &opts_str);
            }
        }
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
    let is_spring_boot = is_spring_boot_run_command(args);
    
    let filtered_flags_vec = if is_spring_boot {
        let original_count = flags.len();
        let filtered = filter_spring_boot_incompatible_flags(flags);
        
        if filtered.len() < original_count {
            log::warn!(
                "Filtered out --also-make flags for spring-boot:run (would execute on all reactor modules including parent POM)"
            );
            log::debug!("Original flags: {:?}", flags);
            log::debug!("Filtered flags: {:?}", filtered);
        }
        filtered
    } else {
        flags.to_vec()
    };
    
    // Add build flags (split on spaces if needed, skip commas and aliases)
    for flag in &filtered_flags_vec {
        // Split flags like "-U, --update-snapshots" into individual flags
        // Take only the first part before comma to skip aliases
        let flag_parts = parse_flag_parts(flag);
        
        for part in &flag_parts {
            command.arg(part);
            log::debug!("Added flag: {}", part);
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
    let command_display = build_command_display(
        &maven_command,
        module,
        profiles,
        settings_path,
        &filtered_flags_vec,
        args,
        use_file_flag,
    );

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
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let is_spring_boot = is_spring_boot_run_command(&args_refs);

        let filtered_flags_vec = if is_spring_boot {
            let original_count = flags.len();
            let filtered = filter_spring_boot_incompatible_flags(&flags);

            if filtered.len() < original_count {
                log::warn!(
                    "Filtered out --also-make flags for spring-boot:run (would execute on all reactor modules)"
                );
            }
            filtered
        } else {
            flags.clone()
        };

        // Add build flags (split on spaces if needed, skip commas and aliases)
        for flag in &filtered_flags_vec {
            let flag_parts = parse_flag_parts(flag);

            for part in &flag_parts {
                command.arg(part);
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
            read_lines_lossy(stdout, tx_clone, "STDOUT");
        });

        let tx_clone = tx.clone();
        let stderr_handle = thread::spawn(move || {
            read_lines_lossy(stderr, tx_clone, "STDERR");
        });

        log::debug!("Waiting for output threads to complete...");
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        log::debug!("Output threads completed");

        log::debug!("Waiting for Maven process to exit...");
        match child.wait() {
            Ok(status) => {
                log::info!("Maven process exited with status: {}", status);
                if status.success() {
                    let _ = tx.send(CommandUpdate::Completed);
                } else {
                    let error_msg = if let Some(code) = status.code() {
                        format!("Build failed with exit code {}", code)
                    } else {
                        "Build failed (terminated by signal)".to_string()
                    };
                    let _ = tx.send(CommandUpdate::Error(error_msg));
                }
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

/// Build command display string for UI
pub fn build_command_display(
    maven_command: &str,
    module: Option<&str>,
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    args: &[&str],
    use_file_flag: bool,
) -> String {
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
    
    for flag in flags {
        command_display.push_str(&format!(" {}", flag));
    }
    
    for arg in args {
        command_display.push_str(&format!(" {}", arg));
    }
    
    command_display
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_display_basic() {
        let result = build_command_display(
            "mvn",
            None,
            &[],
            None,
            &[],
            &["clean", "install"],
            false,
        );
        assert_eq!(result, "$ mvn clean install");
    }

    #[test]
    fn test_build_command_display_with_module() {
        let result = build_command_display(
            "mvn",
            Some("my-module"),
            &[],
            None,
            &[],
            &["clean"],
            false,
        );
        assert_eq!(result, "$ mvn -pl my-module clean");
    }

    #[test]
    fn test_build_command_display_with_module_using_file_flag() {
        let result = build_command_display(
            "mvn",
            Some("my-module"),
            &[],
            None,
            &[],
            &["exec:java"],
            true,
        );
        assert_eq!(result, "$ mvn -f my-module/pom.xml exec:java");
    }

    #[test]
    fn test_build_command_display_with_profiles() {
        let result = build_command_display(
            "mvn",
            None,
            &["dev".to_string(), "local".to_string()],
            None,
            &[],
            &["test"],
            false,
        );
        assert_eq!(result, "$ mvn -P dev,local test");
    }

    #[test]
    fn test_build_command_display_with_settings() {
        let result = build_command_display(
            "mvn",
            None,
            &[],
            Some("settings.xml"),
            &[],
            &["verify"],
            false,
        );
        assert_eq!(result, "$ mvn -s settings.xml verify");
    }

    #[test]
    fn test_build_command_display_with_flags() {
        let result = build_command_display(
            "mvn",
            None,
            &[],
            None,
            &["-U".to_string(), "--quiet".to_string()],
            &["package"],
            false,
        );
        assert_eq!(result, "$ mvn -U --quiet package");
    }

    #[test]
    fn test_build_command_display_complete() {
        let result = build_command_display(
            "./mvnw",
            Some("backend"),
            &["production".to_string()],
            Some("/etc/maven/settings.xml"),
            &["-DskipTests".to_string(), "--batch-mode".to_string()],
            &["clean", "deploy"],
            false,
        );
        assert_eq!(
            result,
            "$ ./mvnw -pl backend -P production -s /etc/maven/settings.xml -DskipTests --batch-mode clean deploy"
        );
    }

    #[test]
    fn test_build_command_display_root_module_dot() {
        let result = build_command_display(
            "mvn",
            Some("."),
            &[],
            None,
            &[],
            &["install"],
            false,
        );
        // Root module (.) should not add -pl
        assert_eq!(result, "$ mvn install");
    }
}

/// Helper function to read lines from a stream with UTF-8 lossy conversion
/// This ensures that non-UTF-8 characters (common in Maven output from Windows)
/// don't crash the reader thread
fn read_lines_lossy<R: Read>(
    reader: R,
    tx: mpsc::Sender<CommandUpdate>,
    stream_name: &str,
) {
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    
    loop {
        buffer.clear();
        
        // Read until newline or EOF
        match buf_reader.read_until(b'\n', &mut buffer) {
            Ok(0) => {
                // EOF reached
                break;
            }
            Ok(_) => {
                // Convert bytes to string with lossy UTF-8 conversion
                // This replaces invalid UTF-8 sequences with �
                let line = String::from_utf8_lossy(&buffer);
                let line = line.trim_end_matches('\n').trim_end_matches('\r');
                
                log::trace!("[{}] {}", stream_name, line);
                
                if tx.send(CommandUpdate::OutputLine(line.to_string())).is_err() {
                    log::warn!("Failed to send {} line (receiver closed)", stream_name);
                    break;
                }
            }
            Err(e) => {
                log::error!("Error reading {}: {}", stream_name, e);
                break;
            }
        }
    }
    
    log::debug!("{} reader thread finished", stream_name);
}
