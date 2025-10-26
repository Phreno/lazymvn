# Log Formatting Implementation

## Objective

Allow users to override the Maven log format via a new `log_format` setting in `lazymvn.toml`.

## Design Choices

- **Configuration**: A new `log_format` field in the `[logging]` section of `lazymvn.toml`.
- **Injection**: Pass the format via Spring Boot override files or JVM system properties depending on the launch strategy.
- **Compatibility**: Support Log4j 1.x (via `log4j.conversionPattern`/generated `log4j.properties`) and Logback/Spring Boot (via `logging.pattern.*`).

## Code Changes

### 1. `src/core/config/logging.rs`

Added `log_format` to `LoggingConfig`:

```rust
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LoggingConfig {
    /// Custom log format override
    pub log_format: Option<String>,

    /// List of packages with custom log levels
    #[serde(default)]
    pub packages: Vec<PackageLogLevel>,
}
```

### 2. `src/maven/command.rs`

- Adds `-Dlog4j.conversionPattern` and `-Dlogging.pattern.console` unless the command already carries `-Dspring-boot.run.jvmArguments=` (the Spring path injects overrides elsewhere).
- Ensures the displayed command matches the executed arguments to avoid user confusion.

## How It Works

1.  **Configuration Loading**: `log_format` is deserialized into `LoggingConfig`.
2.  **Spring Boot Path**: `generate_spring_properties_jvm_arg` adds the format to a high-priority Spring properties file (`logging.pattern.console` + `logging.pattern.file`). The JVM receives a single `-Dspring.config.additional-location=...` argument.
3.  **Other Commands**: The Maven command receives `-Dlog4j.conversionPattern` and `-Dlogging.pattern.console` system properties directly.
4.  **Log4j 1.x Overrides**: When package overrides are present, `generate_log4j_config` writes the requested pattern into the generated `log4j.properties` so that both overrides and format share one file.

## Testing

### Manual Testing

1.  Add `log_format = "[%p] %c{1}: %m%n"` to `lazymvn.toml`.
2.  Run a Spring Boot application.
3.  Verify that the log output in the TUI matches the specified format.

### Automated Tests

Unit tests cover:

- Command string generation skips duplicate overrides when `spring-boot.run.jvmArguments` is present.
- Generated Log4j configuration files honour the requested format.
- `path_to_file_url` produces correct `file://` URLs on both Unix and Windows builds.

## Future Enhancements

-   **Framework-specific formats**: Allow specifying different formats for Log4j and Logback.
-   **Format validation**: Validate the `log_format` string to catch errors before running Maven.
-   **Live reload**: If the `log_format` is changed via `Ctrl+E`, the next command should use the new format.
