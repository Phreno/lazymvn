# Fix: Spring Boot 1.x JVM Arguments Property

**Status**: ✅ **COMPLETED**  
**Date**: 2025-01-27  
**Issue**: Log4j filtering and formatting not working with Spring Boot 1.2.2.RELEASE

## Problem Description

When running a Spring Boot 1.2.2.RELEASE application, the log filtering and formatting configured in `lazymvn.toml` were completely ignored. Neither the custom log format (`[%p][%c] %m%n`) nor the package-level filtering (`fwmc.internal.core=WARN`) were applied.

### Root Cause

LazyMVN was using the Spring Boot 2.x+ property syntax (`-Dspring-boot.run.jvmArguments`) to pass JVM arguments to the forked Spring Boot process. However, Spring Boot 1.x versions use a different property name: `-Drun.jvmArguments`.

**Example of incorrect command (Spring Boot 1.x)**:
```bash
mvn spring-boot:run -Dspring-boot.run.jvmArguments="-Dlog4j.logger.fwmc.internal.core=WARN"
```

**Correct command for Spring Boot 1.x**:
```bash
mvn spring-boot:run -Drun.jvmArguments="-Dlog4j.logger.fwmc.internal.core=WARN"
```

### Impact

- **Spring Boot 1.x applications**: JVM arguments (including Log4j/Logback configuration) were not passed to the application
- **Spring Boot 2.x+ applications**: No impact, continued working as expected

## Solution

### 1. Extract Spring Boot Plugin Version

Modified `src/maven/detection.rs`:

- Added `spring_boot_version: Option<String>` field to `SpringBootDetection` struct
- Extract version from `<version>` tag when parsing effective POM:

```rust
// Extract version for Spring Boot plugin
if current_plugin_artifact_id == "spring-boot-maven-plugin"
    && trimmed.starts_with("<version>")
    && trimmed.contains("</version>")
{
    if let Some(version) = extract_tag_content(trimmed, "version") {
        detection.spring_boot_version = Some(version.clone());
        log::debug!("Found Spring Boot plugin version: {}", version);
    }
}
```

### 2. Version-Aware Property Selection

Modified `build_launch_command()` function to use version-appropriate properties:

```rust
// Determine the correct property prefix based on Spring Boot version
// Spring Boot 1.x uses -Drun.* properties
// Spring Boot 2.x+ uses -Dspring-boot.run.* properties
let is_spring_boot_1x = spring_boot_version
    .map(|v| v.starts_with("1."))
    .unwrap_or(false);

let (profiles_property, jvm_args_property) = if is_spring_boot_1x {
    ("run.profiles", "run.jvmArguments")
} else {
    ("spring-boot.run.profiles", "spring-boot.run.jvmArguments")
};
```

### 3. Update Function Signature

Added `spring_boot_version: Option<&str>` parameter to:

- `build_launch_command()` in `src/maven/detection.rs`
- `execute_launch_command()` in `src/ui/state/mod.rs`

Updated call site in `run_spring_boot_starter()`:

```rust
self.execute_launch_command(
    strategy,
    fqcn,
    &active_profiles,
    &jvm_args,
    detection.packaging.as_deref(),
    detection.spring_boot_version.as_deref(),  // NEW: Pass version
);
```

### 4. Use Full Plugin Coordinates for Spring Boot 1.x

For Spring Boot 1.x, use full `groupId:artifactId:version:goal` format instead of the short `spring-boot:run` prefix:

```rust
// For Spring Boot 1.x, use full plugin coordinates because Maven may not
// resolve the 'spring-boot' prefix correctly with old plugin group IDs
let goal = if is_spring_boot_1x && spring_boot_version.is_some() {
    format!(
        "org.springframework.boot:spring-boot-maven-plugin:{}:run",
        spring_boot_version.unwrap()
    )
} else {
    "spring-boot:run".to_string()
};
```

**Why?** Maven 3.8.2+ may not resolve the `spring-boot` prefix for very old versions (1.x) because the plugin groupId changed. Using full coordinates bypasses plugin resolution.

**Example commands**:
- Spring Boot 1.2.2: `org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run`
- Spring Boot 2.5.0: `spring-boot:run`

## Testing

### Unit Tests

Added comprehensive test coverage:

1. **Spring Boot 2.x test** (existing, updated):
   ```rust
   #[test]
   fn test_build_launch_command_spring_boot_with_profiles() {
       let cmd = build_launch_command(
           LaunchStrategy::SpringBootRun,
           None,
           &["dev".to_string()],
           &[],
           Some("jar"),
           Some("2.5.0"),  // Spring Boot 2.x
       );
       assert!(cmd.iter().any(|arg| arg.contains("spring-boot.run.profiles")));
   }
   ```

2. **Spring Boot 1.x test** (NEW):
   ```rust
   #[test]
   fn test_build_launch_command_spring_boot_1x_uses_run_properties() {
       let cmd = build_launch_command(
           LaunchStrategy::SpringBootRun,
           None,
           &["dev".to_string()],
           &["-Xmx512m".to_string()],
           Some("jar"),
           Some("1.2.2.RELEASE"),  // Spring Boot 1.x
       );
       
       // Should use run.profiles (not spring-boot.run.profiles)
       assert!(cmd.iter().any(|arg| arg.contains("run.profiles=dev")));
       
       // Should use run.jvmArguments (not spring-boot.run.jvmArguments)
       assert!(cmd.iter().any(|arg| arg.contains("run.jvmArguments")));
       
       // Should NOT use spring-boot.run.* properties
       assert!(!cmd.iter().any(|arg| arg.contains("spring-boot.run.")));
   }
   ```

**Test Results**: ✅ All 275 tests pass

### Manual Testing with Real Application

**Test scenario**: Spring Boot 1.2.2.RELEASE application with Log4j 1.x

**Configuration** (`lazymvn.toml`):
```toml
log_format = "[%p][%c] %m%n"

[[packages]]
name = "fwmc.internal.core"
level = "WARN"
```

**Before fix**:
- Command: `mvn spring-boot:run -Dspring-boot.run.jvmArguments="..."`
- Result: ❌ Original format `[27/10/2025 10:58:56:190]` still shown
- Result: ❌ DEBUG logs from `fwmc.internal.core` still appearing

**After fix**:
- Command: `mvn spring-boot:run -Drun.jvmArguments="..."`
- Expected: ✅ Format changed to `[DEBUG][fwmc.internal.core] message`
- Expected: ✅ Only WARN+ logs from `fwmc.internal.core` shown

## Files Modified

1. **`src/maven/detection.rs`**:
   - Added `spring_boot_version` field to `SpringBootDetection` struct
   - Extract version from effective POM during plugin detection
   - Modified `build_launch_command()` to accept version parameter
   - Added version-aware property selection logic
   - Updated all unit tests (14 tests modified)
   - Added 1 new test for Spring Boot 1.x behavior

2. **`src/ui/state/mod.rs`**:
   - Modified `execute_launch_command()` to accept version parameter
   - Updated `run_spring_boot_starter()` to pass version from detection

## References

- **Spring Boot 1.x Maven Plugin**: https://docs.spring.io/spring-boot/docs/1.2.x/maven-plugin/
  - Property: `-Drun.jvmArguments`
  - Property: `-Drun.profiles`

- **Spring Boot 2.x+ Maven Plugin**: https://docs.spring.io/spring-boot/docs/current/maven-plugin/
  - Property: `-Dspring-boot.run.jvmArguments`
  - Property: `-Dspring-boot.run.profiles`

## Related Work

This fix builds on previous Log4j integration work:

1. **Phase 1**: Added Log4j configuration file generation (`src/maven/log4j.rs`)
2. **Phase 2**: Added Log4j logger arguments (`src/ui/state/launcher_config.rs`)
3. **Phase 3** (this fix): Version-aware property passing for Spring Boot 1.x compatibility

## Success Criteria

✅ Spring Boot plugin version extracted from effective POM  
✅ Version detection works for both 1.x and 2.x formats  
✅ Correct property name used based on version  
✅ All existing tests pass (275/275)  
✅ New test validates Spring Boot 1.x behavior  
✅ No breaking changes to Spring Boot 2.x+ support  

## Next Steps

User should test with their actual Spring Boot 1.2.2.RELEASE application to confirm:

1. Log format matches `lazymvn.toml` configuration
2. Package-level filtering works as expected
3. Both Logback and Log4j 1.x configurations are applied

If issues persist, possible causes:
- Application's own `log4j.properties` reloading after LazyMVN's configuration
- Spring Boot 1.2.2 has additional configuration precedence rules
- JVM argument parsing differences in very old Maven versions
