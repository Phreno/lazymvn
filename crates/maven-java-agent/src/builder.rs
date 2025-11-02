//! Fluent builder API for configuring and deploying Java agents

use crate::config::{AgentConfig, AgentDeployment};
use crate::deployment::{deploy_agent, get_agent_path};
use crate::error::Result;

/// Builder for configuring and deploying Java agents
///
/// # Example
///
/// ```rust,no_run
/// use maven_java_agent::AgentBuilder;
///
/// let deployment = AgentBuilder::new()
///     .with_log4j_config("file:///tmp/lazymvn/log4j.properties")
///     .with_jvm_option("-Dlog4j.ignoreTCL=true")
///     .with_jvm_option("-Dlog4j.defaultInitOverride=true")
///     .enable_reconfig(true)
///     .build()?;
///
/// println!("Agent path: {}", deployment.agent_jar_path.display());
/// for arg in &deployment.jvm_args {
///     println!("JVM arg: {}", arg);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Default)]
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new() -> Self {
        Self {
            config: AgentConfig::new(),
        }
    }
    
    /// Set Log4j configuration URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use maven_java_agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new()
    ///     .with_log4j_config("file:///tmp/lazymvn/log4j.properties");
    /// ```
    pub fn with_log4j_config(mut self, url: impl Into<String>) -> Self {
        self.config.log4j_config_url = Some(url.into());
        self
    }
    
    /// Add a JVM option
    ///
    /// # Example
    ///
    /// ```rust
    /// use maven_java_agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new()
    ///     .with_jvm_option("-Dlog4j.ignoreTCL=true")
    ///     .with_jvm_option("-Dlog4j.defaultInitOverride=true");
    /// ```
    pub fn with_jvm_option(mut self, option: impl Into<String>) -> Self {
        self.config.jvm_options.push(option.into());
        self
    }
    
    /// Enable or disable Log4j reconfiguration agent
    ///
    /// When enabled, the agent will periodically reconfigure Log4j to
    /// override any application-specific Log4j initialization.
    ///
    /// # Example
    ///
    /// ```rust
    /// use maven_java_agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new()
    ///     .enable_reconfig(true);
    /// ```
    pub fn enable_reconfig(mut self, enable: bool) -> Self {
        self.config.enable_reconfig = enable;
        self
    }
    
    /// Build the agent deployment
    ///
    /// This will locate or deploy the agent JAR and generate the necessary
    /// JVM arguments and environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The agent JAR cannot be found
    /// - The deployment location is not writable
    /// - The configuration is invalid
    pub fn build(self) -> Result<AgentDeployment> {
        let agent_path = get_agent_path()?;
        let mut deployment = AgentDeployment::new(agent_path);
        
        // Add -javaagent argument if reconfiguration is enabled
        if self.config.enable_reconfig {
            deployment.add_jvm_arg(deployment.javaagent_arg());
        }
        
        // Add custom JVM options
        for option in &self.config.jvm_options {
            deployment.add_jvm_arg(option);
        }
        
        // Build JAVA_TOOL_OPTIONS if Log4j config is provided
        if let Some(log4j_url) = &self.config.log4j_config_url {
            let mut java_tool_options = Vec::new();
            
            // Add Log4j configuration options
            java_tool_options.push("-Dlog4j.ignoreTCL=true".to_string());
            java_tool_options.push("-Dlog4j.defaultInitOverride=true".to_string());
            java_tool_options.push(format!("-Dlog4j.configuration={}", log4j_url));
            
            // Combine into single JAVA_TOOL_OPTIONS string
            let opts_str = java_tool_options.join(" ");
            deployment.add_env_var("JAVA_TOOL_OPTIONS", opts_str);
        }
        
        Ok(deployment)
    }
    
    /// Build and deploy the agent to a specific directory
    ///
    /// This is useful when you need to ensure the agent is copied to a
    /// specific location rather than using the original JAR location.
    pub fn build_and_deploy(self, target_dir: &std::path::Path) -> Result<AgentDeployment> {
        let deployed_path = deploy_agent(target_dir)?;
        let mut deployment = AgentDeployment::new(deployed_path);
        
        // Same logic as build()
        if self.config.enable_reconfig {
            let javaagent_arg = deployment.javaagent_arg();
            deployment.add_jvm_arg(javaagent_arg);
        }
        
        for option in &self.config.jvm_options {
            deployment.add_jvm_arg(option);
        }
        
        if let Some(log4j_url) = &self.config.log4j_config_url {
            let mut java_tool_options = Vec::new();
            java_tool_options.push("-Dlog4j.ignoreTCL=true".to_string());
            java_tool_options.push("-Dlog4j.defaultInitOverride=true".to_string());
            java_tool_options.push(format!("-Dlog4j.configuration={}", log4j_url));
            
            let opts_str = java_tool_options.join(" ");
            deployment.add_env_var("JAVA_TOOL_OPTIONS", opts_str);
        }
        
        Ok(deployment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder_basic() {
        let builder = AgentBuilder::new()
            .with_log4j_config("file:///tmp/config.properties")
            .enable_reconfig(true);
        
        assert_eq!(
            builder.config.log4j_config_url,
            Some("file:///tmp/config.properties".to_string())
        );
        assert!(builder.config.enable_reconfig);
    }
    
    #[test]
    fn test_builder_jvm_options() {
        let builder = AgentBuilder::new()
            .with_jvm_option("-Dtest=true")
            .with_jvm_option("-Xmx512m");
        
        assert_eq!(builder.config.jvm_options.len(), 2);
        assert_eq!(builder.config.jvm_options[0], "-Dtest=true");
        assert_eq!(builder.config.jvm_options[1], "-Xmx512m");
    }
}
