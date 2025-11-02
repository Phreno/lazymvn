//! Agent deployment and path management

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AgentError, Result};

/// Get the path to the agent JAR file
///
/// This function looks for the agent JAR in the following locations:
/// 1. `target/log4j-reconfig-agent.jar` (development)
/// 2. Next to the executable (production)
/// 3. In `~/.cache/lazymvn/` (cached copy)
pub fn get_agent_path() -> Result<PathBuf> {
    // Try development location first
    let dev_path = PathBuf::from("agent/target/log4j-reconfig-agent-0.1.0.jar");
    if dev_path.exists() {
        return Ok(dev_path);
    }
    
    // Try next to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let prod_path = exe_dir.join("log4j-reconfig-agent.jar");
            if prod_path.exists() {
                return Ok(prod_path);
            }
        }
    }
    
    // Try cache location
    if let Some(home) = dirs::home_dir() {
        let cache_path = home.join(".cache/lazymvn/log4j-reconfig-agent.jar");
        if cache_path.exists() {
            return Ok(cache_path);
        }
    }
    
    Err(AgentError::AgentNotFound(PathBuf::from(
        "log4j-reconfig-agent.jar",
    )))
}

/// Deploy agent JAR to a runtime location
///
/// Copies the agent JAR to a location where it can be used by the JVM.
/// Returns the path to the deployed agent.
pub fn deploy_agent(target_dir: &Path) -> Result<PathBuf> {
    let agent_path = get_agent_path()?;
    
    // Create target directory if it doesn't exist
    fs::create_dir_all(target_dir)
        .map_err(AgentError::DeploymentFailed)?;
    
    // Copy agent to target directory
    let target_path = target_dir.join("log4j-reconfig-agent.jar");
    fs::copy(&agent_path, &target_path)
        .map_err(AgentError::DeploymentFailed)?;
    
    Ok(target_path)
}

/// Convert a file path to a file:// URL
pub(crate) fn path_to_file_url(path: &Path) -> String {
    // Normalize path separators for URLs
    let path_str = path.to_string_lossy().replace('\\', "/");
    
    // Add file:// prefix if not present
    if path_str.starts_with("file://") {
        path_str.to_string()
    } else if path_str.starts_with('/') {
        format!("file://{}", path_str)
    } else {
        format!("file:///{}", path_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_to_file_url_unix() {
        let path = Path::new("/tmp/config.properties");
        let url = path_to_file_url(path);
        assert_eq!(url, "file:///tmp/config.properties");
    }
    
    #[test]
    fn test_path_to_file_url_relative() {
        let path = Path::new("config.properties");
        let url = path_to_file_url(path);
        assert_eq!(url, "file:///config.properties");
    }
    
    #[test]
    fn test_path_to_file_url_already_url() {
        let path_str = "file:///tmp/config.properties";
        let path = Path::new(path_str);
        let url = path_to_file_url(path);
        assert_eq!(url, "file:///tmp/config.properties");
    }
}
