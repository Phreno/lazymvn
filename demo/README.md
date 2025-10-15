# Lazymvn Demo Projects

This folder contains demonstration Maven projects for testing and showcasing `lazymvn` functionality.

## Projects

### 1. multi-module/
A multi-module Maven project demonstrating how `lazymvn` handles projects with multiple modules.

**Structure:**
- Parent POM with `<modules>` section
- `library/` - A shared library module with utility classes
- `app/` - An application module that depends on the library

**Features to test:**
- Module selection (arrow keys to switch between `library` and `app`)
- Module-scoped builds using `-pl` flag
- Inter-module dependencies
- Maven profiles: `dev` and `prod`
- Custom Maven settings file

**Use Case:** Large projects with shared code and multiple deployable artifacts.

### 2. single-module/
A simple single-module Maven project demonstrating how `lazymvn` handles projects without modules.

**Structure:**
- Single POM with no `<modules>` section
- Calculator application with main class and tests
- JUnit 4 test suite

**Features to test:**
- Displays as "(root project)" instead of module name
- Commands execute without `-pl` flag
- All standard Maven goals work: compile, test, package, install
- Maven profiles: `dev` and `prod`

**Use Case:** Simple applications, libraries, or standalone services.

## Quick Start

### Test Multi-Module Project
```bash
cd multi-module
lazymvn
# Use ↑/↓ to select a module
# Press 'b' to build selected module
# Press 'p' to view/toggle profiles
```

### Test Single-Module Project
```bash
cd single-module
lazymvn
# You'll see "(root project)" as the only module
# Press 't' to run tests
# Press 'k' to package
```

## What to Test

### 1. Module Display
- **Multi-module**: Should show `library` and `app` as separate entries
- **Single-module**: Should show `(root project)` as the only entry

### 2. Command Execution
Test these commands in both projects:
- `b` - Build (clean install)
- `c` - Compile
- `t` - Test
- `k` - Package
- `i` - Install
- `d` - Dependency tree

### 3. Profiles (press `p`)
Both projects have profiles:
- `dev` - Development environment settings
- `prod` - Production environment settings

Toggle profiles on/off with Space or Enter, then run a build command.

### 4. Build Flags (press `f`)
Test these flags:
- `Also Make` - Build module dependencies
- `Skip Tests` - Skip test execution
- `Update Snapshots` - Force update snapshots
- `Offline` - Work offline
- `Fail Fast` - Stop at first failure

### 5. Output Navigation
- Scroll with arrow keys or Page Up/Down
- Search with `/`, then enter a pattern
- Navigate matches with `n` (next) and `N` (previous)
- Watch for color-coded log levels (INFO, WARN, ERROR)

### 6. Cache Behavior
Test cache functionality:
```bash
# First run - creates cache
cd multi-module && lazymvn
# Quit with 'q'

# Second run - uses cache
lazymvn
# Should load instantly without parsing POM

# Modify pom.xml, then run lazymvn
# Should detect change and re-parse
```

### 7. Maven Settings
The multi-module project includes a `settings.xml` file. LazyMVN should automatically detect and use it for builds.

## Key Differences Between Projects

| Feature | Multi-Module | Single-Module |
|---------|--------------|---------------|
| **Module Display** | `library`, `app` | `(root project)` |
| **Maven Command** | `mvn -pl library test` | `mvn test` |
| **Cache Format** | `["library", "app"]` | `["."]` |
| **Navigation** | Select between modules | Single selection |
| **Use Case** | Complex projects | Simple projects |
| **Output Context** | Shows module name | Shows "(root project)" |

## Mock Maven Wrappers

Both projects include mock `mvnw` scripts that simulate Maven output without requiring actual Maven or Java. This makes the demos work immediately without dependencies.

The mock scripts simulate:
- Build lifecycle phases
- Test execution output
- Success/failure messages
- Dependency tree output
- Profile listing

## Troubleshooting

### Cache Issues
Clear the cache if you experience issues:
```bash
rm ~/.config/lazymvn/cache.json
```

### Debug Mode
Run with debug logging to see what's happening:
```bash
lazymvn --debug
# In another terminal:
tail -f lazymvn-debug.log
```

### Module Not Detected
Verify your POM has a `<modules>` section:
```bash
grep -A 5 "<modules>" pom.xml
```

## Real Project Testing

Once you understand the demos, try lazymvn with real Maven projects:

```bash
# Spring Boot application
git clone https://github.com/spring-projects/spring-petclinic.git
cd spring-petclinic
lazymvn

# Multi-module project
git clone https://github.com/apache/maven.git
cd maven
lazymvn
```
