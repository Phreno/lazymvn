//! Maven command execution utilities

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::builder::MavenCommandBuilder;

/// Check if Maven is available and return version information
///
/// # Examples
///
/// ```no_run
/// use maven_command_builder::check_maven_availability;
/// use std::path::Path;
///
/// match check_maven_availability(Path::new("/project")) {
///     Ok(version) => println!("Maven version: {}", version),
///     Err(e) => eprintln!("Maven not found: {}", e),
/// }
/// ```
pub fn check_maven_availability(project_root: &Path) -> Result<String, std::io::Error> {
    let builder = MavenCommandBuilder::new(project_root);
    let maven_command = builder.get_maven_executable();

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

/// Execute a Maven command synchronously and return all output lines
///
/// # Examples
///
/// ```no_run
/// use maven_command_builder::{MavenCommandBuilder, execute_maven_command};
/// use std::path::Path;
///
/// let builder = MavenCommandBuilder::new(Path::new("/project"))
///     .goal("clean")
///     .goal("compile");
///
/// match execute_maven_command(&builder) {
///     Ok(output) => {
///         for line in output {
///             println!("{}", line);
///         }
///     }
///     Err(e) => eprintln!("Build failed: {}", e),
/// }
/// ```
pub fn execute_maven_command(builder: &MavenCommandBuilder) -> Result<Vec<String>, std::io::Error> {
    let maven_command = builder.get_maven_executable();
    let args = builder.build_args();
    let project_root = builder.project_root();

    let mut command = Command::new(maven_command);
    command.args(&args).current_dir(project_root);

    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.ok_or_else(|| {
        std::io::Error::other("Failed to capture stdout")
    })?;

    let reader = BufReader::new(stdout);
    let mut output_lines = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        output_lines.push(line);
    }

    Ok(output_lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_maven_availability_returns_result() {
        // This test just ensures the function signature is correct
        // Actual execution would require Maven installed
        let result = check_maven_availability(Path::new("."));
        // Don't assert on result as Maven may not be installed in test environment
        let _ = result;
    }

    #[test]
    fn test_execute_maven_command_accepts_builder() {
        let builder = MavenCommandBuilder::new(Path::new(".")).goal("--version");
        
        // Just test that the function accepts the builder
        // Actual execution would require Maven installed
        let result = execute_maven_command(&builder);
        let _ = result;
    }
}
