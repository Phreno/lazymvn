# Spring Boot Run with --also-make Fix

## Problem Identified (2025-10-27 18:34)

### Symptom

**Maven Error when running Spring Boot application:**
```
[INFO] --- spring-boot-maven-plugin:1.2.2.RELEASE:run (default-cli) @ h6g-parent ---
[ERROR] Failed to execute goal ... on project h6g-parent: 
        Unable to find a suitable main class, please add a 'mainClass' property
```

### Root Cause

When using `spring-boot:run` with Maven reactor flags `--also-make` and `--also-make-dependents`, Maven executes the `run` goal on **ALL modules in the reactor**, including the **parent POM**.

Parent POMs have packaging type `<packaging>pom</packaging>` and have **no mainClass**, causing the error.

## Why This Happens

Maven reactor flags behavior:
- `--also-make` (or `-am`): Build all dependencies of selected module
- `--also-make-dependents` (or `-amd`): Build all modules that depend on selected module

These flags work perfectly for **build goals** like `compile`, `test`, `package`.

But for **execution goals** like `spring-boot:run`, Maven applies the goal to **every module** in the reactor:

```bash
mvn -pl my-web-app --also-make --also-make-dependents spring-boot:run
```

**What happens:**
1. Maven identifies reactor modules: `parent`, `library`, `my-web-app`
2. Builds dependencies with `--also-make` ✅
3. **Executes `spring-boot:run` on EACH module** ❌:
   - `parent` → ❌ No mainClass (POM packaging)
   - `library` → ❌ No mainClass (JAR library)
   - `my-web-app` → ✅ Has mainClass

**Result:** Fails on parent POM before reaching the actual web app.

## Solution: Automatic Flag Filtering

### Implementation

In `src/maven/command.rs`, we now **automatically filter** `--also-make` and `--also-make-dependents` when detecting `spring-boot:run`:

```rust
// Filter incompatible flags for spring-boot:run
let is_spring_boot_run = args.iter().any(|arg| {
    arg.contains("spring-boot:run") || 
    arg.contains("spring-boot-maven-plugin") && arg.contains(":run")
});

let filtered_flags: Vec<&String> = if is_spring_boot_run {
    flags.iter()
        .filter(|flag| {
            let flag_lower = flag.to_lowercase();
            !flag_lower.contains("also-make")
        })
        .collect()
} else {
    flags.iter().collect()
};
```

### Logging

When flags are filtered, we log:
```
WARN - Filtered out --also-make flags for spring-boot:run 
       (would execute on all reactor modules including parent POM)
DEBUG - Original flags: ["--also-make", "--also-make-dependents", "-DskipTests"]
DEBUG - Filtered flags: ["-DskipTests"]
```

## User Impact

**Before Fix:**  
```bash
$ mvn -pl module --also-make --also-make-dependents spring-boot:run
[ERROR] Unable to find a suitable main class (on parent POM)
```

**After Fix:**  
```bash
$ mvn -pl module -DskipTests spring-boot:run  # Flags auto-filtered
[INFO] Running application successfully ✅
```

**User-facing change:**  
- LazyMVN automatically removes `--also-make` flags when using `spring-boot:run`
- Users see a warning in debug logs explaining why
- No action required - just rebuild and run

## Testing

1. **Manual testing:**  
   - Enable `--also-make` and `--also-make-dependents` flags in LazyMVN
   - Press `s` to run Spring Boot starter
   - Verify: Application starts successfully (no parent POM error)
   - Check logs: Should see warning about filtered flags

2. **Command validation:**  
   - Expected command (after fix):
     ```bash
     mvn.cmd --settings ... -pl module -DskipTests \
       -Drun.jvmArguments=... spring-boot:run
     ```
   - Notice: NO `--also-make` or `--also-make-dependents` flags

3. **Existing tests:**  
   - All 60 maven module tests pass ✅
   - No regression in other Maven commands

## Maven Documentation References

From [Maven Reactor Options](https://maven.apache.org/guides/mini/guide-multiple-modules.html):

> **-am, --also-make**: If project list is specified, also build projects required by the list
> 
> **-amd, --also-make-dependents**: If project list is specified, also build projects that depend on projects on the list

**Important note:**  
These flags are designed for **build phases** (compile, test, package), not **execution goals** (spring-boot:run, exec:java).

## Code Locations

Modified files:
- `src/maven/command.rs` (lines 307-339): Flag filtering in `execute_maven_command_async()`
- `src/maven/command.rs` (lines 151-178): Flag filtering in `build_command_string_with_options()`

Implementation applies to:
1. Actual Maven command execution (`Command::arg()`)
2. Display string for logs (`build_command_string_with_options()`)

## Related Issues & Future Work

### Potential Edge Cases

1. **exec:java with --also-make:**  
   Currently we **auto-ADD** `--also-make` for `exec:java` (line 295).  
   Question: Does `exec:java` have the same problem?  
   → Needs testing with multi-module exec:java setup

2. **Other execution plugins:**  
   - `jetty:run`
   - `tomcat:run`
   - `quarkus:dev`
   
   → May need similar filtering if they exhibit same behavior

3. **User override:**  
   Currently no way to force `--also-make` if user really wants it.  
   → Could add config option like `spring.force_reactor_flags = true`

### Build vs Run Workflows

**Best practice recommendation:**

```bash
# Step 1: Build all dependencies (use --also-make)
mvn -pl my-web-app --also-make clean package

# Step 2: Run only the web app (NO --also-make)
mvn -pl my-web-app spring-boot:run
```

LazyMVN now enforces this automatically for `spring-boot:run`.

---

**Last Updated:** 2025-10-27  
**Version:** LazyMVN 0.4.0-nightly  
**Related Fixes:** See also WINDOWS_ARGS_FIX.md for other argument parsing fixes
