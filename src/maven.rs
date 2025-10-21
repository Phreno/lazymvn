use crate::config::LaunchMode;
use crate::utils;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
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

/// Information about a module's Spring Boot capabilities
#[derive(Debug, Clone)]
pub struct SpringBootDetection {
    pub has_spring_boot_plugin: bool,
    pub has_exec_plugin: bool,
    pub main_class: Option<String>,
    pub packaging: Option<String>,
}

impl SpringBootDetection {
    /// Check if spring-boot:run should work
    pub fn can_use_spring_boot_run(&self) -> bool {
        self.has_spring_boot_plugin
            && self
                .packaging
                .as_ref()
                .map(|p| p == "jar" || p == "war")
                .unwrap_or(true)
    }

    /// Check if this looks like a Spring Boot web application that should prefer spring-boot:run
    pub fn should_prefer_spring_boot_run(&self) -> bool {
        // For war packaging with Spring Boot plugin, prefer spring-boot:run
        // to avoid servlet classpath issues with exec:java
        self.has_spring_boot_plugin && 
        self.packaging.as_ref().map(|p| p == "war").unwrap_or(false)
    }

    /// Check if exec:java can be used as fallback
    pub fn can_use_exec_java(&self) -> bool {
        self.has_exec_plugin || self.main_class.is_some()
    }
}

/// Decide which launch strategy to use
pub fn decide_launch_strategy(
    detection: &SpringBootDetection,
    launch_mode: LaunchMode,
) -> LaunchStrategy {
    match launch_mode {
        LaunchMode::ForceRun => LaunchStrategy::SpringBootRun,
        LaunchMode::ForceExec => LaunchStrategy::ExecJava,
        LaunchMode::Auto => {
            if detection.should_prefer_spring_boot_run() {
                log::info!("Auto mode: Spring Boot web app detected (war packaging), strongly preferring spring-boot:run");
                LaunchStrategy::SpringBootRun
            } else if detection.can_use_spring_boot_run() {
                log::info!("Auto mode: Spring Boot plugin detected, using spring-boot:run");
                LaunchStrategy::SpringBootRun
            } else if detection.can_use_exec_java() {
                log::info!(
                    "Auto mode: No Spring Boot plugin or incompatible packaging, using exec:java"
                );
                LaunchStrategy::ExecJava
            } else {
                log::warn!(
                    "Auto mode: No viable launch strategy detected, defaulting to spring-boot:run"
                );
                LaunchStrategy::SpringBootRun
            }
        }
    }
}

/// Launch strategy for running applications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStrategy {
    SpringBootRun,
    ExecJava,
    VSCodeJava, // Use VS Code Java extension to launch
}

/// Build launch command based on detection and strategy
pub fn build_launch_command(
    strategy: LaunchStrategy,
    main_class: Option<&str>,
    profiles: &[String],
    jvm_args: &[String],
) -> Vec<String> {
    let mut command_parts = Vec::new();

    match strategy {
        LaunchStrategy::SpringBootRun => {
            // Build spring-boot:run command with parameters
            if !profiles.is_empty() {
                // Pass profiles as spring-boot.run.profiles
                let profiles_arg = format!("-Dspring-boot.run.profiles={}", profiles.join(","));
                command_parts.push(quote_arg_for_platform(&profiles_arg));
            }

            if !jvm_args.is_empty() {
                // Pass JVM args as spring-boot.run.jvmArguments
                let jvm_args_str = jvm_args.join(" ");
                let jvm_arg = format!("-Dspring-boot.run.jvmArguments={}", jvm_args_str);
                command_parts.push(quote_arg_for_platform(&jvm_arg));
            }

            command_parts.push("spring-boot:run".to_string());

            log::info!(
                "Built spring-boot:run command with {} profile(s) and {} JVM arg(s)",
                profiles.len(),
                jvm_args.len()
            );
        }
        LaunchStrategy::ExecJava => {
            // Build exec:java command with mainClass
            if let Some(mc) = main_class {
                let main_class_arg = format!("-Dexec.mainClass={}", mc);
                command_parts.push(quote_arg_for_platform(&main_class_arg));
            }

            // Add JVM args as system properties
            for arg in jvm_args {
                command_parts.push(quote_arg_for_platform(arg));
            }

            command_parts.push("exec:java".to_string());

            log::info!(
                "Built exec:java command with mainClass={:?} and {} JVM arg(s)",
                main_class,
                jvm_args.len()
            );
        }
        LaunchStrategy::VSCodeJava => {
            // This is a placeholder - actual VS Code integration would be different
            command_parts.push("# VS Code Java launch not implemented yet".to_string());
            log::info!("VS Code Java launch strategy selected (not implemented)");
        }
    }

    command_parts
}

/// Quote arguments appropriately for the platform (especially PowerShell on Windows)
fn quote_arg_for_platform(arg: &str) -> String {
    #[cfg(windows)]
    {
        // On Windows (PowerShell), quote -D arguments
        if arg.starts_with("-D") {
            format!("\"{}\"", arg)
        } else {
            arg.to_string()
        }
    }
    #[cfg(not(windows))]
    {
        arg.to_string()
    }
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
    build_command_string_with_options(
        maven_command,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false,
        Path::new("."),
    )
}

/// Build the full command string for display with option to use -f
fn build_command_string_with_options(
    maven_command: &str,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    project_root: &Path,
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
    execute_maven_command_with_options(
        project_root,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false, // use_file_flag = false for backward compatibility
    )
}

/// Execute Maven command with option to use -f instead of -pl
pub fn execute_maven_command_with_options(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
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
    )
}

/// Async version with option to use -f instead of -pl
pub fn execute_maven_command_async_with_options(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
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

    // Get profiles from Maven command output (POM files)
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
                    log::debug!("Found profile from POM: {}", profile_name);
                    profile_set.insert(profile_name.to_string());
                }
            }
        }
    }

    // Also get profiles from settings.xml (Maven's help:all-profiles doesn't include these)
    if let Some(settings_path) = config.maven_settings.as_ref() {
        log::debug!("Checking settings.xml for profiles: {}", settings_path);
        if let Ok(settings_content) = fs::read_to_string(settings_path)
            && let Ok(profiles_from_settings) =
                extract_profiles_from_settings_xml(&settings_content)
        {
            for profile_name in profiles_from_settings {
                log::debug!("Found profile from settings.xml: {}", profile_name);
                profile_set.insert(profile_name);
            }
        }
    }

    // Convert to sorted Vec for consistent ordering
    let mut profiles: Vec<String> = profile_set.into_iter().collect();
    profiles.sort();

    log::info!("Discovered {} unique Maven profiles", profiles.len());
    Ok(profiles)
}

/// Extract profile IDs from settings.xml content
fn extract_profiles_from_settings_xml(xml_content: &str) -> Result<Vec<String>, String> {
    let mut profiles = Vec::new();
    let lines: Vec<&str> = xml_content.lines().collect();

    let mut in_profiles_section = false;
    let mut in_profile = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if we're entering the <profiles> section
        if trimmed.starts_with("<profiles>") {
            in_profiles_section = true;
            continue;
        }

        // Check if we're leaving the <profiles> section
        if trimmed.starts_with("</profiles>") {
            in_profiles_section = false;
            continue;
        }

        if in_profiles_section {
            // Check if we're entering a <profile>
            if trimmed.starts_with("<profile>") {
                in_profile = true;
                continue;
            }

            // Check if we're leaving a <profile>
            if trimmed.starts_with("</profile>") {
                in_profile = false;
                continue;
            }

            // If we're in a profile, look for <id>
            if in_profile
                && trimmed.starts_with("<id>")
                && trimmed.contains("</id>")
                && let Some(id_start) = trimmed.find("<id>")
                && let Some(id_end) = trimmed.find("</id>")
            {
                let id = &trimmed[id_start + 4..id_end];
                profiles.push(id.to_string());
            }
        }
    }

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

/// Extract the XML snippet for a specific profile from POM files
/// Returns (profile_xml, source_pom_path) or None if not found
pub fn get_profile_xml(project_root: &Path, profile_id: &str) -> Option<(String, PathBuf)> {
    log::debug!(
        "Searching for profile '{}' XML in {:?}",
        profile_id,
        project_root
    );

    let mut pom_paths = Vec::new();

    // Load config to get the maven_settings path (which may be maven_settings.xml or settings.xml)
    let config = crate::config::load_config(project_root);

    // 1. If config has maven_settings configured, use that
    if let Some(ref settings_path) = config.maven_settings {
        let settings = PathBuf::from(settings_path);
        if settings.exists() {
            log::debug!("Using configured Maven settings: {:?}", settings);
            pom_paths.push(settings);
        }
    }

    // 2. Also check user settings.xml (~/.m2/settings.xml) if not already added
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let user_settings = PathBuf::from(home).join(".m2").join("settings.xml");
        if user_settings.exists() && !pom_paths.contains(&user_settings) {
            pom_paths.push(user_settings);
        }
    }

    // 3. Check project root pom.xml
    pom_paths.push(project_root.join("pom.xml"));

    // 4. Check module POMs
    if let Ok(entries) = fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let module_pom = path.join("pom.xml");
                if module_pom.exists() {
                    pom_paths.push(module_pom);
                }
            }
        }
    }

    // Search each file
    for pom_path in pom_paths {
        if let Ok(content) = fs::read_to_string(&pom_path)
            && let Some(xml) = extract_profile_from_xml(&content, profile_id)
        {
            log::info!("Found profile '{}' in {:?}", profile_id, pom_path);
            // Prettify the XML before returning
            let prettified = prettify_xml(&xml).unwrap_or(xml);
            return Some((prettified, pom_path));
        }
    }

    log::warn!(
        "Profile '{}' not found in any POM or settings file",
        profile_id
    );
    None
}

/// Prettify XML with proper indentation
fn prettify_xml(xml: &str) -> Option<String> {
    use std::io::Cursor;

    // Try to parse and reformat the XML
    match xmltree::Element::parse(Cursor::new(xml.as_bytes())) {
        Ok(element) => {
            let mut output = Vec::new();
            let config = xmltree::EmitterConfig::new()
                .perform_indent(true)
                .indent_string("    ");

            if element.write_with_config(&mut output, config).is_ok() {
                String::from_utf8(output).ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Extract a single profile XML block from POM content
fn extract_profile_from_xml(xml_content: &str, profile_id: &str) -> Option<String> {
    // Find the profile block with the matching ID
    // Look for <profile> ... <id>profile_id</id> ... </profile>

    let mut in_profile = false;
    let mut in_profile_id = false;
    let mut current_profile = String::new();
    let mut depth = 0;
    let mut found_matching_id = false;

    for line in xml_content.lines() {
        let trimmed = line.trim();

        // Track when we enter a <profile> tag
        if trimmed.starts_with("<profile>") || trimmed.starts_with("<profile ") {
            in_profile = true;
            current_profile.clear();
            found_matching_id = false;
            depth = 0;
        }

        if in_profile {
            current_profile.push_str(line);
            current_profile.push('\n');

            // Track depth to handle nested tags
            if trimmed.contains("<profile>") {
                depth += 1;
            }

            // Check if we're in the <id> tag
            if trimmed.starts_with("<id>") {
                in_profile_id = true;
                if trimmed.contains(profile_id) && trimmed.contains("</id>") {
                    found_matching_id = true;
                }
            }

            if in_profile_id && trimmed.contains("</id>") {
                in_profile_id = false;
            }

            // Check if we've closed the profile tag
            if trimmed.contains("</profile>") {
                depth -= 1;
                if depth == 0 {
                    in_profile = false;

                    // If this was the matching profile, return it
                    if found_matching_id {
                        // Clean up the XML - preserve indentation
                        return Some(current_profile.trim_end().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Detect Spring Boot configuration for a module by analyzing effective POM
pub fn detect_spring_boot_capabilities(
    project_root: &Path,
    module: Option<&str>,
) -> Result<SpringBootDetection, std::io::Error> {
    log::debug!(
        "Detecting Spring Boot capabilities for module: {:?}",
        module
    );

    let config = crate::config::load_config(project_root);

    // Get effective POM for the module
    let args = vec!["help:effective-pom"];

    let output = execute_maven_command(
        project_root,
        module,
        &args,
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    let pom_content = output.join("\n");

    let mut detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: false,
        main_class: None,
        packaging: None,
    };

    // Parse the effective POM
    let mut in_plugins = false;
    let mut in_plugin = false;
    let mut current_plugin_artifact_id = String::new();
    let mut in_configuration = false;

    for line in pom_content.lines() {
        let trimmed = line.trim();

        // Detect packaging
        if trimmed.starts_with("<packaging>")
            && trimmed.contains("</packaging>")
            && let Some(start) = trimmed.find("<packaging>")
            && let Some(end) = trimmed.find("</packaging>")
        {
            let packaging = &trimmed[start + 11..end];
            detection.packaging = Some(packaging.to_string());
            log::debug!("Found packaging: {}", packaging);
        }

        // Track plugin sections
        if trimmed.starts_with("<plugins>") {
            in_plugins = true;
        } else if trimmed.starts_with("</plugins>") {
            in_plugins = false;
        }

        if in_plugins {
            if trimmed.starts_with("<plugin>") {
                in_plugin = true;
                current_plugin_artifact_id.clear();
            } else if trimmed.starts_with("</plugin>") {
                in_plugin = false;
                in_configuration = false;
            }

            if in_plugin {
                // Check for Spring Boot plugin
                if trimmed.starts_with("<artifactId>spring-boot-maven-plugin</artifactId>") {
                    detection.has_spring_boot_plugin = true;
                    current_plugin_artifact_id = "spring-boot-maven-plugin".to_string();
                    log::debug!("Found spring-boot-maven-plugin");
                }

                // Check for exec plugin
                if trimmed.starts_with("<artifactId>exec-maven-plugin</artifactId>") {
                    detection.has_exec_plugin = true;
                    current_plugin_artifact_id = "exec-maven-plugin".to_string();
                    log::debug!("Found exec-maven-plugin");
                }

                // Track configuration section
                if trimmed.starts_with("<configuration>") {
                    in_configuration = true;
                } else if trimmed.starts_with("</configuration>") {
                    in_configuration = false;
                }

                // Extract mainClass from configuration
                if in_configuration
                    && (trimmed.starts_with("<mainClass>") || trimmed.starts_with("<main-class>"))
                    && (trimmed.contains("</mainClass>") || trimmed.contains("</main-class>"))
                {
                    let main_class = if trimmed.contains("</mainClass>") {
                        extract_tag_content(trimmed, "mainClass")
                    } else {
                        extract_tag_content(trimmed, "main-class")
                    };

                    if let Some(mc) = main_class {
                        detection.main_class = Some(mc.clone());
                        log::debug!("Found mainClass '{}' in {}", mc, current_plugin_artifact_id);
                    }
                }
            }
        }

        // Also check for properties (spring-boot.run.mainClass, start-class, etc.)
        if trimmed.starts_with("<spring-boot.run.mainClass>")
            || trimmed.starts_with("<spring-boot.main-class>")
            || trimmed.starts_with("<start-class>")
        {
            let property_name = if trimmed.contains("spring-boot.run.mainClass") {
                "spring-boot.run.mainClass"
            } else if trimmed.contains("spring-boot.main-class") {
                "spring-boot.main-class"
            } else {
                "start-class"
            };

            if let Some(mc) = extract_tag_content(trimmed, property_name)
                && detection.main_class.is_none()
            {
                detection.main_class = Some(mc.clone());
                log::debug!("Found mainClass '{}' from property {}", mc, property_name);
            }
        }
    }

    log::info!(
        "Spring Boot detection results: plugin={}, exec={}, mainClass={:?}, packaging={:?}",
        detection.has_spring_boot_plugin,
        detection.has_exec_plugin,
        detection.main_class,
        detection.packaging
    );

    Ok(detection)
}

/// Extract content from an XML tag
fn extract_tag_content(line: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag_name);
    let close_tag = format!("</{}>", tag_name);

    if let Some(start) = line.find(&open_tag)
        && let Some(end) = line.find(&close_tag)
    {
        let content = &line[start + open_tag.len()..end];
        return Some(content.trim().to_string());
    }
    None
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
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();
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
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();
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
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();
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
    fn test_get_profile_xml() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a POM with a profile
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-project</artifactId>
    <version>1.0.0</version>
    
    <profiles>
        <profile>
            <id>dev</id>
            <activation>
                <activeByDefault>true</activeByDefault>
            </activation>
            <properties>
                <env>development</env>
            </properties>
        </profile>
        <profile>
            <id>prod</id>
            <properties>
                <env>production</env>
            </properties>
        </profile>
    </profiles>
</project>"#;

        fs::write(project_root.join("pom.xml"), pom_content).unwrap();

        // Test extracting the dev profile
        let result = get_profile_xml(project_root, "dev");
        assert!(result.is_some(), "Should find dev profile");

        let (xml, _path) = result.unwrap();
        assert!(
            xml.contains("<id>dev</id>"),
            "XML should contain profile ID"
        );
        assert!(
            xml.contains("<env>development</env>"),
            "XML should contain profile properties"
        );
        assert!(xml.contains("<profile>"), "XML should have opening tag");
        assert!(xml.contains("</profile>"), "XML should have closing tag");

        // Test extracting the prod profile
        let result = get_profile_xml(project_root, "prod");
        assert!(result.is_some(), "Should find prod profile");

        let (xml, _path) = result.unwrap();
        assert!(
            xml.contains("<id>prod</id>"),
            "XML should contain prod profile ID"
        );
        assert!(
            xml.contains("<env>production</env>"),
            "XML should contain prod properties"
        );

        // Test non-existent profile
        let result = get_profile_xml(project_root, "nonexistent");
        assert!(result.is_none(), "Should not find nonexistent profile");
    }

    #[test]
    fn test_get_profile_xml_from_settings() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create a settings.xml with profiles
        let settings_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <profiles>
        <profile>
            <id>corporate-proxy</id>
            <properties>
                <http.proxyHost>proxy.corp.com</http.proxyHost>
                <http.proxyPort>8080</http.proxyPort>
            </properties>
        </profile>
        <profile>
            <id>development</id>
            <activation>
                <activeByDefault>false</activeByDefault>
            </activation>
            <properties>
                <maven.compiler.debug>true</maven.compiler.debug>
                <env>dev</env>
            </properties>
        </profile>
    </profiles>
</settings>"#;

        fs::write(project_root.join("settings.xml"), settings_content).unwrap();

        // Also create a POM to ensure we search settings.xml first
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test</artifactId>
    <version>1.0.0</version>
</project>"#;
        fs::write(project_root.join("pom.xml"), pom_content).unwrap();

        // Test finding profile from settings.xml
        let result = get_profile_xml(project_root, "development");
        assert!(
            result.is_some(),
            "Should find development profile from settings.xml"
        );

        let (xml, path) = result.unwrap();
        assert!(
            xml.contains("<id>development</id>"),
            "XML should contain development profile"
        );
        assert!(
            xml.contains("<env>dev</env>"),
            "XML should contain settings profile properties"
        );
        assert!(
            path.ends_with("settings.xml"),
            "Should be from settings.xml"
        );

        // Test corporate proxy profile
        let result = get_profile_xml(project_root, "corporate-proxy");
        assert!(result.is_some(), "Should find corporate-proxy profile");

        let (xml, _) = result.unwrap();
        assert!(
            xml.contains("proxy.corp.com"),
            "XML should contain proxy settings"
        );
    }

    #[test]
    fn test_get_profile_xml_with_maven_settings_xml() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // Create maven_settings.xml (note: not settings.xml)
        let maven_settings = project_root.join("maven_settings.xml");
        fs::write(
            &maven_settings,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <profiles>
        <profile>
            <id>custom-profile</id>
            <properties>
                <custom.property>custom-value</custom.property>
            </properties>
        </profile>
    </profiles>
</settings>"#,
        )
        .unwrap();

        // Create lazymvn.toml to point to maven_settings.xml
        let config_file = project_root.join("lazymvn.toml");
        fs::write(
            &config_file,
            format!("maven_settings = \"{}\"", maven_settings.to_str().unwrap()),
        )
        .unwrap();

        // Also need a pom.xml so it's a valid Maven project
        let pom = project_root.join("pom.xml");
        fs::write(&pom, "<project></project>").unwrap();

        // Test finding profile from maven_settings.xml
        let result = get_profile_xml(project_root, "custom-profile");
        assert!(
            result.is_some(),
            "Should find custom-profile from maven_settings.xml"
        );

        let (xml, path) = result.unwrap();
        assert!(
            xml.contains("<id>custom-profile</id>"),
            "XML should contain profile ID"
        );
        assert!(
            xml.contains("custom-value"),
            "XML should contain custom property"
        );
        assert_eq!(
            path, maven_settings,
            "Should return maven_settings.xml path"
        );
    }

    #[test]
    fn test_spring_boot_detection_with_plugin() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
        };

        assert!(
            detection.can_use_spring_boot_run(),
            "Should be able to use spring-boot:run with plugin and jar packaging"
        );
    }

    #[test]
    fn test_spring_boot_detection_with_war_packaging() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("war".to_string()),
        };

        assert!(
            detection.can_use_spring_boot_run(),
            "Should be able to use spring-boot:run with plugin and war packaging"
        );
    }

    #[test]
    fn test_spring_boot_detection_with_pom_packaging() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("pom".to_string()),
        };

        assert!(
            !detection.can_use_spring_boot_run(),
            "Should not be able to use spring-boot:run with pom packaging"
        );
    }

    #[test]
    fn test_spring_boot_detection_fallback_to_exec() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.App".to_string()),
            packaging: Some("jar".to_string()),
        };

        assert!(
            !detection.can_use_spring_boot_run(),
            "Should not use spring-boot:run without plugin"
        );
        assert!(
            detection.can_use_exec_java(),
            "Should be able to use exec:java with exec plugin"
        );
    }

    #[test]
    fn test_launch_strategy_auto_prefers_spring_boot() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: true,
            main_class: Some("com.example.App".to_string()),
            packaging: Some("jar".to_string()),
        };

        let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
        assert_eq!(
            strategy,
            LaunchStrategy::SpringBootRun,
            "Auto mode should prefer spring-boot:run when available"
        );
    }

    #[test]
    fn test_launch_strategy_auto_falls_back_to_exec() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.App".to_string()),
            packaging: Some("jar".to_string()),
        };

        let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
        assert_eq!(
            strategy,
            LaunchStrategy::ExecJava,
            "Auto mode should fall back to exec:java when spring-boot:run not available"
        );
    }

    #[test]
    fn test_launch_strategy_force_run() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.App".to_string()),
            packaging: Some("jar".to_string()),
        };

        let strategy = decide_launch_strategy(&detection, LaunchMode::ForceRun);
        assert_eq!(
            strategy,
            LaunchStrategy::SpringBootRun,
            "ForceRun should always use spring-boot:run"
        );
    }

    #[test]
    fn test_launch_strategy_force_exec() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
        };

        let strategy = decide_launch_strategy(&detection, LaunchMode::ForceExec);
        assert_eq!(
            strategy,
            LaunchStrategy::ExecJava,
            "ForceExec should always use exec:java"
        );
    }

    #[test]
    fn test_extract_tag_content() {
        let line = "<mainClass>com.example.Application</mainClass>";
        let content = extract_tag_content(line, "mainClass");
        assert_eq!(content, Some("com.example.Application".to_string()));

        let line_with_spaces = "  <packaging>jar</packaging>  ";
        let content = extract_tag_content(line_with_spaces, "packaging");
        assert_eq!(content, Some("jar".to_string()));

        let invalid_line = "<mainClass>incomplete";
        let content = extract_tag_content(invalid_line, "mainClass");
        assert_eq!(content, None);
    }

    #[test]
    fn test_build_launch_command_spring_boot_run() {
        let profiles = vec!["dev".to_string(), "debug".to_string()];
        let jvm_args = vec!["-Dfoo=bar".to_string(), "-Xmx512m".to_string()];

        let command =
            build_launch_command(LaunchStrategy::SpringBootRun, None, &profiles, &jvm_args);

        // Should contain profiles argument
        assert!(
            command
                .iter()
                .any(|arg| arg.contains("spring-boot.run.profiles=dev,debug")),
            "Should set profiles: {:?}",
            command
        );

        // Should contain JVM arguments
        assert!(
            command
                .iter()
                .any(|arg| arg.contains("spring-boot.run.jvmArguments")),
            "Should set jvmArguments: {:?}",
            command
        );

        // Should end with the goal
        assert_eq!(command.last(), Some(&"spring-boot:run".to_string()));
    }

    #[test]
    fn test_build_launch_command_exec_java() {
        let jvm_args = vec!["-Dfoo=bar".to_string()];

        let command = build_launch_command(
            LaunchStrategy::ExecJava,
            Some("com.example.Application"),
            &[],
            &jvm_args,
        );

        // Should contain mainClass argument
        assert!(
            command
                .iter()
                .any(|arg| arg.contains("exec.mainClass=com.example.Application")),
            "Should set mainClass: {:?}",
            command
        );

        // Should contain JVM args
        assert!(
            command.contains(&quote_arg_for_platform("-Dfoo=bar")),
            "Should include JVM args: {:?}",
            command
        );

        // Should end with the goal
        assert_eq!(command.last(), Some(&"exec:java".to_string()));
    }

    #[test]
    fn test_build_launch_command_exec_java_without_main_class() {
        let command = build_launch_command(LaunchStrategy::ExecJava, None, &[], &[]);

        // Should not contain mainClass if not provided
        assert!(
            !command.iter().any(|arg| arg.contains("exec.mainClass")),
            "Should not set mainClass if none provided: {:?}",
            command
        );

        // Should still have the goal
        assert_eq!(command.last(), Some(&"exec:java".to_string()));
    }

    #[test]
    #[cfg(windows)]
    fn test_quote_arg_for_platform_windows() {
        assert_eq!(
            quote_arg_for_platform("-Dfoo=bar"),
            "\"-Dfoo=bar\"",
            "Should quote -D args on Windows"
        );
        assert_eq!(
            quote_arg_for_platform("spring-boot:run"),
            "spring-boot:run",
            "Should not quote goals"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_quote_arg_for_platform_unix() {
        assert_eq!(
            quote_arg_for_platform("-Dfoo=bar"),
            "-Dfoo=bar",
            "Should not quote on Unix"
        );
    }

    #[test]
    fn test_extract_profiles_from_settings_xml() {
        let settings_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <profiles>
        <profile>
            <id>development</id>
            <properties>
                <env>dev</env>
            </properties>
        </profile>
        <profile>
            <id>production</id>
            <properties>
                <env>prod</env>
            </properties>
        </profile>
        <profile>
            <id>testing</id>
            <properties>
                <env>test</env>
            </properties>
        </profile>
    </profiles>
</settings>"#;

        let profiles = extract_profiles_from_settings_xml(settings_xml).unwrap();
        assert_eq!(profiles.len(), 3, "Should find 3 profiles");
        assert!(profiles.contains(&"development".to_string()));
        assert!(profiles.contains(&"production".to_string()));
        assert!(profiles.contains(&"testing".to_string()));
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
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();
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
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();
        assert_eq!(maven_output, vec!["test"]);
    }

    #[test]
    #[cfg(unix)]
    fn test_exec_java_with_file_flag_adds_also_make() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let output: Vec<String> = execute_maven_command_with_options(
            project_root,
            Some("my-module"),
            &["exec:java"],
            &[],
            None,
            &[],
            true, // use_file_flag = true
        )
        .unwrap()
        .iter()
        .filter_map(|line| utils::clean_log_line(line))
        .collect();

        // Skip command line header
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();

        // Should contain -f flag, --also-make, and exec:java
        let command_output = maven_output.join(" ");
        assert!(command_output.contains("-f"));
        assert!(command_output.contains("--also-make"));
        assert!(command_output.contains("exec:java"));
    }

    #[test]
    #[cfg(unix)]
    fn test_exec_java_with_file_flag_preserves_existing_also_make() {
        let _guard = test_lock().lock().unwrap();
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        let mvnw_path = project_root.join("mvnw");
        write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

        let flags = vec!["--also-make-dependents".to_string()];
        let output: Vec<String> = execute_maven_command_with_options(
            project_root,
            Some("my-module"),
            &["exec:java"],
            &[],
            None,
            &flags,
            true, // use_file_flag = true
        )
        .unwrap()
        .iter()
        .filter_map(|line| utils::clean_log_line(line))
        .collect();

        // Skip command line header
        let maven_output: Vec<String> = output
            .iter()
            .skip_while(|line| line.starts_with("$ "))
            .cloned()
            .collect();

        let command_output = maven_output.join(" ");
        // Should contain existing flag but not auto-add --also-make
        assert!(command_output.contains("--also-make-dependents"));
        // Should have only one occurrence of "also-make" (from the existing flag)
        assert_eq!(command_output.matches("also-make").count(), 1);
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
