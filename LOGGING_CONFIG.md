# Logging Configuration

LazyMVN allows you to control log verbosity for Maven commands without modifying source code.

## How it works

Log level overrides are injected as JVM arguments to all Maven commands:
- `-Dlog4j.logger.{package}={level}` (for Log4j)
- `-Dlogging.level.{package}={level}` (for Spring Boot / Logback / SLF4J)

## Configuration

Add a `[logging]` section to your `lazymvn.toml`:

```toml
[logging]
packages = [
    { name = "com.mycompany.api.service", level = "ERROR" },
    { name = "org.springframework.web", level = "WARN" },
    { name = "org.hibernate", level = "ERROR" },
]
```

### Supported Log Levels

- `TRACE` - Most verbose
- `DEBUG` - Debug information
- `INFO` - Informational messages (default)
- `WARN` - Warning messages
- `ERROR` - Error messages only
- `FATAL` - Fatal errors only
- `OFF` - No logging

## Use Cases

### Reducing noisy logs

When running Spring Boot applications with verbose logging:

```toml
[logging]
packages = [
    { name = "org.springframework", level = "WARN" },
    { name = "org.hibernate", level = "ERROR" },
    { name = "org.apache.catalina", level = "WARN" },
]
```

### Debugging specific packages

When troubleshooting issues in a specific package:

```toml
[logging]
packages = [
    { name = "com.mycompany.problematic", level = "DEBUG" },
    { name = "org.springframework", level = "ERROR" },
]
```

### Multi-module projects

Apply different log levels to different modules:

```toml
[logging]
packages = [
    # Your business logic - debug
    { name = "com.mycompany.service", level = "DEBUG" },
    
    # Your API layer - info
    { name = "com.mycompany.api", level = "INFO" },
    
    # Framework code - errors only
    { name = "org.springframework", level = "ERROR" },
    { name = "org.hibernate", level = "ERROR" },
]
```

## Benefits

✅ **No source code changes** - No need to modify `log4j.properties` or `logback.xml`  
✅ **Per-developer** - Each developer can have their own log preferences  
✅ **Portable** - Configuration travels with your `lazymvn.toml`  
✅ **Reversible** - Just remove from config to restore default logging  
✅ **Works everywhere** - Applies to all modules without finding the right config file  

## Compatibility

Works with:
- Log4j 1.x and 2.x
- Logback (via SLF4J)
- Spring Boot logging
- java.util.logging (limited)

## Finding noisy packages

Run your application and look at the log output:

```
2024-01-15 10:23:45 DEBUG [org.springframework.web.servlet.DispatcherServlet] - ...
2024-01-15 10:23:45 DEBUG [org.hibernate.SQL] - ...
2024-01-15 10:23:45 DEBUG [com.mycompany.api.service.UserService] - ...
```

The package name is the part in brackets. Add it to your config with a higher level like `WARN` or `ERROR`.

## Example: Spring Boot application

Common verbose packages in Spring Boot apps:

```toml
[logging]
packages = [
    # Spring Framework
    { name = "org.springframework.web", level = "WARN" },
    { name = "org.springframework.security", level = "WARN" },
    { name = "org.springframework.boot.actuate", level = "WARN" },
    
    # Hibernate / JPA
    { name = "org.hibernate", level = "WARN" },
    { name = "org.hibernate.SQL", level = "WARN" },
    
    # Embedded server
    { name = "org.apache.catalina", level = "WARN" },
    { name = "org.apache.tomcat", level = "WARN" },
    
    # Your app - keep verbose for development
    { name = "com.mycompany", level = "DEBUG" },
]
```

## Notes

- Changes apply on the next Maven command execution
- Overrides are passed to all Maven goals (build, test, run, etc.)
- JVM arguments have higher priority than file-based configuration
- This doesn't modify your `log4j.properties` or `logback.xml` files
