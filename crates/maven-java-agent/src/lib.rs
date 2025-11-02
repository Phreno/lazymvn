//! Maven Java Agent Library
//!
//! This library provides functionality for managing Java agents in Maven-based applications.
//! It handles the building, deployment, and configuration of Java agents, with special
//! support for Log4j reconfiguration.
//!
//! # Features
//!
//! - **Agent Building**: Automatic compilation of Java agent from source
//! - **Agent Deployment**: Copy agent JAR to runtime locations
//! - **JVM Configuration**: Generate `-javaagent` arguments
//! - **Environment Setup**: Manage `JAVA_TOOL_OPTIONS` and other env vars
//! - **Log4j Support**: Special handling for Log4j reconfiguration agent
//!
//! # Example
//!
//! ```rust,no_run
//! use maven_java_agent::AgentBuilder;
//!
//! let deployment = AgentBuilder::new()
//!     .with_log4j_config("file:///tmp/lazymvn/log4j.properties")
//!     .enable_reconfig(true)
//!     .build()?;
//!
//! // Use deployment in Maven command
//! for arg in &deployment.jvm_args {
//!     println!("JVM arg: {}", arg);
//! }
//! for (key, val) in &deployment.env_vars {
//!     println!("ENV {}: {}", key, val);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod builder;
mod config;
mod deployment;
mod error;

pub use builder::AgentBuilder;
pub use config::{AgentConfig, AgentDeployment};
pub use deployment::{deploy_agent, get_agent_path};
pub use error::{AgentError, Result};
