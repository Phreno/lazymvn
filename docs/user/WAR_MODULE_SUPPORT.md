# WAR Module Support with `exec:java`

## Problem

When running WAR-packaged Maven modules with `exec:java`, you may encounter:

```
java.lang.NoClassDefFoundError: javax/servlet/Filter
```

This happens because:
1. WAR modules typically have servlet dependencies with `provided` scope
2. `exec-maven-plugin` uses `runtime` scope by default
3. `provided` dependencies are excluded from the runtime classpath

## Solution

LazyMVN automatically detects WAR packaging and adjusts the classpath scope when using `exec:java`.

### Automatic Fix

When you launch a WAR module with the **Start** command (`s` key), LazyMVN:

1. Detects the module's packaging type from the POM
2. If packaging is `war`, automatically adds `-Dexec.classpathScope=compile`
3. Also adds `-Dexec.cleanupDaemonThreads=false` for better shutdown

### Generated Command

For a WAR module, LazyMVN generates:

```bash
mvn exec:java \
  -Dexec.mainClass=com.example.Application \
  -Dexec.classpathScope=compile \
  -Dexec.cleanupDaemonThreads=false
```

### Manual Override

If you need to run `exec:java` manually outside LazyMVN:

```bash
mvn exec:java \
  -Dexec.mainClass=your.main.Class \
  -Dexec.classpathScope=compile
```

Or with full GAV (recommended):

```bash
mvn org.codehaus.mojo:exec-maven-plugin:3.3.0:java \
  -Dexec.mainClass=your.main.Class \
  -Dexec.classpathScope=compile \
  -Dexec.cleanupDaemonThreads=false
```

## How It Works

### Classpath Scopes

| Scope | Description | Included in `runtime` | Included in `compile` |
|-------|-------------|----------------------|---------------------|
| `compile` | Default scope | ✅ | ✅ |
| `runtime` | Only at runtime | ✅ | ❌ |
| `provided` | Container provides | ❌ | ✅ |
| `test` | Test only | ❌ | ❌ |

For WAR modules:
- Servlet API dependencies are typically `provided` (container supplies them)
- Using `classpathScope=compile` includes these dependencies at runtime
- This allows running the WAR as a standalone application with `exec:java`

### Detection Logic

LazyMVN checks:
1. Module's packaging type in effective POM
2. If `<packaging>war</packaging>`, applies the fix
3. For JAR or other packaging, uses default classpath scope

## Testing

LazyMVN includes tests for this behavior:

```rust
#[test]
fn test_build_launch_command_exec_java_war_packaging() {
    // Verifies classpathScope=compile is added for WAR
}

#[test]
fn test_build_launch_command_exec_java_jar_packaging() {
    // Verifies classpathScope is NOT added for JAR
}
```

Run tests:

```bash
cargo test test_build_launch_command
```

## Benefits

✅ **No POM changes required** - Works with existing projects  
✅ **Automatic detection** - No manual configuration needed  
✅ **Transparent** - LazyMVN handles it behind the scenes  
✅ **Logging** - Debug logs show when fix is applied  

## Related

- [Spring Boot Launcher](SPRING_BOOT_LAUNCHER.md) - For Spring Boot WAR modules
- [Maven Exec Plugin Docs](https://www.mojohaus.org/exec-maven-plugin/java-mojo.html#classpathScope)

## References

- Issue: `NoClassDefFoundError: javax/servlet/Filter` in WAR modules
- Solution: `-Dexec.classpathScope=compile`
- Implementation: `src/maven/detection.rs` - `build_launch_command()`
