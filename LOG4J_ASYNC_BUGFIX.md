# CRITICAL BUGFIX: JAVA_TOOL_OPTIONS in Async Commands

## Date
2025-10-27 21:05

## Problem

After implementing `JAVA_TOOL_OPTIONS` in fix attempt #4, **the fix still didn't work** because the code was only added to `execute_maven_command_with_options()` but **NOT** to `execute_maven_command_async_with_options()`.

**Spring Boot launcher uses async execution**, so the `JAVA_TOOL_OPTIONS` environment variable was never set!

## Evidence from Debug Logs (5th Report)

User reported "toujours pas" (still not working) with debug logs showing:

```
[SESSION:20251027-210020-094] [2025-10-27 21:00:35.468] INFO - Executing: mvn.cmd --settings ... -Drun.jvmArguments=...
```

**Missing line**: There was NO `INFO - JAVA_TOOL_OPTIONS=...` log entry!

This meant the JAVA_TOOL_OPTIONS code was never executed.

## Root Cause

In `src/maven/command.rs`, there are **TWO** command execution functions:

1. **`execute_maven_command_with_options()`** - For **synchronous** commands
   - Used for: `mvn compile`, `mvn test`, `mvn package`, etc.
   - ✅ Had JAVA_TOOL_OPTIONS code (from fix #4)

2. **`execute_maven_command_async_with_options()`** - For **asynchronous** commands
   - Used for: `spring-boot:run`, `exec:java` (background processes)
   - ❌ Did NOT have JAVA_TOOL_OPTIONS code
   - **This is what Spring Boot launcher uses!**

## Solution

Added the exact same JAVA_TOOL_OPTIONS injection code to `execute_maven_command_async_with_options()`:

```rust
let mut command = Command::new(maven_command);

// CRITICAL: Set JAVA_TOOL_OPTIONS environment variable
if logging_config.is_some() {
    let mut java_tool_opts = Vec::new();
    
    if let Some(log4j_config_url) = extract_log4j_config_url(args) {
        java_tool_opts.push("-Dlog4j.ignoreTCL=true".to_string());
        java_tool_opts.push("-Dlog4j.defaultInitOverride=true".to_string());
        java_tool_opts.push(format!("-Dlog4j.configuration={}", log4j_config_url));
        log::info!("Setting JAVA_TOOL_OPTIONS with Log4j configuration: {}", log4j_config_url);
    }
    
    if !java_tool_opts.is_empty() {
        let opts_str = java_tool_opts.join(" ");
        command.env("JAVA_TOOL_OPTIONS", &opts_str);
        log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
    }
}
```

**Now present in BOTH functions**:
- ✅ `execute_maven_command_with_options()` (line ~292-312)
- ✅ `execute_maven_command_async_with_options()` (line ~591-611)

## Files Modified

**`src/maven/command.rs`**:
- Added JAVA_TOOL_OPTIONS injection in `execute_maven_command_async_with_options()`
- Now consistent with sync version

**`LOG4J_JAVA_TOOL_OPTIONS_FIX.md`**:
- Updated to document that code must be in BOTH functions
- Added explanation of why both are needed

## Testing

```bash
# Build
cargo build --release

# Tests
cargo test --lib
# Result: 276 passed; 0 failed ✓
```

## Expected Behavior (Now)

When user presses `s` to start Spring Boot application, debug logs should now show:

```
[INFO] Setting JAVA_TOOL_OPTIONS with Log4j configuration: file:///C:/Users/.../lazymvn/log4j/log4j-override-ec936686.properties
[INFO] JAVA_TOOL_OPTIONS=-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///...
[INFO] Executing: mvn.cmd ... -Drun.jvmArguments=... org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run
```

And application startup should show:

```
Picked up JAVA_TOOL_OPTIONS: -Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///...
```

## Why This Bug Happened

**Code duplication**: There are two nearly identical command execution functions (sync vs async), and the fix was only applied to one of them.

**Lesson learned**: When modifying command execution logic, **ALWAYS check BOTH functions**:
- `execute_maven_command_with_options()`
- `execute_maven_command_async_with_options()`

## User Action Required

1. **Rebuild LazyMVN**:
   ```bash
   cargo build --release
   ```

2. **Restart LazyMVN** and launch application (press `s`)

3. **Look for JAVA_TOOL_OPTIONS in debug logs**:
   ```bash
   tail -f lazymvn-debug.log | grep JAVA_TOOL_OPTIONS
   ```

4. **Verify log format** changed to LazyMVN format:
   - ✅ Expected: `[INFO][fr.company.branch.assemblage.ApplicationStarter]`
   - ❌ Old: `[27/10/2025 21:00:44:340] [INFO ]`

5. **If still not working**: Shift+Y for debug report

## Related Documents

- `LOG4J_JAVA_TOOL_OPTIONS_FIX.md` - Main documentation for fix #4
- `LOG4J_FIX_COMPLETE_HISTORY.md` - Complete history of all 4 attempts
- `SPRING_BOOT_1X_FIX_SUMMARY.md` - Fixes #1 and #2
- `LOG4J_CUSTOM_FACTORY_FIX.md` - Fix #3

## Timeline

- 2025-10-27 20:45 - Attempt #4: JAVA_TOOL_OPTIONS implemented (sync only)
- 2025-10-27 21:00 - User report: "toujours pas"
- 2025-10-27 21:05 - **Bugfix**: Added JAVA_TOOL_OPTIONS to async function
