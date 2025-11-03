//! Test harness for lazymvn
//!
//! This library exposes lazymvn's core functionality without the TUI,
//! making it easy to write integration tests that verify build, start,
//! and other Maven operations work correctly.
//!
//! # Example
//!
//! ```no_run
//! use lazymvn_test_harness::{TestProject, CommandResult};
//!
//! let project = TestProject::new("demo/multi-module");
//! let result = project.build_module("library").unwrap();
//!
//! assert!(result.success);
//! assert!(result.output.contains("BUILD SUCCESS"));
//! ```

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// Result of a Maven command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command succeeded
    pub success: bool,
    /// All output lines (stdout + stderr)
    pub output: Vec<String>,
    /// Exit code if available
    pub exit_code: Option<i32>,
    /// Duration of execution
    pub duration: Duration,
}

impl CommandResult {
    /// Check if output contains a specific string
    pub fn contains(&self, needle: &str) -> bool {
        self.output.iter().any(|line| line.contains(needle))
    }

    /// Check if output matches a pattern
    pub fn matches(&self, pattern: &str) -> bool {
        self.output.iter().any(|line| line.contains(pattern))
    }

    /// Get all lines containing a pattern
    pub fn lines_matching(&self, pattern: &str) -> Vec<String> {
        self.output
            .iter()
            .filter(|line| line.contains(pattern))
            .cloned()
            .collect()
    }

    /// Count lines in output
    pub fn line_count(&self) -> usize {
        self.output.len()
    }
}

/// A test project instance
pub struct TestProject {
    /// Project root directory
    pub root: PathBuf,
    /// Maven settings file (if any)
    pub settings: Option<PathBuf>,
    /// Active profiles
    pub profiles: Vec<String>,
    /// Build flags
    pub flags: Vec<String>,
}

impl TestProject {
    /// Create a new test project from a path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root = std::env::current_dir()
            .unwrap()
            .join(path.as_ref());
        
        Self {
            root,
            settings: None,
            profiles: Vec::new(),
            flags: Vec::new(),
        }
    }

    /// Set Maven settings file
    pub fn with_settings<P: AsRef<Path>>(mut self, settings: P) -> Self {
        self.settings = Some(self.root.join(settings.as_ref()));
        self
    }

    /// Add profile(s)
    pub fn with_profiles(mut self, profiles: &[&str]) -> Self {
        self.profiles.extend(profiles.iter().map(|s| s.to_string()));
        self
    }

    /// Add build flag(s)
    pub fn with_flags(mut self, flags: &[&str]) -> Self {
        self.flags.extend(flags.iter().map(|s| s.to_string()));
        self
    }

    /// Execute a Maven build on a module
    pub fn build_module(&self, module: &str) -> Result<CommandResult, String> {
        self.run_command(module, &["clean", "install"])
    }

    /// Execute a Maven package on a module
    pub fn package_module(&self, module: &str) -> Result<CommandResult, String> {
        self.run_command(module, &["clean", "package"])
    }

    /// Execute a Maven clean on a module
    pub fn clean_module(&self, module: &str) -> Result<CommandResult, String> {
        self.run_command(module, &["clean"])
    }

    /// Execute a Maven compile on a module
    pub fn compile_module(&self, module: &str) -> Result<CommandResult, String> {
        self.run_command(module, &["compile"])
    }

    /// Start a Spring Boot application
    pub fn start_module(&self, module: &str) -> Result<CommandResult, String> {
        self.run_command(module, &["spring-boot:run"])
    }

    /// Execute a custom Maven command
    pub fn run_command(&self, module: &str, args: &[&str]) -> Result<CommandResult, String> {
        self.run_command_with_options(module, args, false)
    }

    /// Execute a Maven command with advanced options
    pub fn run_command_with_options(
        &self,
        module: &str,
        args: &[&str],
        use_file_flag: bool,
    ) -> Result<CommandResult, String> {
        use std::time::Instant;

        let start = Instant::now();

        log::info!("Running command on module '{}'", module);
        log::debug!("  root: {:?}", self.root);
        log::debug!("  args: {:?}", args);
        log::debug!("  profiles: {:?}", self.profiles);
        log::debug!("  flags: {:?}", self.flags);

        // Use lazymvn's async execution
        let receiver = lazymvn::maven::execute_maven_command_async_with_options(
            &self.root,
            Some(module),
            args,
            &self.profiles,
            self.settings.as_ref().and_then(|p| p.to_str()),
            &self.flags,
            use_file_flag,
            None, // No logging config for now
        )
        .map_err(|e| format!("Failed to execute command: {}", e))?;

        // Collect output
        let output = collect_command_output(receiver)?;

        let duration = start.elapsed();

        Ok(CommandResult {
            success: output.success,
            output: output.lines,
            exit_code: output.exit_code,
            duration,
        })
    }

    /// Build the entire project (all modules)
    pub fn build_all(&self) -> Result<CommandResult, String> {
        self.run_command(".", &["clean", "install"])
    }

    /// Clean the entire project
    pub fn clean_all(&self) -> Result<CommandResult, String> {
        self.run_command(".", &["clean"])
    }
}

/// Internal structure to collect command output
struct CommandOutput {
    success: bool,
    lines: Vec<String>,
    exit_code: Option<i32>,
}

/// Collect all output from a Maven command receiver
fn collect_command_output(
    receiver: mpsc::Receiver<lazymvn::maven::CommandUpdate>,
) -> Result<CommandOutput, String> {
    use lazymvn::maven::CommandUpdate;

    let mut lines = Vec::new();
    let mut success = false;
    let mut exit_code = None;

    loop {
        match receiver.recv() {
            Ok(CommandUpdate::Started(pid)) => {
                log::debug!("Process started with PID: {}", pid);
            }
            Ok(CommandUpdate::OutputLine(line)) => {
                log::trace!("Output: {}", line);
                lines.push(line);
            }
            Ok(CommandUpdate::Completed) => {
                log::info!("Command completed successfully");
                success = true;
                exit_code = Some(0);
                break;
            }
            Ok(CommandUpdate::Error(err)) => {
                log::warn!("Command failed: {}", err);
                lines.push(format!("ERROR: {}", err));
                success = false;
                
                // Try to extract exit code from error message
                if let Some(code_str) = err.split("exit code ").nth(1) {
                    if let Ok(code) = code_str.split_whitespace().next().unwrap_or("").parse() {
                        exit_code = Some(code);
                    }
                }
                break;
            }
            Err(_) => {
                log::error!("Channel closed unexpectedly");
                return Err("Command channel closed unexpectedly".to_string());
            }
        }
    }

    Ok(CommandOutput {
        success,
        lines,
        exit_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = TestProject::new("demo/multi-module")
            .with_profiles(&["dev"])
            .with_flags(&["-U"]);

        assert!(project.root.to_str().unwrap().ends_with("demo/multi-module"));
        assert_eq!(project.profiles, vec!["dev"]);
        assert_eq!(project.flags, vec!["-U"]);
    }

    #[test]
    fn test_command_result_contains() {
        let result = CommandResult {
            success: true,
            output: vec![
                "Building module...".to_string(),
                "BUILD SUCCESS".to_string(),
            ],
            exit_code: Some(0),
            duration: Duration::from_secs(1),
        };

        assert!(result.contains("BUILD SUCCESS"));
        assert!(result.contains("Building"));
        assert!(!result.contains("FAILURE"));
    }

    #[test]
    fn test_command_result_lines_matching() {
        let result = CommandResult {
            success: true,
            output: vec![
                "Compiling class A".to_string(),
                "Building jar".to_string(),
                "Compiling class B".to_string(),
            ],
            exit_code: Some(0),
            duration: Duration::from_secs(1),
        };

        let matching = result.lines_matching("Compiling");
        assert_eq!(matching.len(), 2);
        assert!(matching[0].contains("class A"));
        assert!(matching[1].contains("class B"));
    }
}
