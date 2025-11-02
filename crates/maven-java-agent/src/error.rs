//! Error types for the maven-java-agent library

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type alias for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;

/// Errors that can occur during agent operations
#[derive(Debug)]
pub enum AgentError {
    /// Agent JAR file not found
    AgentNotFound(PathBuf),
    
    /// Failed to copy agent to deployment location
    DeploymentFailed(io::Error),
    
    /// Invalid configuration
    InvalidConfig(String),
    
    /// IO error
    Io(io::Error),
    
    /// Agent build failed
    BuildFailed(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::AgentNotFound(path) => {
                write!(f, "Agent JAR not found at: {}", path.display())
            }
            AgentError::DeploymentFailed(err) => {
                write!(f, "Failed to deploy agent: {}", err)
            }
            AgentError::InvalidConfig(msg) => {
                write!(f, "Invalid agent configuration: {}", msg)
            }
            AgentError::Io(err) => {
                write!(f, "IO error: {}", err)
            }
            AgentError::BuildFailed(msg) => {
                write!(f, "Agent build failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for AgentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AgentError::DeploymentFailed(err) | AgentError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for AgentError {
    fn from(err: io::Error) -> Self {
        AgentError::Io(err)
    }
}
