# maven-java-agent

A Rust library for managing Java agents in Maven-based applications.

## Features

- 🚀 **Automatic agent building** - Compiles Java agent from source during build
- 📦 **Agent deployment** - Copies agent JAR to runtime locations
- ⚙️ **JVM configuration** - Generates `-javaagent` arguments automatically
- 🌍 **Environment setup** - Manages `JAVA_TOOL_OPTIONS` and other env vars
- 🪵 **Log4j support** - Special handling for Log4j reconfiguration agent

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
maven-java-agent = { path = "../maven-java-agent" }
```

### Basic Example

```rust
use maven_java_agent::AgentBuilder;

// Configure and build agent deployment
let deployment = AgentBuilder::new()
    .with_log4j_config("file:///tmp/lazymvn/log4j.properties")
    .enable_reconfig(true)
    .build()?;

// Use in Maven command
let mut command = Command::new("mvn");
for arg in &deployment.jvm_args {
    command.arg(arg);
}
for (key, val) in &deployment.env_vars {
    command.env(key, val);
}
```

### Advanced Example

```rust
use maven_java_agent::AgentBuilder;
use std::path::Path;

// Build and deploy to specific directory
let deployment = AgentBuilder::new()
    .with_log4j_config("file:///tmp/lazymvn/log4j.properties")
    .with_jvm_option("-Dlog4j.ignoreTCL=true")
    .with_jvm_option("-Dlog4j.defaultInitOverride=true")
    .enable_reconfig(true)
    .build_and_deploy(Path::new("/tmp/agents"))?;

println!("Agent deployed to: {}", deployment.agent_jar_path.display());
```

## How It Works

### Agent Location

The library looks for the agent JAR in the following locations (in order):

1. `agent/target/log4j-reconfig-agent-0.1.0.jar` (development)
2. Next to the executable (production)
3. `~/.cache/lazymvn/log4j-reconfig-agent.jar` (cached copy)

### Log4j Reconfiguration

The Java agent provided with this library solves a common problem: applications with
custom Log4j factories (like `Log4jJbossLoggerFactory`) that reinitialize Log4j with
their own configuration, overwriting your configuration.

**Strategy:**
1. JVM starts with `JAVA_TOOL_OPTIONS` → Log4j loads your config ✓
2. Application starts → Custom factory reloads its config ✗ (overwrites yours)
3. Our agent waits 2 seconds → Forces reconfiguration with your config ✓

## API Documentation

### `AgentBuilder`

Fluent builder for configuring and deploying Java agents.

**Methods:**
- `new()` - Create a new builder
- `with_log4j_config(url)` - Set Log4j configuration URL
- `with_jvm_option(option)` - Add a JVM option
- `enable_reconfig(bool)` - Enable/disable reconfiguration agent
- `build()` - Build deployment using existing agent
- `build_and_deploy(path)` - Build and copy agent to path

### `AgentDeployment`

Contains deployment information for a configured agent.

**Fields:**
- `agent_jar_path: PathBuf` - Path to the agent JAR
- `jvm_args: Vec<String>` - JVM arguments to add
- `env_vars: HashMap<String, String>` - Environment variables to set

### `AgentConfig`

Configuration for a Java agent.

**Fields:**
- `log4j_config_url: Option<String>` - Log4j config URL
- `jvm_options: Vec<String>` - Additional JVM options
- `enable_reconfig: bool` - Enable reconfiguration agent

## Building

The library automatically builds the Java agent during the Rust build process.
Requires:
- Maven (`mvn` command)
- Java 8 or later

```bash
cargo build
```

## Testing

```bash
cargo test
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
