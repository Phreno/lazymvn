# Log4j Configuration Override - Complete Fix History

## Overview

This document summarizes **4 attempts** to fix log filtering for Spring Boot 1.x applications with custom Log4j initialization (specifically `Log4jJbossLoggerFactory`).

## Timeline

| Date | Attempt | Approach | Result |
|------|---------|----------|--------|
| 2025-10-27 (1st) | #1 | Spring Boot 1.x argument detection | ❌ Partial - Args passed but config ignored |
| 2025-10-27 (2nd) | #2 | `-Dlog4j.defaultInitOverride=true` | ❌ Failed - Custom factory bypasses flag |
| 2025-10-27 (3rd) | #3 | `-Dlog4j.ignoreTCL=true` | ❌ Failed - Initialization too late |
| 2025-10-27 (4th) | #4 | **`JAVA_TOOL_OPTIONS`** | ✅ **Should work - Earliest init** |

## Attempt #1: Spring Boot 1.x Argument Detection

### Problem
Code only detected `-Dspring-boot.run.jvmArguments=` (Spring Boot 2.x) but not `-Drun.jvmArguments=` (Spring Boot 1.x).

### Solution
Added detection for both formats in `src/maven/command.rs`:
```rust
let has_spring_boot_jvm_args = args
    .iter()
    .any(|arg| 
        arg.starts_with("-Dspring-boot.run.jvmArguments=") || 
        arg.starts_with("-Drun.jvmArguments=")
    );
```

### Outcome
✅ Arguments properly passed to Spring Boot Maven plugin
❌ But Log4j still loaded embedded `log4j.properties`

### Documentation
- `SPRING_BOOT_1X_FIX_SUMMARY.md`

---

## Attempt #2: Disable Log4j Automatic Initialization

### Problem
Log4j automatically loads `log4j.properties` from classpath before LazyMVN config can be applied.

### Solution
Added `-Dlog4j.defaultInitOverride=true` flag:
```rust
jvm_args.push("-Dlog4j.defaultInitOverride=true".to_string());
jvm_args.push(log4j_arg);
```

### Root Cause Discovery
Custom factory `Log4jJbossLoggerFactory` **manually calls** `PropertyConfigurator.configure()` in its constructor, bypassing the `defaultInitOverride` flag.

### Outcome
❌ Flag only prevents *automatic* initialization
❌ Does NOT prevent manual `configure()` calls

### Documentation
- `SPRING_BOOT_1X_FIX_SUMMARY.md` (updated)

---

## Attempt #3: Force System ClassLoader

### Problem
`Log4jJbossLoggerFactory` uses **Thread Context ClassLoader** to find `log4j.properties`, which has priority over system properties.

### Debug Evidence
```
Log4jJbossLoggerFactory : utilise le fichier log4j.properties
log4j: Setting property [conversionPattern] to [[%d{dd/MM/yyyy HH:mm:ss:SSS}][%-5p] %c - %m%n]
```
(Wrong format - embedded config loaded)

### Solution
Added `-Dlog4j.ignoreTCL=true` to force System ClassLoader:
```rust
jvm_args.push("-Dlog4j.ignoreTCL=true".to_string());
jvm_args.push("-Dlog4j.defaultInitOverride=true".to_string());
jvm_args.push(log4j_arg);
```

### Root Cause Discovery
**Initialization timing problem**: `Log4jJbossLoggerFactory` constructor runs BEFORE system properties from `-Drun.jvmArguments=` are set!

```
Maven starts → Plugin reads -Drun.jvmArguments → Forks JVM → JVM starts → Properties set
                                                                 ↓
                                              (TOO LATE! Factory already initialized)
```

### Outcome
❌ Properties applied too late in initialization sequence

### Documentation
- `LOG4J_CUSTOM_FACTORY_FIX.md`

---

## Attempt #4: JAVA_TOOL_OPTIONS Environment Variable ⭐

### Problem
All previous approaches set properties via `-Drun.jvmArguments=`, which are applied AFTER the JVM forks. But `Log4jJbossLoggerFactory` initializes in the **application constructor**, BEFORE those properties take effect.

### Complete Initialization Sequence (Problem)
```
1. User presses 's' in LazyMVN
2. LazyMVN spawns Maven with: -Drun.jvmArguments="..."
3. Maven JVM starts
4. Maven loads Spring Boot Maven Plugin
5. Plugin reads -Drun.jvmArguments="..."
6. Plugin FORKS new JVM with those arguments
7. New JVM starts
8. System properties from -Drun.jvmArguments FINALLY SET ← TOO LATE!
9. Spring Boot starts
10. ApplicationConfigurationManager bean created
11. Constructor calls initBootStrap()
12. Log4jJbossLoggerFactory constructor runs
13. Factory manually calls: PropertyConfigurator.configure("log4j.properties")
14. Log4j loads EMBEDDED log4j.properties (system properties checked AFTER manual config)
```

### Solution: Use JAVA_TOOL_OPTIONS
**`JAVA_TOOL_OPTIONS`** is a JVM environment variable that:
- ✅ Applies **BEFORE** any Java code executes
- ✅ Affects **ALL** JVMs (Maven + forked application)
- ✅ Has **HIGHEST PRIORITY** - Properties available immediately

### Implementation

**File**: `src/maven/command.rs`

**New helper function**:
```rust
/// Extract Log4j configuration URL from Maven JVM arguments
fn extract_log4j_config_url(args: &[&str]) -> Option<String> {
    for arg in args {
        if arg.starts_with("-Drun.jvmArguments=") || 
           arg.starts_with("-Dspring-boot.run.jvmArguments=") {
            // Extract -Dlog4j.configuration=file:///...
            if let Some(jvm_args_str) = arg.split('=').nth(1) {
                for part in jvm_args_str.split_whitespace() {
                    if part.starts_with("-Dlog4j.configuration=") {
                        return part.strip_prefix("-Dlog4j.configuration=")
                            .map(|s| s.to_string());
                    }
                }
            }
        }
    }
    None
}
```

**Modified command execution**:
```rust
let mut command = Command::new(&maven_command);

// Set JAVA_TOOL_OPTIONS environment variable
if logging_config.is_some() {
    let mut java_tool_opts = Vec::new();
    
    if let Some(log4j_config_url) = extract_log4j_config_url(args) {
        java_tool_opts.push("-Dlog4j.ignoreTCL=true".to_string());
        java_tool_opts.push("-Dlog4j.defaultInitOverride=true".to_string());
        java_tool_opts.push(format!("-Dlog4j.configuration={}", log4j_config_url));
    }
    
    if !java_tool_opts.is_empty() {
        let opts_str = java_tool_opts.join(" ");
        command.env("JAVA_TOOL_OPTIONS", &opts_str);
        log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
    }
}
```

### How It Works (Corrected Sequence)
```
1. User presses 's' in LazyMVN
2. LazyMVN extracts Log4j config URL from JVM arguments
3. LazyMVN sets: JAVA_TOOL_OPTIONS="-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///..."
4. LazyMVN spawns Maven process
5. Maven JVM starts with JAVA_TOOL_OPTIONS properties ALREADY SET ✅
6. Maven loads Spring Boot Maven Plugin
7. Plugin FORKS new JVM (inherits JAVA_TOOL_OPTIONS)
8. Application JVM starts with Log4j properties ALREADY SET ✅
9. Spring Boot starts
10. ApplicationConfigurationManager constructor runs
11. Log4jJbossLoggerFactory constructor runs
12. Factory calls: PropertyConfigurator.configure("log4j.properties")
13. Log4j checks system properties FIRST (because -Dlog4j.ignoreTCL=true)
14. Log4j finds: -Dlog4j.configuration=file:///... (from JAVA_TOOL_OPTIONS)
15. Log4j loads LazyMVN's configuration! ✅
```

### Expected Output
When application starts:
```
Picked up JAVA_TOOL_OPTIONS: -Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///C:/Users/XVHR845/AppData/Roaming/lazymvn/log4j/log4j-override-ec936686.properties
[INFO][fr.company.branch.assemblage.ApplicationStarter] Starting ApplicationStarter on T10J11103-0136
```

### Outcome
✅ **Should work** - Properties set early enough
✅ All JVMs affected (Maven + application)
✅ Cannot be bypassed by manual initialization

### Testing Required
User must test with real Spring Boot 1.x application and verify:
1. Log format changes to LazyMVN format: `[INFO][package]`
2. Log levels respect configuration (WARN/DEBUG)
3. `JAVA_TOOL_OPTIONS` appears in startup logs

### Documentation
- `LOG4J_JAVA_TOOL_OPTIONS_FIX.md` (this fix)

---

## Summary of All Fixes Combined

### Files Modified

1. **`src/maven/command.rs`** (All 4 attempts)
   - Attempt #1: Spring Boot 1.x detection
   - Attempt #2: No changes (flag added in launcher_config.rs)
   - Attempt #3: No changes (flag added in launcher_config.rs)
   - Attempt #4: ✅ **JAVA_TOOL_OPTIONS injection** + helper function

2. **`src/ui/state/launcher_config.rs`** (Attempts #2, #3, #4)
   - Attempt #2: Added `-Dlog4j.defaultInitOverride=true`
   - Attempt #3: Added `-Dlog4j.ignoreTCL=true`
   - Attempt #4: Added `-Dlog4j.configuratorClass=...` (kept for redundancy)

3. **`src/maven/log4j.rs`** (Unchanged)
   - Generates `log4j-override-<hash>.properties` with LazyMVN format

### Current State (Attempt #4)

**JVM Arguments** (passed via `-Drun.jvmArguments=`):
```
-Dlog4j.ignoreTCL=true
-Dlog4j.defaultInitOverride=true
-Dlog4j.configuratorClass=org.apache.log4j.PropertyConfigurator
-Dlog4j.configuration=file:///C:/Users/.../lazymvn/log4j/log4j-override-ec936686.properties
-Dlogging.level.fr.company.branch.fwmc=WARN
-Dlog4j.logger.fr.company.branch.fwmc=WARN
-Dlogging.level.org.springframework=WARN
-Dlog4j.logger.org.springframework=WARN
-Dlogging.level.com.couchbase=WARN
-Dlog4j.logger.com.couchbase=WARN
-Dlogging.level.fr.company.branch.assemblage=DEBUG
-Dlog4j.logger.fr.company.branch.assemblage=DEBUG
-Dspring.config.additional-location=file:///C:/Users/.../lazymvn/spring/application-override-ec936686.properties
```

**Environment Variables** (NEW in Attempt #4):
```
JAVA_TOOL_OPTIONS=-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///C:/Users/.../lazymvn/log4j/log4j-override-ec936686.properties
```

### Why Attempt #4 Should Succeed

| Previous Attempts | Attempt #4 (JAVA_TOOL_OPTIONS) |
|-------------------|--------------------------------|
| Properties set AFTER JVM fork | Properties set BEFORE JVM starts |
| Application code already executing | No application code started yet |
| Custom factory bypasses flags | Impossible to bypass - JVM-level |
| Too late in init sequence | Earliest possible initialization |

### Fallback Options (If Attempt #4 Fails)

If `JAVA_TOOL_OPTIONS` doesn't work, remaining options:

1. **Java Agent** - Instrument Log4j bytecode at load time
2. **AOP (Aspect-Oriented Programming)** - Intercept `PropertyConfigurator.configure()` calls
3. **Custom Maven Plugin** - Modify application classpath before launch
4. **Application Code Modification** - Ask user to respect system properties in factory
5. **Accept Limitation** - Document as incompatible with custom Log4j factories

## Testing Checklist

- [ ] Rebuild LazyMVN: `cargo build --release`
- [ ] Restart LazyMVN with debug: `lazymvn --debug`
- [ ] Launch Spring Boot app: Press `s`
- [ ] Check debug logs: `tail -f ~/.local/share/lazymvn/logs/debug.log | grep JAVA_TOOL_OPTIONS`
- [ ] Verify startup message: "Picked up JAVA_TOOL_OPTIONS: ..."
- [ ] Verify log format: `[INFO][package] message` (not `[timestamp][level] package - message`)
- [ ] Verify log levels: INFO suppressed for WARN packages, DEBUG shown for DEBUG packages
- [ ] Generate debug report if needed: Press `Shift+Y`

## Related Documentation

- `SPRING_BOOT_1X_FIX_SUMMARY.md` - Attempts #1 and #2
- `LOG4J_CUSTOM_FACTORY_FIX.md` - Attempt #3
- `LOG4J_JAVA_TOOL_OPTIONS_FIX.md` - Attempt #4 (detailed)
- `docs/user/LOGGING_CONFIG.md` - User-facing logging configuration guide
- `docs/user/LOG4J_AUTO_CONFIG.md` - Log4j automatic configuration documentation

## References

- [JVM Tool Interface - Environment Variables](https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#tooloptions)
- [Log4j 1.x Manual](https://logging.apache.org/log4j/1.2/manual.html)
- [Spring Boot Maven Plugin 1.x](https://docs.spring.io/spring-boot/docs/1.2.2.RELEASE/maven-plugin/reference/html/)
