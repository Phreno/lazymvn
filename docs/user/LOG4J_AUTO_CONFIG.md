# Automatic Log4j 1.x Configuration Override

## Problem

When running Spring Boot applications that use Log4j 1.x (common in enterprise applications), the logging level overrides defined in `lazymvn.toml` were not working. The application would load its own `log4j.properties` file before Spring Boot could apply the `-Dlogging.level.*` system properties.

### Symptoms

User reported seeing verbose logs from frameworks like:
- `org.springframework`
- `com.couchbase`
- `fr.company.example.fwmc`

Even though they had configured logging overrides in `lazymvn.toml`:

```toml
[logging]
packages = [
    { name = "org.springframework", level = "WARN" },
    { name = "com.couchbase", level = "WARN" },
]
```

### Root Cause

Log4j 1.x applications load their configuration from a `log4j.properties` file at startup, **before** any JVM system properties can take effect. The `-Dlog4j.logger.*` properties passed as JVM arguments only work if the `log4j.properties` file explicitly references them using `${...}` placeholders.

## Solution

LazyMVN now automatically generates a temporary Log4j 1.x configuration file when:
1. Logging overrides are defined in `lazymvn.toml`
2. A Spring Boot application is launched

The generated file contains:
- Root logger configuration
- Console appender with reasonable format
- All logging level overrides from `lazymvn.toml`

This file is then automatically injected via `-Dlog4j.configuration=file:///path/to/config` JVM argument, which Log4j 1.x respects **before** loading default configuration files.

### User Experience

The user doesn't need to do anything. They simply:

1. Define logging levels in `lazymvn.toml`:
```toml
[logging]
packages = [
    { name = "org.springframework", level = "WARN" },
    { name = "com.couchbase", level = "WARN" },
    { name = "fr.company.example.fwmc", level = "WARN" },
    { name = "fr.company.example", level = "WARN" },
    { name = "fr.company.example.assemblage", level = "DEBUG" },
]
```

2. Launch their Spring Boot application (press `s` in LazyMVN)

3. **It just works™** - Log levels are applied automatically

### How It Works

When launching a Spring Boot application (`s` key):

1. **Read configuration**: LazyMVN reads `[logging]` section from `lazymvn.toml`

2. **Generate Log4j file**: Creates configuration in `~/.config/lazymvn/log4j/`:
   ```properties
   # LazyMVN Generated Log4j 1.x Configuration
   log4j.rootLogger=INFO, CONSOLE
   
   log4j.appender.CONSOLE=org.apache.log4j.ConsoleAppender
   log4j.appender.CONSOLE.layout=org.apache.log4j.PatternLayout
   log4j.appender.CONSOLE.layout.ConversionPattern=[%d{dd/MM/yyyy HH:mm:ss:SSS}] %5p %c{1} - %m%n
   
   # Logging level overrides from lazymvn.toml
   log4j.logger.org.springframework=WARN
   log4j.logger.com.couchbase=WARN
   # ... etc
   ```

3. **Inject configuration**: Adds JVM argument pointing to the config directory:
   ```bash
   -Dlog4j.configuration=file:///home/user/.config/lazymvn/log4j/log4j-override-a1b2c3d4.properties
   ```

4. **Backward compatibility**: Also adds `-Dlogging.level.*` for Logback/Spring Boot 2+

### File Location

The generated Log4j configuration is stored in LazyMVN's config directory:
```
~/.config/lazymvn/log4j/log4j-override-<hash>.properties
```

Where:
- **Linux/macOS**: `~/.config/lazymvn/log4j/`
- **Windows**: `%APPDATA%\lazymvn\log4j\`
- **Hash**: First 8 characters of MD5 hash of project path (to support multiple projects)

This keeps all LazyMVN configuration in one place, alongside:
- `~/.config/lazymvn/cache.json` - Module cache
- `~/.config/lazymvn/recent.json` - Recent projects
- `~/.config/lazymvn/starters/` - Spring Boot starter cache

## Implementation Details

### New Module: `src/maven/log4j.rs`

Provides two main functions:

#### `generate_log4j_config()`
```rust
pub fn generate_log4j_config(
    project_root: &Path,
    logging_overrides: &[(String, String)],
    log_format: Option<&str>,
) -> Option<PathBuf>
```

Generates a Log4j 1.x properties file with:
- Standard console appender
- Root logger at INFO level
- All logging overrides as `log4j.logger.<package>=<level>`
- Unique filename based on project path hash

Returns the path to the generated file in `~/.config/lazymvn/log4j/`.

#### `detect_log4j1_usage()` (for future use)
```rust
pub fn detect_log4j1_usage(output_lines: &[String]) -> bool
```

Detects if application output indicates Log4j 1.x usage by looking for:
- "Log4jJbossLoggerFactory"
- "log4j.properties"
- "log4j:WARN" messages

Currently unused but kept for potential future features (e.g., showing a notification when Log4j 1.x is detected).

### Modified: `src/ui/state/mod.rs`

In `run_spring_boot_starter()` method (around line 2810):

**Before:**
```rust
let jvm_args: Vec<String> = logging_config
    .packages
    .iter()
    .flat_map(|pkg| vec![
        format!("-Dlog4j.logger.{}={}", pkg.name, pkg.level),
        format!("-Dlogging.level.{}={}", pkg.name, pkg.level),
    ])
    .collect();
```

**After:**
```rust
// Generate Log4j config file
let mut jvm_args: Vec<String> = if let Some(log4j_config_path) = 
    generate_log4j_config(&project_root, &logging_overrides, Some("[%p] %c - %m%n")) 
{
    let config_url = format!("file:///{}", log4j_config_path.display());
    vec![format!("-Dlog4j.configuration={}", config_url)]
} else {
    Vec::new()
};

// Add Logback/Spring Boot compatibility
for pkg in &logging_config.packages {
    jvm_args.push(format!("-Dlogging.level.{}={}", pkg.name, pkg.level));
}
```

### File Structure

```
src/maven/
├── command.rs        # Maven command execution
├── detection.rs      # Spring Boot detection
├── log4j.rs          # NEW: Log4j config generation
├── mod.rs            # Module exports
├── process.rs        # Process management
└── profiles.rs       # Maven profiles
```

## Testing

### Manual Test

1. Open a Spring Boot project that uses Log4j 1.x:
   ```bash
   cargo run --release -- --project /path/to/spring-boot-app
   ```

2. Configure logging in `lazymvn.toml`:
   ```toml
   [logging]
   packages = [
       { name = "org.springframework", level = "WARN" },
   ]
   ```

3. Launch the app (press `s`)

4. **Expected results:**
   - File `~/.config/lazymvn/log4j/log4j-override-<hash>.properties` is created
   - Application logs show reduced Spring Framework output
   - Debug log shows: `"Injecting Log4j 1.x configuration: file:///..."`

5. Check the generated file:
   ```bash
   cat ~/.config/lazymvn/log4j/log4j-override-*.properties
   ```

### Automated Tests

Tests are included in `src/maven/log4j.rs`:

```bash
cargo test maven::log4j::tests
```

Tests verify:
- Log4j 1.x detection from output lines
- Config file generation
- Correct content in generated file

## Compatibility

### Log4j 1.x
✅ **Fully supported** - Configuration file is automatically generated and injected

### Logback (Spring Boot 2.x+)
✅ **Fully supported** - Uses `-Dlogging.level.*` properties (unchanged behavior)

### Log4j 2.x
⚠️ **Partially supported** - Uses different configuration file format
- Current implementation generates Log4j 1.x format only
- Future enhancement: detect Log4j 2.x and generate XML/JSON config

### java.util.logging (JUL)
❌ **Not supported** - Would require different configuration approach
- Rare in Spring Boot applications
- Can be added if needed

## Benefits

1. **Zero user effort**: Just configure `lazymvn.toml` and it works
2. **No repository pollution**: Config stored in system directory (`~/.config/lazymvn/`)
3. **Multi-project support**: Unique file per project using path hash
4. **Backward compatible**: Works with both Log4j 1.x and Logback
5. **Safe**: If not needed, generated file is simply ignored
6. **Flexible**: User can still override by providing their own `log4j.configuration`
7. **Centralized**: All LazyMVN config in one place

## Future Enhancements

1. **Detect Log4j version**: Show notification when Log4j 1.x is detected
2. **Support Log4j 2.x**: Generate XML/JSON configuration for Log4j 2.x
3. **Support java.util.logging**: Generate `logging.properties` for JUL
4. **Config templates**: Allow users to provide custom Log4j templates
5. **Per-module logging**: Different log levels per Maven module

## Related Issues

- Original issue: "Logging overrides don't work with Log4j 1.x"
- Root cause: Log4j 1.x loads config before JVM properties take effect
- Solution: Generate and inject configuration file automatically

## References

- [Apache Log4j 1.x Configuration](https://logging.apache.org/log4j/1.x/manual.html)
- [Spring Boot Logging Documentation](https://docs.spring.io/spring-boot/docs/current/reference/html/features.html#features.logging)
- [Spring Boot Maven Plugin](https://docs.spring.io/spring-boot/maven-plugin/run.html)

## Files Changed

- `src/maven/log4j.rs` - NEW: Log4j configuration generation
- `src/maven/mod.rs` - Export new Log4j module
- `src/ui/state/mod.rs` - Integrate Log4j config generation in launch flow

## Status

✅ **Implemented** and ready for use
✅ Clippy clean
✅ Tests passing
✅ Documentation complete
