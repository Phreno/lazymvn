//! # Maven Command Builder
//!
//! A fluent API for building and executing Maven commands in Rust.
//!
//! ## Features
//!
//! - 🔧 **Fluent Builder API** - Chainable methods for intuitive command construction
//! - 🎯 **Type-Safe** - Compile-time guarantees for valid Maven commands
//! - 📦 **Auto-Detection** - Automatically detects `mvnw` wrapper or system `mvn`
//! - 🚀 **Execution Support** - Built-in synchronous command execution
//! - ⚙️ **Rich Options** - Profiles, properties, flags, modules, and more
//! - 🧪 **Well-Tested** - Comprehensive test coverage
//!
//! ## Quick Start
//!
//! ```
//! use maven_command_builder::MavenCommandBuilder;
//! use std::path::Path;
//!
//! // Build a simple Maven command
//! let cmd = MavenCommandBuilder::new(Path::new("/path/to/project"))
//!     .goal("clean")
//!     .goal("install")
//!     .skip_tests(true)
//!     .build();
//!
//! println!("Command: {}", cmd);
//! // Output: mvn clean install -DskipTests
//! ```
//!
//! ## Advanced Usage
//!
//! ### Multi-Module Projects
//!
//! ```
//! use maven_command_builder::MavenCommandBuilder;
//! use std::path::Path;
//!
//! let cmd = MavenCommandBuilder::new(Path::new("/project"))
//!     .goal("install")
//!     .module("backend")
//!     .also_make(true)
//!     .build();
//! ```
//!
//! ### With Profiles and Properties
//!
//! ```
//! use maven_command_builder::MavenCommandBuilder;
//! use std::path::Path;
//!
//! let cmd = MavenCommandBuilder::new(Path::new("/project"))
//!     .goal("package")
//!     .profile("production")
//!     .property("env", "prod")
//!     .property("log.level", "INFO")
//!     .threads("2C")
//!     .offline(true)
//!     .build();
//! ```
//!
//! ### Command Execution
//!
//! ```no_run
//! use maven_command_builder::{MavenCommandBuilder, execute_maven_command};
//! use std::path::Path;
//!
//! let builder = MavenCommandBuilder::new(Path::new("/project"))
//!     .goal("clean")
//!     .goal("compile");
//!
//! match execute_maven_command(&builder) {
//!     Ok(output) => {
//!         for line in output {
//!             println!("{}", line);
//!         }
//!     }
//!     Err(e) => eprintln!("Build failed: {}", e),
//! }
//! ```
//!
//! ## Common Patterns
//!
//! ### Spring Boot Application
//!
//! ```
//! use maven_command_builder::MavenCommandBuilder;
//! use std::path::Path;
//!
//! let cmd = MavenCommandBuilder::new(Path::new("/project"))
//!     .goal("spring-boot:run")
//!     .profile("dev")
//!     .property("spring.profiles.active", "development")
//!     .build();
//! ```
//!
//! ### Fast Build (Skip Tests and Documentation)
//!
//! ```
//! use maven_command_builder::MavenCommandBuilder;
//! use std::path::Path;
//!
//! let cmd = MavenCommandBuilder::new(Path::new("/project"))
//!     .goal("clean")
//!     .goal("install")
//!     .skip_tests(true)
//!     .property("maven.javadoc.skip", "true")
//!     .threads("1C")
//!     .build();
//! ```
//!
//! ### Release Build
//!
//! ```
//! use maven_command_builder::MavenCommandBuilder;
//! use std::path::Path;
//!
//! let cmd = MavenCommandBuilder::new(Path::new("/project"))
//!     .goal("clean")
//!     .goal("deploy")
//!     .profile("release")
//!     .update_snapshots(true)
//!     .threads("2C")
//!     .build();
//! ```

pub mod builder;
pub mod executor;

// Re-export main types
pub use builder::MavenCommandBuilder;
pub use executor::{check_maven_availability, execute_maven_command};
