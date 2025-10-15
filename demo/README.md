# Lazymvn Demo Projects

This folder contains demonstration Maven projects for testing and showcasing `lazymvn` functionality.

## Projects

### 1. multi-module/
A multi-module Maven project demonstrating `lazymvn` with projects that have multiple modules.

**Structure:**
- `library/` - A shared library module
- `app/` - An application module that depends on the library

**Use Case:** Test module selection, scoped builds (`-pl`), and module-specific commands.

### 2. single-module/
A simple single-module Maven project demonstrating `lazymvn` with projects without modules.

**Structure:**
- Simple calculator application with tests
- No `<modules>` section in pom.xml

**Use Case:** Test that `lazymvn` properly handles single-module projects by treating them as a root project.

## Testing Both Projects

### Multi-Module Project
```bash
cd multi-module
lazymvn
# You should see: library, app
# Commands are scoped with -pl <module>
```

### Single-Module Project
```bash
cd single-module
lazymvn
# You should see: (root project)
# Commands run without -pl flag
```

## What to Test

1. **Module Display**: Verify that multi-module shows module names and single-module shows "(root project)"
2. **Command Execution**: Test build, compile, test commands in both projects
3. **Profiles**: Both projects have profiles (dev/prod) that can be toggled
4. **Build Flags**: Test flags like --also-make, -DskipTests, etc.
5. **Output Display**: Verify Maven output is captured and displayed correctly
6. **Cache Behavior**: Test that switching between projects updates the cache properly

## Key Differences

| Feature | Multi-Module | Single-Module |
|---------|--------------|---------------|
| Module Display | "library", "app" | "(root project)" |
| Maven Command | `mvn -pl <module> ...` | `mvn ...` |
| Cache Entry | `["library", "app"]` | `["."]` |
| Use Case | Large projects with shared code | Simple applications, libraries |
