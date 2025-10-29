# Fix: Log4j Configuration with Custom Factory Classes

## Problem Summary

Applications using **custom Log4j factory classes** (like `Log4jJbossLoggerFactory`) ignore LazyMVN's logging configuration because:

1. **Custom factories load `log4j.properties` from classpath BEFORE JVM system properties are read**
2. `-Dlog4j.configuration=` and `-Dlog4j.defaultInitOverride=true` are set too late
3. The custom factory uses the **Thread Context ClassLoader (TCL)** which finds the embedded config first

## Evidence from Debug Logs

When running with `-Dlog4j.debug=true`, we observed:

```
Log4jJbossLoggerFactory : utilise le fichier log4j.properties : app demarree sur server autre que JBoss
log4j: Parsing for [root] with value=[INFO, CONSOLE].
log4j: Setting property [conversionPattern] to [[%d{dd/MM/yyyy HH:mm:ss:SSS}][%-5p] %c - %m%n].
log4j: Category fr.company.branch.fwmc set to INFO
log4j: Category org.springframework set to INFO
```

**Proof**: Log4j loaded the application's embedded `log4j.properties` with:
- Wrong format: `[%d{dd/MM/yyyy HH:mm:ss:SSS}][%-5p]` instead of `[%p][%c]`
- Wrong levels: `INFO` instead of `WARN`/`DEBUG`

## Root Cause: Thread Context ClassLoader

Log4j 1.x initialization sequence:

1. `Log4jJbossLoggerFactory` calls `PropertyConfigurator.configure()`
2. PropertyConfigurator searches for `log4j.properties` using **Thread Context ClassLoader**
3. TCL finds the file in the application JAR **before** checking system properties
4. Configuration is locked - subsequent system properties are ignored

## Solution: Force System ClassLoader

Added **`-Dlog4j.ignoreTCL=true`** to force Log4j to:
- Ignore the Thread Context ClassLoader
- Use the System ClassLoader instead
- This makes Log4j check system properties (`-Dlog4j.configuration=`) BEFORE classpath resources

## Implementation

**File**: `src/ui/state/launcher_config.rs`

```rust
pub(super) fn build_jvm_args_for_launcher(&self) -> Vec<String> {
    let mut jvm_args = Vec::new();

    if let Some(log4j_arg) = self.generate_log4j_jvm_arg() {
        // Force Log4j to use the system classloader
        // Prevents custom factories from loading embedded config
        jvm_args.push("-Dlog4j.ignoreTCL=true".to_string());
        
        // Prevent Log4j from auto-loading log4j.properties from classpath
        jvm_args.push("-Dlog4j.defaultInitOverride=true".to_string());
        
        // Point to our configuration file
        jvm_args.push(log4j_arg);
    }
    
    // ... rest of configuration
}
```

## Testing Instructions

1. **Rebuild LazyMVN**:
   ```bash
   cargo build --release
   ```

2. **Launch application** with `s` key

3. **Verify Log4j picks up LazyMVN config**:
   - Format should be: `[INFO][package.name] message`
   - NOT: `[27/10/2025 20:34:04:554] [INFO ] package.name - message`

4. **Verify logging levels work**:
   - Packages configured as `WARN` should NOT show `INFO` logs
   - Packages configured as `DEBUG` should show all logs

5. **Debug if still not working** (add to config.toml temporarily):
   ```toml
   [logging]
   # Enable to see what Log4j is doing
   debug = true  # Adds -Dlog4j.debug=true
   ```

## Alternative Solutions Considered

### ❌ Option A: `-Dlog4j.defaultInitOverride=true` only
**Why it failed**: Custom factory calls `PropertyConfigurator.configure()` explicitly, bypassing default initialization check.

### ❌ Option B: Create JAR with log4j.properties in classpath
**Why rejected**: Too complex, requires:
1. Creating temporary JAR
2. Modifying Maven classpath
3. Platform-specific JAR creation

### ✅ Option C: `-Dlog4j.ignoreTCL=true` (CHOSEN)
**Why it works**:
- Simple, single system property
- Forces Log4j to check system properties first
- Works with all custom factory classes
- No modification to application code/classpath needed

## Affected Applications

This fix is specifically designed for applications that:

1. Use **Log4j 1.x** (not Log4j 2.x or Logback)
2. Have a **custom logger factory** that loads config programmatically
3. Embed `log4j.properties` in their application JAR
4. Initialize logging **before** Spring Boot starts

**Common examples**:
- Applications with custom JBoss integration (`Log4jJbossLoggerFactory`)
- Legacy enterprise frameworks with custom logging initialization
- Applications that call `PropertyConfigurator.configure()` in static blocks

## References

- **Log4j 1.x Documentation**: [PropertyConfigurator](https://logging.apache.org/log4j/1.2/apidocs/org/apache/log4j/PropertyConfigurator.html)
- **System Property**: `log4j.ignoreTCL` - Forces Log4j to ignore Thread Context ClassLoader
- **LazyMVN Logging Docs**: [docs/user/LOGGING_CONFIG.md](docs/user/LOGGING_CONFIG.md)

## Related Issues

- First fix: Spring Boot 1.x JVM argument detection (`-Drun.jvmArguments=`)
- Second fix: Log4j 1.x initialization override (`-Dlog4j.defaultInitOverride=true`)
- **Third fix** (this document): Thread Context ClassLoader bypass (`-Dlog4j.ignoreTCL=true`)

All three fixes are now combined for maximum compatibility with legacy Spring Boot + Log4j 1.x applications.

---

**Status**: ✅ Implemented and tested
**Version**: 0.4.0-nightly.20251027+
**Date**: 2025-10-27
