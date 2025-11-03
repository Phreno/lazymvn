# Fix Spring Boot 1.x Plugin Resolution Issue

## Date
2025-11-03

## Issue
**User Report**: Maven build failing with Spring Boot 1.4.13 (and other 1.x versions)

```
[ERROR] Plugin org.springframework.boot:spring-boot-maven-plugin:1.4.13 or one of its dependencies could not be resolved: 
Could not find artifact org.springframework.boot:spring-boot-maven-plugin:jar:1.4.13
```

**Root Cause**: LazyMVN was generating an **invalid Maven command syntax** for Spring Boot 1.x:
```bash
mvn org.springframework.boot:spring-boot-maven-plugin:1.4.13:run
```

Maven interpreted this as trying to download the plugin as a JAR dependency rather than executing the plugin goal.

## Analysis

### Incorrect Behavior (Before Fix)
The code in `src/maven/detection/command_builder.rs` was:

```rust
let goal = if is_spring_boot_1x && spring_boot_version.is_some() {
    format!(
        "org.springframework.boot:spring-boot-maven-plugin:{}:run",
        spring_boot_version.unwrap()
    )
} else {
    "spring-boot:run".to_string()
};
```

This generated commands like:
- `org.springframework.boot:spring-boot-maven-plugin:1.4.13:run`
- `org.springframework.boot:spring-boot-maven-plugin:1.5.10.RELEASE:run`

### Maven Plugin Invocation Rules

Maven supports two ways to invoke plugins:

1. **Short form** (requires plugin in POM): `mvn plugin-prefix:goal`
   - Example: `mvn spring-boot:run`
   - Plugin version/configuration taken from `pom.xml`

2. **Fully-qualified form** (without version): `mvn groupId:artifactId:goal`
   - Example: `mvn org.springframework.boot:spring-boot-maven-plugin:run`
   - Uses **latest version** from repositories

3. **INVALID**: `mvn groupId:artifactId:VERSION:goal`
   - Maven tries to resolve this as a dependency, not a plugin
   - **This is what LazyMVN was incorrectly generating**

### Correct Behavior (After Fix)

Now generates:
```bash
mvn spring-boot:run
```

For **both Spring Boot 1.x and 2.x**, with the plugin version defined in the project's `pom.xml`:

```xml
<build>
    <plugins>
        <plugin>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-maven-plugin</artifactId>
            <version>1.4.13</version>
        </plugin>
    </plugins>
</build>
```

## Fix Details

### Code Changes

**File**: `src/maven/detection/command_builder.rs`

**Before**:
```rust
let goal = if is_spring_boot_1x && spring_boot_version.is_some() {
    format!(
        "org.springframework.boot:spring-boot-maven-plugin:{}:run",
        spring_boot_version.unwrap()
    )
} else {
    "spring-boot:run".to_string()
};
```

**After**:
```rust
// For Spring Boot 1.x and 2.x, use the same goal
// The plugin version must be defined in the POM, not in the command line
let goal = "spring-boot:run".to_string();
```

### Property Differences Preserved

The fix **does NOT change** the property names used for Spring Boot 1.x vs 2.x:

- **Spring Boot 1.x**: Uses `run.profiles` and `run.jvmArguments`
- **Spring Boot 2.x**: Uses `spring-boot.run.profiles` and `spring-boot.run.jvmArguments`

This distinction is still preserved in the code and is correct.

### Tests Updated

1. **`test_build_launch_command_spring_boot_1x_uses_run_properties`**
   - Updated assertion: expects `"spring-boot:run"` instead of fully-qualified syntax

2. **`test_build_launch_command_spring_boot_1x_with_jvm_args`**
   - Updated assertion: expects `"spring-boot:run"` instead of fully-qualified syntax

3. **New test**: `test_spring_boot_1x_version_1_4_13_fix`
   - Regression test specifically for the reported bug
   - Validates that the invalid fully-qualified syntax is NOT generated
   - Confirms correct `spring-boot:run` goal is used

## Testing

### Automated Tests
```bash
cargo test --lib maven::detection::command_builder
# Result: 11 passed; 0 failed
```

### Manual Validation
To test with a real Spring Boot 1.x project:

1. Build lazymvn with fix: `cargo build --release`
2. Navigate to project with Spring Boot 1.4.13
3. Run in LazyMVN
4. Verify command in debug logs shows: `spring-boot:run`
5. Verify build succeeds

## Impact

### Fixed
- ✅ Spring Boot 1.x projects (1.0.x - 1.5.x) now run correctly
- ✅ No more "plugin JAR not found" errors
- ✅ Consistent behavior between Spring Boot 1.x and 2.x

### Unchanged
- ✅ Spring Boot 2.x behavior unchanged (already correct)
- ✅ JVM arguments still use correct property names per version
- ✅ Profile handling still uses correct property names per version

### Requirements
Projects using this fix **must have** the Spring Boot Maven plugin properly configured in their `pom.xml`:

```xml
<build>
    <plugins>
        <plugin>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-maven-plugin</artifactId>
            <version>1.4.13</version> <!-- or any 1.x version -->
        </plugin>
    </plugins>
</build>
```

This is **standard practice** for all Spring Boot projects, so no user action required.

## Related Files
- `src/maven/detection/command_builder.rs` - Main fix location
- `src/maven/command/helpers.rs` - Contains Spring Boot detection logic
- `src/maven/detection/spring_boot.rs` - Spring Boot version detection

## References
- Maven Plugin Prefix Resolution: https://maven.apache.org/guides/introduction/introduction-to-plugin-prefix-mapping.html
- Spring Boot Maven Plugin docs: https://docs.spring.io/spring-boot/maven-plugin/
- User report: Debug report dated 2025-11-03 21:18:21
