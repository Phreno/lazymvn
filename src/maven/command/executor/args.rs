use crate::core::config::LoggingConfig;
use crate::maven::command::helpers::{filter_spring_boot_incompatible_flags, is_spring_boot_run_command, parse_flag_parts};
use crate::maven::command::log4j_config::get_logging_overrides;
use std::path::Path;
use std::process::Command;

/// Add Maven arguments to command
#[allow(clippy::too_many_arguments)]
pub fn add_maven_arguments(
    command: &mut Command,
    module: Option<&str>,
    project_root: &Path,
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    args: &[&str],
    use_file_flag: bool,
    logging_config: Option<&LoggingConfig>,
) {
    // Add settings
    if let Some(settings_path) = settings_path {
        command.arg("--settings").arg(settings_path);
        log::debug!("Added settings argument: {}", settings_path);
    }

    // Add profiles
    if !profiles.is_empty() {
        let profile_str = profiles.join(",");
        command.arg("-P").arg(&profile_str);
        log::debug!("Added profiles: {}", profile_str);
    }

    // Add module selection
    add_module_arguments(command, module, project_root, args, flags, use_file_flag);

    // Filter and add build flags
    let filtered_flags = filter_flags_for_spring_boot(args, flags);
    add_build_flags(command, &filtered_flags);

    // Add logging configuration
    add_logging_arguments(command, args, logging_config);

    // Add Maven goal arguments
    for arg in args {
        command.arg(arg);
        log::debug!("Added arg: {}", arg);
    }
}

/// Add module-specific arguments (-pl or -f)
fn add_module_arguments(
    command: &mut Command,
    module: Option<&str>,
    project_root: &Path,
    args: &[&str],
    flags: &[String],
    use_file_flag: bool,
) {
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
            }
        } else {
            log::debug!("Running on project root, no -pl/-f flag needed");
        }
    }
}

/// Filter flags for Spring Boot compatibility
fn filter_flags_for_spring_boot(args: &[&str], flags: &[String]) -> Vec<String> {
    let is_spring_boot = is_spring_boot_run_command(args);
    
    if is_spring_boot {
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
    }
}

/// Add build flags to command
fn add_build_flags(command: &mut Command, flags: &[String]) {
    for flag in flags {
        // Split flags like "-U, --update-snapshots" into individual flags
        let flag_parts = parse_flag_parts(flag);
        
        for part in &flag_parts {
            command.arg(part);
            log::debug!("Added flag: {}", part);
        }
    }
}

/// Add logging configuration arguments
fn add_logging_arguments(
    command: &mut Command,
    args: &[&str],
    logging_config: Option<&LoggingConfig>,
) {
    let has_spring_boot_jvm_args = args
        .iter()
        .any(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments=") || arg.starts_with("-Drun.jvmArguments="));

    if let Some(logging_config) = logging_config
        && !has_spring_boot_jvm_args
    {
        let logging_overrides = get_logging_overrides(Some(logging_config));
        
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn filter_flags_removes_also_make_for_spring_boot() {
        let args = vec!["spring-boot:run"];
        let flags = vec!["--also-make".to_string(), "-U".to_string()];
        let filtered = filter_flags_for_spring_boot(&args, &flags);
        assert_eq!(filtered, vec!["-U"]);
    }

    #[test]
    fn filter_flags_preserves_flags_for_non_spring_boot() {
        let args = vec!["clean", "install"];
        let flags = vec!["--also-make".to_string(), "-U".to_string()];
        let filtered = filter_flags_for_spring_boot(&args, &flags);
        assert_eq!(filtered, flags);
    }

    #[test]
    fn add_module_arguments_uses_pl_by_default() {
        let mut command = Command::new("mvn");
        let project_root = PathBuf::from("/tmp");
        add_module_arguments(&mut command, Some("my-module"), &project_root, &[], &[], false);
        
        // Check command string representation contains -pl
        let cmd_str = format!("{:?}", command);
        assert!(cmd_str.contains("-pl"), "Command should contain -pl: {}", cmd_str);
        assert!(cmd_str.contains("my-module"), "Command should contain my-module: {}", cmd_str);
    }

    #[test]
    fn add_module_arguments_uses_f_when_requested() {
        let mut command = Command::new("mvn");
        let project_root = PathBuf::from("/tmp");
        add_module_arguments(&mut command, Some("my-module"), &project_root, &[], &[], true);
        
        // Check command string representation contains -f
        let cmd_str = format!("{:?}", command);
        assert!(cmd_str.contains("-f"), "Command should contain -f: {}", cmd_str);
        assert!(cmd_str.contains("pom.xml"), "Command should contain pom.xml: {}", cmd_str);
    }
}
