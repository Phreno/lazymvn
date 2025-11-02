//! Configuration types for Java agent deployment

use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for Java agent
#[derive(Debug, Clone, Default)]
pub struct AgentConfig {
    /// Log4j configuration URL (file:// or classpath:)
    pub log4j_config_url: Option<String>,
    
    /// Additional JVM options to set
    pub jvm_options: Vec<String>,
    
    /// Whether to enable Log4j reconfiguration agent
    pub enable_reconfig: bool,
}

impl AgentConfig {
    /// Create a new agent configuration
    pub fn new() -> Self {
        Self::default()
    }
}

/// Deployment information for a Java agent
#[derive(Debug, Clone)]
pub struct AgentDeployment {
    /// Path to the deployed agent JAR file
    pub agent_jar_path: PathBuf,
    
    /// JVM arguments to inject the agent
    /// Includes -javaagent flag and related options
    pub jvm_args: Vec<String>,
    
    /// Environment variables to set
    /// Typically includes JAVA_TOOL_OPTIONS
    pub env_vars: HashMap<String, String>,
}

impl AgentDeployment {
    /// Create a new agent deployment
    pub fn new(agent_jar_path: PathBuf) -> Self {
        Self {
            agent_jar_path,
            jvm_args: Vec::new(),
            env_vars: HashMap::new(),
        }
    }
    
    /// Add a JVM argument
    pub fn add_jvm_arg(&mut self, arg: impl Into<String>) {
        self.jvm_args.push(arg.into());
    }
    
    /// Add an environment variable
    pub fn add_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env_vars.insert(key.into(), value.into());
    }
    
    /// Get the -javaagent argument string
    pub fn javaagent_arg(&self) -> String {
        format!("-javaagent:{}", self.agent_jar_path.display())
    }
}
