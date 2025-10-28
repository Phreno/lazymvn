# LazyMVN Configuration Examples

This directory contains example configuration files for LazyMVN.

## Available Examples

### Basic Configuration

- **[lazymvn.toml.example](lazymvn.toml.example)** - Complete example with all features
  - Maven settings path
  - Launch mode selection
  - Notifications
  - Watch mode
  - Output buffer configuration
  - Logging configuration

### Specialized Configurations

- **[lazymvn.toml.spring-boot-example](lazymvn.toml.spring-boot-example)** - Spring Boot specific settings
  - Optimized for Spring Boot projects
  - spring-boot:run launch mode
  - Common Spring Boot logging configuration

- **[lazymvn.toml.spring-properties-example](lazymvn.toml.spring-properties-example)** - Spring Boot properties override
  - Override application.properties without modifying source
  - Environment switching (local/staging/production)
  - Database configuration overrides
  - Strong override priority (LazyMVN has the last word)

- **[lazymvn.toml.watch-example](lazymvn.toml.watch-example)** - File watching configuration
  - Auto-reload on file changes
  - Configurable file patterns
  - Debounce settings
  - Command triggers

- **[lazymvn.toml.logging-example](lazymvn.toml.logging-example)** - Logging configuration
  - Package-level log filtering
  - Multiple logging frameworks (log4j, SLF4J)
  - JVM arguments for logging control

- **[lazymvn.toml.loglevel-example](lazymvn.toml.loglevel-example)** - Log level examples
  - Different log levels (ERROR, WARN, INFO, DEBUG, TRACE)
  - Per-package configuration

- **[lazymvn.toml.custom-flags-example](lazymvn.toml.custom-flags-example)** - Custom Maven flags
  - Define project-specific Maven arguments
  - Custom -D properties and flags
  - Toggle custom flags in the Flags panel
  - Enable flags by default

## Using These Examples

### Copy to Your Project

```bash
# Copy the basic example
cp examples/lazymvn.toml.example lazymvn.toml

# Or copy a specialized example
cp examples/lazymvn.toml.spring-boot-example lazymvn.toml
```

### Customize

Edit `lazymvn.toml` in your project root to match your needs.

### Live Reload

LazyMVN watches for changes to `lazymvn.toml` and reloads automatically.
You can also manually reload with `Ctrl+E` (opens editor) or `Ctrl+R` (reload).

## Configuration Sections

### Maven Settings

```toml
maven_settings = "path/to/settings.xml"
```

### Launch Mode

```toml
launch_mode = "spring-boot-run"  # or "exec-java"
```

### Notifications

```toml
notifications_enabled = true
```

### Watch Mode

```toml
[watch]
enabled = true
commands = ["test", "start"]
patterns = ["src/**/*.java", "src/**/*.xml"]
debounce_ms = 500
```

### Output Buffer

```toml
[output]
max_lines = 10000
max_updates_per_poll = 100
```

### Logging

```toml
[logging]
packages = [
  { name = "com.example", level = "DEBUG" },
  { name = "org.springframework", level = "WARN" },
]
```

### Custom Maven Flags

```toml
[maven]
custom_flags = [
  { name = "Enable feature X", flag = "-Dfeature.x=true" },
  { name = "Development mode", flag = "-Dspring.profiles.active=dev", enabled = true },
  { name = "Skip integration tests", flag = "-DskipITs=true" },
]
```

## More Information

- See [../docs/LOGGING_CONFIG.md](../docs/LOGGING_CONFIG.md) for logging details
- See [../docs/LIVE_CONFIG_RELOAD.md](../docs/LIVE_CONFIG_RELOAD.md) for reload info
- See [../README.md](../README.md) for general usage

## Setup Wizard

Generate a config file interactively:

```bash
lazymvn --setup
```

This creates a `lazymvn.toml` in your current directory with sensible defaults.
