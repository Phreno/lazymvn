# Custom Maven Flags

LazyMVN allows you to define custom Maven flags in your configuration file that will appear in the Flags panel alongside the built-in flags. This is useful for project-specific Maven arguments, custom `-D` properties, or any other Maven command-line options.

## Overview

- **Define once**: Configure custom flags in `lazymvn.toml`
- **Toggle easily**: Custom flags appear in the Flags panel (press `f`)
- **Persistent**: Flag states are saved per module
- **Live reload**: Changes to configuration are applied immediately with `Ctrl+E`

## Configuration

Add a `[maven]` section to your `lazymvn.toml`:

```toml
[maven]
custom_flags = [
    { name = "Display name", flag = "Maven argument" },
    { name = "Enabled by default", flag = "Maven argument", enabled = true },
]
```

### Field Reference

Each custom flag has the following fields:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name shown in the Flags panel |
| `flag` | Yes | The actual Maven argument(s) to pass |
| `enabled` | No | Whether the flag is enabled by default (default: `false`) |

## Examples

### Basic Custom Properties

```toml
[maven]
custom_flags = [
    # Simple property definition
    { name = "Enable feature X", flag = "-Dfeature.x.enabled=true" },
    
    # Property with value
    { name = "Custom database URL", flag = "-Ddb.url=jdbc:postgresql://localhost:5432/mydb" },
    
    # Multiple properties in one flag
    { name = "Fast build", flag = "-Dmaven.test.skip=true -Dmaven.javadoc.skip=true" },
]
```

### Environment Configuration

```toml
[maven]
custom_flags = [
    # Development environment (enabled by default)
    { name = "Development mode", flag = "-Dspring.profiles.active=dev", enabled = true },
    
    # Staging environment
    { name = "Staging mode", flag = "-Dspring.profiles.active=staging" },
    
    # Production environment
    { name = "Production mode", flag = "-Dspring.profiles.active=prod" },
]
```

### Test Configuration

```toml
[maven]
custom_flags = [
    # Skip different types of tests
    { name = "Skip integration tests", flag = "-DskipITs=true" },
    { name = "Skip unit tests", flag = "-DskipTests=true" },
    
    # Run specific test
    { name = "Run MyTest only", flag = "-Dtest=MyTest" },
    
    # Test groups
    { name = "Run unit tests", flag = "-Dgroups=unit" },
    { name = "Run integration tests", flag = "-Dgroups=integration" },
]
```

### Build Optimization

```toml
[maven]
custom_flags = [
    # Skip documentation
    { name = "Skip JavaDoc", flag = "-Dmaven.javadoc.skip=true" },
    
    # Skip code quality checks
    { name = "Skip Checkstyle", flag = "-Dcheckstyle.skip=true" },
    { name = "Skip PMD", flag = "-Dpmd.skip=true" },
    { name = "Skip SpotBugs", flag = "-Dspotbugs.skip=true" },
    
    # Fast build (combination)
    { name = "Ultra-fast build", flag = "-DskipTests -Dmaven.javadoc.skip=true -Dcheckstyle.skip=true" },
]
```

### Debug and Logging

```toml
[maven]
custom_flags = [
    # Enable debug mode
    { name = "Debug mode", flag = "-Ddebug=true" },
    
    # Verbose logging
    { name = "Verbose SQL", flag = "-Dhibernate.show_sql=true -Dhibernate.format_sql=true" },
    
    # Log level override
    { name = "Debug logging", flag = "-Dlogging.level.root=DEBUG" },
]
```

### Feature Toggles

```toml
[maven]
custom_flags = [
    # Enable/disable features
    { name = "New API enabled", flag = "-Dfeature.new-api=true" },
    { name = "Experimental features", flag = "-Dexperimental.enabled=true" },
    { name = "Beta features", flag = "-Dbeta.features=true" },
    
    # Legacy mode
    { name = "Legacy compatibility", flag = "-Dlegacy.mode=true" },
]
```

## Usage

### In the TUI

1. **View flags**: Press `f` to open the Flags panel
2. **Navigate**: Use `↑`/`↓` arrow keys or `j`/`k`
3. **Toggle**: Press `Space` to enable/disable a flag
4. **Apply**: Run any Maven command (compile, test, etc.)

Custom flags appear after the built-in flags in the list.

### Flag State Persistence

- Flag states are saved per module in `~/.config/lazymvn/projects/<hash>/preferences.json`
- States are restored when you switch between modules or restart LazyMVN
- Each module can have different flag configurations

### With Commands

When you run a Maven command (e.g., `c` for compile), all enabled flags are automatically included:

```bash
# Without custom flags:
mvn clean compile

# With "Development mode" and "Skip tests" enabled:
mvn clean compile -Dspring.profiles.active=dev -DskipTests
```

### Viewing the Command

Press `y` (yank) to copy the full Maven command with all flags to the clipboard. This is useful for:
- Verifying which flags are enabled
- Running the same command outside LazyMVN
- Debugging command-line arguments

## Advanced Usage

### Combining Multiple Properties

You can include multiple Maven properties in a single flag:

```toml
{ name = "Full dev setup", flag = "-Dspring.profiles.active=dev -Ddb.host=localhost -Dapi.mock=true" }
```

### Maven Arguments vs Properties

Custom flags can include any Maven command-line argument, not just `-D` properties:

```toml
[maven]
custom_flags = [
    # Maven properties (most common)
    { name = "Property", flag = "-Dmy.property=value" },
    
    # Maven arguments
    { name = "Resume from", flag = "-rf :my-module" },
    { name = "Batch mode", flag = "-B" },
    { name = "Strict checksums", flag = "-C" },
    
    # Multiple threads (alternative to built-in flag)
    { name = "8 threads", flag = "-T 8" },
]
```

### Conditional Logic

Use custom flags to enable different configurations:

```toml
[maven]
custom_flags = [
    # Local development
    { name = "Local (H2 DB)", flag = "-Dspring.datasource.url=jdbc:h2:mem:testdb", enabled = true },
    
    # Production-like (PostgreSQL)
    { name = "Production-like (PostgreSQL)", flag = "-Dspring.datasource.url=jdbc:postgresql://localhost:5432/prod" },
]
```

Toggle between them in the Flags panel as needed.

## Best Practices

### Naming Conventions

- Use descriptive names that explain what the flag does
- Keep names short but meaningful (they appear in a narrow panel)
- Group related flags with prefixes (e.g., "DB: Local", "DB: Remote")

### Organization

Order your flags logically:

```toml
[maven]
custom_flags = [
    # === Environment ===
    { name = "Dev environment", flag = "-Denv=dev", enabled = true },
    { name = "Staging environment", flag = "-Denv=staging" },
    
    # === Database ===
    { name = "Local database", flag = "-Ddb.host=localhost" },
    { name = "Remote database", flag = "-Ddb.host=db.example.com" },
    
    # === Features ===
    { name = "Feature X", flag = "-Dfeature.x=true" },
    { name = "Feature Y", flag = "-Dfeature.y=true" },
]
```

### Default States

Use `enabled = true` for flags you want active by default:

```toml
[maven]
custom_flags = [
    # Most common environment
    { name = "Development mode", flag = "-Dspring.profiles.active=dev", enabled = true },
    
    # Optional optimizations
    { name = "Skip tests", flag = "-DskipTests" },
]
```

### Avoid Conflicts

Be careful not to duplicate built-in flags:

**Built-in flags** (don't redefine these):
- `-o` (Work offline)
- `-U` (Force update snapshots)
- `-X` (Debug output)
- `-DskipTests` (Skip tests)
- `-T 4` (Build with 4 threads)
- `--also-make`
- `--also-make-dependents`

## Troubleshooting

### Flags Not Appearing

1. Check TOML syntax: `cargo run -- --project . --debug`
2. Verify the `[maven]` section exists
3. Ensure `custom_flags` is an array of objects
4. Check for typos in field names (`name`, `flag`, `enabled`)

### Flags Not Applied

1. Press `y` to see the full command
2. Check if the flag is enabled (highlighted in the Flags panel)
3. Verify the flag syntax is correct for Maven
4. Check debug logs: `tail -f ~/.local/share/lazymvn/logs/debug.log`

### Configuration Not Reloading

1. Press `Ctrl+E` to edit config
2. Save and close the editor
3. Check for TOML syntax errors in the debug log
4. Restart LazyMVN if needed

## Related Features

- **[Built-in Flags](../README.md#key-bindings)**: Standard Maven flags
- **[Profiles](../README.md#maven-profiles)**: Maven profile activation
- **[Live Config Reload](./LIVE_CONFIG_RELOAD.md)**: Edit config without restarting
- **[Logging Configuration](./LOGGING_CONFIG.md)**: Package-level log filtering

## Examples

See [examples/lazymvn.toml.custom-flags-example](../../examples/lazymvn.toml.custom-flags-example) for a comprehensive example with common patterns.

## Testing

Test your custom flags configuration:

```bash
./scripts/test-custom-flags.sh
```

This script:
1. Creates a test configuration
2. Verifies TOML parsing
3. Provides instructions for manual TUI testing
