# Custom Log Formatting

LazyMVN allows you to override the default log format for Maven commands, giving you full control over what information is displayed.

## How it works

When a `log_format` is specified in your `lazymvn.toml`, LazyMVN adapts the override to match the launch strategy and logging framework:

- **Spring Boot / Logback** (`spring-boot:run`): the format is written to a generated Spring configuration file (`logging.pattern.console` and `logging.pattern.file`). The file is referenced via `-Dspring.config.additional-location=...`, so your project code stays untouched. The file path is emitted as a `file://` URL and works on both Linux and Windows.
- **Exec Java / other goals**: the JVM receives `-Dlog4j.conversionPattern={format}` and `-Dlogging.pattern.console={format}` system properties. Log4j 1.x and Logback respect these properties, so the format is applied immediately.

Both paths ensure consistent output regardless of whether you run `spring-boot:run` or `exec:java` (or any other Maven goal).

## Configuration

Add a `log_format` key to the `[logging]` section of your `lazymvn.toml`:

```toml
[logging]
log_format = "[%p] %c{1} - %m%n"
packages = [
    { name = "org.springframework.web", level = "WARN" },
]
```

### Supported Format Specifiers

The format is a string with special "conversion characters" prefixed with a `%`. Here are some of the most common ones:

| Specifier | Description | Example |
|---|---|---|
| `%p` | Priority (log level) | `DEBUG`, `INFO`, `WARN` |
| `%c` | Logger name (package/class) | `com.example.MyService` |
| `%c{1}` | Last component of logger name | `MyService` |
| `%m` | The log message | `User logged in successfully` |
| `%n` | Newline character | |
| `%d` | Date and time | `2025-10-26 14:30:00,123` |
| `%t` | Thread name | `main`, `http-nio-8080-exec-1` |
| `%L` | Line number | `42` |
| `%M` | Method name | `processRequest` |
| `%F` | File name | `MyService.java` |

For a full list, refer to the [Log4j PatternLayout documentation](https://logging.apache.org/log4j/1.2/apidocs/org/apache/log4j/PatternLayout.html).

### Package Name Highlighting

**Important:** When you specify a `log_format` that includes `%c` (logger/package name), LazyMVN will automatically **highlight package names in cyan** in the output pane. This makes it much easier to visually scan logs and identify which package each log line comes from.

Example with `log_format = "[%p] %c - %m%n"`:
```
[INFO] com.example.service.UserService - User created successfully
       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
       (displayed in cyan)
```

This highlighting works for any format pattern that includes the `%c` specifier.

## Use Cases

### Show only package and message

To reduce noise and focus on the essentials:

```toml
[logging]
log_format = "%c{1}: %m%n"
```

**Output:**
```
DispatcherServlet: Initializing Spring DispatcherServlet 'dispatcherServlet'
UserService: Creating new user
```

### Add thread and line number for debugging

When debugging concurrency issues, you can add thread and line number information:

```toml
[logging]
log_format = "[%t] %c{1}.%M(%L) - %m%n"
```

**Output:**
```
[main] Application.main(25) - Starting application...
[http-1] OrderController.placeOrder(112) - Received new order
```

### Verbose format for detailed analysis

For deep debugging, you can create a very verbose format:

```toml
[logging]
log_format = "%d{ISO8601} [%t] %-5p %c - %m%n"
```

**Output:**
```
2025-10-26 15:00:00,123 [main] INFO  com.example.Application - Application started
```

## Benefits

- **No source code changes**: Override log formats without touching project files.
- **Per-developer settings**: Each developer can have their own preferred format.
- **Easy to toggle**: Just add or remove the `log_format` key.
- **Powerful debugging**: Add contextual information like thread, file, and line number.

## Compatibility

This feature has first-class support for:

- **Log4j 1.x** – the generated configuration file (when package overrides are present) uses your pattern, and the `log4j.conversionPattern` system property ensures consistency for direct launches.
- **Logback / Spring Boot** – both console and file patterns are overridden via a high-priority Spring configuration file. Windows paths are automatically converted to the `file:///C:/...` form expected by the JVM.

Other SLF4J-compatible frameworks that honour these properties may also pick up the override, but they are not explicitly validated.
