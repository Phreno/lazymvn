# Java Agent Library Extraction Plan

## Overview

Extract Java agent functionality into a dedicated Rust crate `maven-java-agent` to provide reusable Java agent management capabilities for Maven-based applications.

## Motivation

The Java agent code is currently scattered across multiple locations:
- Java source: `agent/src/main/java/io/github/phreno/lazymvn/agent/Log4jReconfigAgent.java`
- Build configuration: `agent/pom.xml`, `build.rs`
- Agent management: `src/ui/state/launcher_config.rs` (lines 21-27, 72-105)
- Environment setup: `src/maven/command/executor.rs` (lines 67-81)

Extracting this into a library would:
1. **Encapsulate complexity** - Hide Java agent build and deployment details
2. **Enable reusability** - Other projects could use the agent management
3. **Improve maintainability** - Clear separation of concerns
4. **Better testing** - Isolated unit tests for agent functionality
5. **Consistent with architecture** - Follows pattern of log-analyzer, log-colorizer, command-builder

## Current Architecture

```
lazymvn/
├── agent/                          # Java agent source
│   ├── pom.xml                    # Maven build for agent JAR
│   └── src/main/java/...          # Log4jReconfigAgent.java
├── build.rs                        # Builds agent during cargo build
└── src/
    ├── maven/command/executor.rs  # Sets JAVA_TOOL_OPTIONS
    └── ui/state/launcher_config.rs # Injects -javaagent flag
```

## Proposed Architecture

```
lazymvn/
├── crates/
│   ├── maven-java-agent/          # NEW LIBRARY
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   ├── build.rs               # Builds agent JAR
│   │   ├── agent/                 # Java agent source (moved)
│   │   │   ├── pom.xml
│   │   │   └── src/main/java/... 
│   │   └── src/
│   │       ├── lib.rs             # Public API
│   │       ├── builder.rs         # Agent path management
│   │       ├── config.rs          # Agent configuration
│   │       └── deployment.rs      # Copy agent to runtime location
│   ├── maven-log-analyzer/
│   ├── maven-log-colorizer/
│   └── maven-command-builder/
└── src/
    ├── maven/command/executor.rs  # Uses maven-java-agent
    └── ui/state/launcher_config.rs # Uses maven-java-agent
```

## Library API Design

### Core Types

```rust
// config.rs
pub struct AgentConfig {
    /// Log4j configuration URL (file:// or classpath:)
    pub log4j_config_url: Option<String>,
    /// Additional JVM options to set
    pub jvm_options: Vec<String>,
    /// Whether to enable reconfiguration agent
    pub enable_reconfig: bool,
}

pub struct AgentDeployment {
    /// Path to deployed agent JAR
    pub agent_jar_path: PathBuf,
    /// JVM arguments to inject agent
    pub jvm_args: Vec<String>,
    /// Environment variables (JAVA_TOOL_OPTIONS)
    pub env_vars: HashMap<String, String>,
}
```

### Public API

```rust
// lib.rs
pub use config::{AgentConfig, AgentDeployment};
pub use builder::AgentBuilder;
pub use deployment::{deploy_agent, get_agent_path};

/// Builder for configuring and deploying Java agent
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    pub fn new() -> Self { /* ... */ }
    
    /// Set Log4j configuration URL
    pub fn with_log4j_config(mut self, url: impl Into<String>) -> Self { /* ... */ }
    
    /// Add JVM option
    pub fn with_jvm_option(mut self, option: impl Into<String>) -> Self { /* ... */ }
    
    /// Enable/disable reconfiguration agent
    pub fn enable_reconfig(mut self, enable: bool) -> Self { /* ... */ }
    
    /// Build deployment configuration
    pub fn build(self) -> Result<AgentDeployment, AgentError> { /* ... */ }
}
```

### Usage Example

```rust
use maven_java_agent::{AgentBuilder, deploy_agent};

// 1. Configure agent
let deployment = AgentBuilder::new()
    .with_log4j_config("file:///tmp/lazymvn/log4j.properties")
    .with_jvm_option("-Dlog4j.ignoreTCL=true")
    .with_jvm_option("-Dlog4j.defaultInitOverride=true")
    .enable_reconfig(true)
    .build()?;

// 2. Use in command
let mut command = Command::new("mvn");
for arg in &deployment.jvm_args {
    command.arg(arg);
}
for (key, val) in &deployment.env_vars {
    command.env(key, val);
}

// 3. Or get raw components
let agent_path = deployment.agent_jar_path;
let java_tool_options = deployment.env_vars.get("JAVA_TOOL_OPTIONS");
```

## Implementation Plan

### Phase 1: Extract Core Functionality (1-2 days)

1. **Create library structure**
   ```bash
   mkdir -p crates/maven-java-agent/src
   mkdir -p crates/maven-java-agent/agent
   ```

2. **Move Java agent**
   ```bash
   mv agent/* crates/maven-java-agent/agent/
   ```

3. **Create Cargo.toml**
   ```toml
   [package]
   name = "maven-java-agent"
   version = "0.1.0"
   edition = "2024"
   
   [dependencies]
   # None needed for now
   
   [build-dependencies]
   # For building agent JAR
   ```

4. **Implement core types**
   - `src/config.rs` - AgentConfig, AgentDeployment
   - `src/error.rs` - AgentError types
   - `src/lib.rs` - Public API

### Phase 2: Agent Management (1-2 days)

1. **Implement builder.rs**
   - Agent path detection
   - JAR deployment to temp/cache location
   - Path to file:// URL conversion

2. **Implement deployment.rs**
   - Copy agent JAR to runtime location
   - Generate JVM arguments
   - Generate environment variables

3. **Move build logic**
   - Move agent build from root `build.rs` to library `build.rs`
   - Ensure agent JAR is included in library resources

### Phase 3: Integration (1 day)

1. **Update workspace**
   ```toml
   # Cargo.toml
   [workspace]
   members = [
       ".",
       "crates/maven-log-analyzer",
       "crates/maven-log-colorizer", 
       "crates/maven-command-builder",
       "crates/maven-java-agent"  # NEW
   ]
   ```

2. **Add dependency**
   ```toml
   # Cargo.toml
   [dependencies]
   maven-java-agent = { path = "crates/maven-java-agent" }
   ```

3. **Refactor launcher_config.rs**
   - Replace `get_or_copy_log4j_agent()` with `AgentBuilder`
   - Replace manual javaagent arg building

4. **Refactor executor.rs**
   - Replace manual JAVA_TOOL_OPTIONS building with `AgentDeployment`

### Phase 4: Documentation & Testing (1 day)

1. **Create README.md**
   - Library purpose and features
   - API documentation
   - Usage examples
   - Integration guide

2. **Add tests**
   - Unit tests for builder
   - Unit tests for deployment
   - Integration tests

3. **Update main docs**
   - Add to library extraction status
   - Document Java agent in user guide

## Benefits

### For LazyMVN

1. **Cleaner codebase**
   - Agent logic isolated in one place
   - Clear API boundaries
   - Less coupling between components

2. **Easier maintenance**
   - Single place to update agent functionality
   - Isolated testing
   - Clear ownership

3. **Better debugging**
   - Centralized logging for agent operations
   - Clear error messages
   - Easier to trace issues

### For External Users

1. **Reusable functionality**
   - Other TUI/CLI tools can use the agent
   - Maven-based projects can integrate agent management
   - Useful for any tool that needs Log4j control

2. **Documentation**
   - Clear API documentation
   - Examples and guides
   - Integration patterns

3. **Stability**
   - Versioned and tested
   - Clear compatibility guarantees
   - Regular updates

## Migration Strategy

### Backward Compatibility

During migration, maintain backward compatibility:
1. Keep old code paths working
2. Add deprecation warnings
3. Provide migration guide
4. Remove old code in next major version

### Gradual Rollout

1. **Week 1**: Create library structure, move Java code
2. **Week 2**: Implement API, add tests
3. **Week 3**: Integrate with LazyMVN, parallel implementation
4. **Week 4**: Switch to new library, remove old code
5. **Week 5**: Documentation and polish

## Success Criteria

- [ ] Agent JAR builds automatically with library
- [ ] Agent deploys correctly to runtime location
- [ ] API is simple and intuitive
- [ ] All tests pass
- [ ] Documentation is complete
- [ ] LazyMVN uses library exclusively
- [ ] No regression in functionality
- [ ] Build time doesn't increase significantly

## Future Enhancements

1. **Multiple agent support**
   - Support for multiple Java agents
   - Agent ordering/priority

2. **Agent discovery**
   - Automatic detection of available agents
   - Dynamic agent loading

3. **Configuration templates**
   - Pre-built configurations for common scenarios
   - Framework-specific presets (Spring Boot, Quarkus, etc.)

4. **Advanced features**
   - Agent hot-reload support
   - Runtime agent attachment
   - Agent state inspection

## Related Documentation

- [LIBRARY_EXTRACTION_PLAN.md](./LIBRARY_EXTRACTION_PLAN.md) - Overall extraction strategy
- [LIBRARY_STATUS.md](./LIBRARY_STATUS.md) - Current extraction status
- [Log4j Implementation Docs](./LOG4J_FINAL_SOLUTION.md) - Why agent is needed

## Questions & Decisions

### Q1: Should the library manage Maven command building?

**Decision**: No. Keep focused on Java agent management only. Use `maven-command-builder` for command construction.

### Q2: Include agent JAR in binary or download on-demand?

**Decision**: Include in binary for reliability and offline usage. Size impact is minimal (~10KB).

### Q3: Support other logging frameworks?

**Decision**: Start with Log4j 1.x only. Add Logback/SLF4J support in future if needed.

### Q4: Should library handle temp file cleanup?

**Decision**: Yes, provide cleanup methods but also handle cleanup automatically on Drop.

## Timeline

- **Total estimated time**: 5-6 days
- **Priority**: Medium (not blocking, but valuable)
- **Complexity**: Medium
- **Risk**: Low (agent already works, just refactoring)

## Next Steps

1. Get feedback on API design
2. Create feature branch
3. Implement Phase 1
4. Review and iterate
5. Complete remaining phases

---

*Created: 2025-11-02*
*Status: Proposal*
*Owner: To be assigned*
