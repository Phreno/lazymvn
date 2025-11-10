//! Maven command string building utilities

use crate::core::config::LoggingConfig;
use std::path::Path;
use super::log4j_config::get_logging_overrides;

/// Determine the Maven command to use (wrapper or system Maven)
///
/// Prefers Maven wrapper (mvnw) over system Maven when available.
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
/// use lazymvn::maven::command::get_maven_command;
///
/// let project_root = Path::new("/path/to/project");
/// let maven_cmd = get_maven_command(project_root);
/// // Returns "./mvnw" if mvnw exists, otherwise "mvn" (or "mvn.cmd" on Windows)
/// ```
pub fn get_maven_command(project_root: &Path) -> String {
    find_maven_wrapper(project_root).unwrap_or_else(get_default_maven_command)
}

/// Try to find Maven wrapper in project
fn find_maven_wrapper(project_root: &Path) -> Option<String> {
    #[cfg(unix)]
    {
        find_unix_wrapper(project_root)
    }

    #[cfg(windows)]
    {
        find_windows_wrapper(project_root)
    }
}

/// Find Maven wrapper on Unix systems
#[cfg(unix)]
fn find_unix_wrapper(project_root: &Path) -> Option<String> {
    if wrapper_exists(project_root, "mvnw") {
        Some("./mvnw".to_string())
    } else {
        None
    }
}

/// Find Maven wrapper on Windows systems
#[cfg(windows)]
fn find_windows_wrapper(project_root: &Path) -> Option<String> {
    for wrapper in &["mvnw.bat", "mvnw.cmd", "mvnw"] {
        if wrapper_exists(project_root, wrapper) {
            return Some(wrapper.to_string());
        }
    }
    None
}

/// Check if wrapper exists
pub(super) fn wrapper_exists(project_root: &Path, wrapper_name: &str) -> bool {
    project_root.join(wrapper_name).exists()
}

/// Get default Maven command
fn get_default_maven_command() -> String {
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
        None, // No logging overrides for backward compatibility
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
    logging_config: Option<&LoggingConfig>,
) -> String {
    let mut parts = vec![maven_command.to_string()];

    add_settings_if_present(&mut parts, settings_path);
    add_profiles_if_present(&mut parts, profiles);
    add_module_if_present(&mut parts, module, use_file_flag, project_root, args, flags);
    add_filtered_flags(&mut parts, flags, args);
    add_logging_config_if_needed(&mut parts, logging_config, args);
    add_args(&mut parts, args);

    parts.join(" ")
}

/// Add settings path to command parts
fn add_settings_if_present(parts: &mut Vec<String>, settings_path: Option<&str>) {
    if let Some(settings_path) = settings_path {
        parts.push("--settings".to_string());
        parts.push(settings_path.to_string());
    }
}

/// Add profiles to command parts
fn add_profiles_if_present(parts: &mut Vec<String>, profiles: &[String]) {
    if !profiles.is_empty() {
        parts.push("-P".to_string());
        parts.push(profiles.join(","));
    }
}

/// Add module to command parts
fn add_module_if_present(
    parts: &mut Vec<String>,
    module: Option<&str>,
    use_file_flag: bool,
    project_root: &Path,
    args: &[&str],
    flags: &[String],
) {
    if let Some(module) = module
        && module != "."
    {
        if use_file_flag {
            add_file_flag_module(parts, project_root, module, args, flags);
        } else {
            add_project_list_module(parts, module);
        }
    }
}

/// Add module using -f flag
fn add_file_flag_module(
    parts: &mut Vec<String>,
    project_root: &Path,
    module: &str,
    args: &[&str],
    flags: &[String],
) {
    let module_pom = project_root.join(module).join("pom.xml");
    parts.push("-f".to_string());
    parts.push(module_pom.to_string_lossy().to_string());

    if should_auto_add_also_make(args, flags) {
        parts.push("--also-make".to_string());
    }
}

/// Add module using -pl flag
fn add_project_list_module(parts: &mut Vec<String>, module: &str) {
    parts.push("-pl".to_string());
    parts.push(module.to_string());
}

/// Check if --also-make should be auto-added
fn should_auto_add_also_make(args: &[&str], flags: &[String]) -> bool {
    args.contains(&"exec:java") && !flags.iter().any(|f| f.contains("also-make"))
}

/// Add filtered flags to command parts
fn add_filtered_flags(parts: &mut Vec<String>, flags: &[String], args: &[&str]) {
    let is_spring_boot_run = is_spring_boot_run_command(args);
    let filtered_flags = filter_flags_for_command(flags, is_spring_boot_run);
    
    for flag in filtered_flags {
        add_flag_parts(parts, flag);
    }
}

/// Check if this is a spring-boot:run command
fn is_spring_boot_run_command(args: &[&str]) -> bool {
    args.iter().any(|arg| {
        arg.contains("spring-boot:run") || arg.contains("spring-boot-maven-plugin") && arg.contains(":run")
    })
}

/// Filter flags based on command type
fn filter_flags_for_command(flags: &[String], is_spring_boot_run: bool) -> Vec<&str> {
    if is_spring_boot_run {
        flags.iter()
            .filter(|flag| !flag.to_lowercase().contains("also-make"))
            .map(|s| s.as_str())
            .collect()
    } else {
        flags.iter().map(|s| s.as_str()).collect()
    }
}

/// Add flag parts to command parts
fn add_flag_parts(parts: &mut Vec<String>, flag: &str) {
    let flag_parts: Vec<&str> = flag
        .split(',')
        .next()
        .unwrap_or(flag)
        .split_whitespace()
        .collect();
    
    for part in flag_parts {
        if !part.is_empty() {
            parts.push(part.to_string());
        }
    }
}

/// Add logging configuration if needed
fn add_logging_config_if_needed(
    parts: &mut Vec<String>,
    logging_config: Option<&LoggingConfig>,
    args: &[&str],
) {
    if has_spring_boot_jvm_args(args) {
        return;
    }

    if let Some(logging_config) = logging_config {
        add_log_format_config(parts, logging_config);
        add_logging_overrides(parts, logging_config);
    }
}

/// Check if command has Spring Boot JVM arguments
fn has_spring_boot_jvm_args(args: &[&str]) -> bool {
    args.iter().any(|arg| {
        arg.starts_with("-Dspring-boot.run.jvmArguments=") || arg.starts_with("-Drun.jvmArguments=")
    })
}

/// Add log format configuration
fn add_log_format_config(parts: &mut Vec<String>, logging_config: &LoggingConfig) {
    if let Some(log_format) = &logging_config.log_format {
        parts.push(format!("-Dlog4j.conversionPattern={}", log_format));
        parts.push(format!("-Dlogging.pattern.console={}", log_format));
    }
}

/// Add logging level overrides
fn add_logging_overrides(parts: &mut Vec<String>, logging_config: &LoggingConfig) {
    for (package, level) in &get_logging_overrides(Some(logging_config)) {
        parts.push(format!("-Dlog4j.logger.{}={}", package, level));
        parts.push(format!("-Dlogging.level.{}={}", package, level));
    }
}

/// Add arguments to command parts
fn add_args(parts: &mut Vec<String>, args: &[&str]) {
    for arg in args {
        parts.push(arg.to_string());
    }
}

/// Check Maven availability and return version info
pub fn check_maven_availability(project_root: &Path) -> Result<String, std::io::Error> {
    use std::process::Command;
    
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

