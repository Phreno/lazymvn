# Spring Boot 1.x JVM Arguments Fix - Quick Summary

## What Was Fixed

LazyMVN now correctly passes JVM arguments (including Log4j/Logback configuration) to **Spring Boot 1.x applications** by using the correct Maven property syntax.

## The Problem

Spring Boot 1.x (e.g., 1.2.2.RELEASE) uses different Maven properties than Spring Boot 2.x+:

| Spring Boot Version | JVM Args Property | Profiles Property |
|---------------------|-------------------|-------------------|
| **1.x** | `-Drun.jvmArguments` | `-Drun.profiles` |
| **2.x+** | `-Dspring-boot.run.jvmArguments` | `-Dspring-boot.run.profiles` |

LazyMVN was only using the 2.x+ syntax, causing JVM arguments to be ignored in Spring Boot 1.x applications.

## The Solution

LazyMVN now:

1. **Detects the Spring Boot plugin version** from your effective POM
2. **Automatically uses the correct property syntax** based on the detected version
3. **Uses full plugin coordinates for Spring Boot 1.x** to avoid Maven plugin resolution issues
4. **Maintains full compatibility** with both Spring Boot 1.x and 2.x+ applications

### What Changed?

**For Spring Boot 1.x (e.g., 1.2.2.RELEASE)**:
- Property: `-Drun.jvmArguments` (not `-Dspring-boot.run.jvmArguments`)
- Goal: `org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run` (full coordinates)

**For Spring Boot 2.x+ (e.g., 2.5.0)**:
- Property: `-Dspring-boot.run.jvmArguments`
- Goal: `spring-boot:run` (short prefix)

**Why full coordinates for 1.x?** Maven 3.8.2+ doesn't always resolve the `spring-boot` prefix correctly for very old plugin versions because the groupId structure changed. Using full coordinates bypasses plugin resolution.

## What You Need to Do

### 1. Rebuild LazyMVN

```bash
cd /workspaces/lazymvn
cargo build --release
```

### 2. Test with Your Application

Run LazyMVN with debug logging enabled:

```bash
cargo run --release -- --project /path/to/your/spring-boot-app --debug
```

### 3. Verify the Fix

Check the debug log (`~/.local/share/lazymvn/logs/debug.log`) for these indicators:

✅ **Version detected**:
```
Found Spring Boot plugin version: 1.2.2.RELEASE
```

✅ **Correct property used** (for Spring Boot 1.x):
```
Maven command: ... -Drun.jvmArguments="..." org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run
```

✅ **Application logs formatted correctly**:
- Custom format applied: `[DEBUG][fwmc.internal.core] message`
- Package filtering working: Only WARN+ logs from `fwmc.internal.core`

## Configuration

Your existing `lazymvn.toml` configuration should now work correctly:

```toml
log_format = "[%p][%c] %m%n"

[[packages]]
name = "fwmc.internal.core"
level = "WARN"
```

## Testing

Run the validation script to confirm the fix:

```bash
./scripts/test-spring-boot-1x-fix.sh
```

Expected output:
```
✓ Spring Boot 1.x version detection working
✓ Spring Boot 1.x uses -Drun.* properties
✓ Spring Boot 2.x uses -Dspring-boot.run.* properties
✓ All unit tests passing
```

## Files Changed

- `src/maven/detection.rs` - Version detection and property selection
- `src/ui/state/mod.rs` - Pass version to command builder
- `docs/internal/FIX_SPRING_BOOT_1X_JVM_ARGS.md` - Full technical documentation
- `scripts/test-spring-boot-1x-fix.sh` - Validation test script

## Next Steps

If log formatting/filtering still doesn't work after this fix, possible causes:

1. **Application reloading its own config**: Your application's `log4j.properties` might be loaded after LazyMVN's configuration
2. **Maven version issues**: Very old Maven versions might parse JVM arguments differently
3. **Classpath ordering**: Check the order of Log4j configuration files on the classpath

See `docs/internal/FIX_SPRING_BOOT_1X_JVM_ARGS.md` for detailed troubleshooting.

## Status

✅ **Code changes**: Complete  
✅ **Unit tests**: 275/275 passing  
✅ **Documentation**: Complete  
⏳ **Real-world validation**: Pending user testing

Please test with your Spring Boot 1.2.2 application and report results!
