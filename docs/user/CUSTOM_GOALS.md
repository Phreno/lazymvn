# Custom Maven Goals

Execute custom Maven plugin goals directly from LazyMVN using a dedicated popup interface.

## Overview

Custom goals allow you to quickly execute Maven plugin invocations (like code formatters, quality checks, code generators, etc.) without leaving the TUI. Unlike Maven flags (which are options like `-DskipTests`), goals are complete Maven plugin executions.

## Usage

### Opening the Popup

Press **`Ctrl+G`** to open the custom goals popup. The popup displays:
- A numbered list of configured goals
- The goal name (left side)
- The Maven goal coordinate (right side)
- Navigation controls in the footer

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate through the goal list |
| `Enter` | Execute the selected goal on the current module |
| `Esc` / `q` | Close the popup without executing |

### Execution

When you press `Enter`:
1. The popup closes
2. The selected Maven goal executes on the currently focused module
3. Output appears in the Output pane (just like any Maven command)
4. You can kill the execution with `Esc` if needed

## Configuration

Define custom goals in your `lazymvn.toml` configuration file:

```toml
[maven]
custom_goals = [
    { name = "Format Code", goal = "net.revelc.code.formatter:formatter-maven-plugin:2.23.0:format" },
    { name = "Checkstyle", goal = "checkstyle:check" },
    { name = "Dependency Tree", goal = "dependency:tree" },
]
```

### Goal Formats

You can use two formats for Maven goals:

1. **Full coordinates** (recommended for reproducibility):
   ```toml
   { name = "Format", goal = "net.revelc.code.formatter:formatter-maven-plugin:2.23.0:format" }
   ```
   Format: `groupId:artifactId:version:goal`

2. **Short form** (requires plugin in your POM):
   ```toml
   { name = "Format", goal = "formatter:format" }
   ```
   Format: `pluginPrefix:goal`

### Configuration Fields

- **`name`** (required): Display name shown in the popup (keep it short and descriptive)
- **`goal`** (required): Maven plugin goal to execute (can include additional arguments)

### Examples

See `examples/lazymvn.toml.custom-goals-example` for a comprehensive list of examples including:
- Code formatters (formatter-maven-plugin)
- Quality checks (checkstyle, PMD, SpotBugs)
- Code generators (jOOQ, OpenAPI)
- Documentation (Javadoc, site)
- Dependency management (dependency tree/analyze, versions)
- Docker/containers (Jib)
- Framework-specific (Spring Boot, Quarkus, Micronaut)
- License management

## Common Use Cases

### Code Formatting

```toml
custom_goals = [
    { name = "Format Code", goal = "net.revelc.code.formatter:formatter-maven-plugin:2.23.0:format" },
    { name = "Sort POM", goal = "com.github.ekryd.sortpom:sortpom-maven-plugin:3.3.0:sort" },
]
```

### Code Quality

```toml
custom_goals = [
    { name = "Checkstyle", goal = "org.apache.maven.plugins:maven-checkstyle-plugin:3.3.1:check" },
    { name = "PMD", goal = "org.apache.maven.plugins:maven-pmd-plugin:3.21.2:check" },
    { name = "SpotBugs", goal = "com.github.spotbugs:spotbugs-maven-plugin:4.8.3.0:check" },
]
```

### Dependency Analysis

```toml
custom_goals = [
    { name = "Dependency Tree", goal = "dependency:tree" },
    { name = "Analyze Dependencies", goal = "dependency:analyze" },
    { name = "Check Updates", goal = "versions:display-dependency-updates" },
]
```

### Spring Boot

```toml
custom_goals = [
    { name = "Build Native Image", goal = "spring-boot:build-image" },
    { name = "Build Info", goal = "spring-boot:build-info" },
]
```

### Docker/Containers

```toml
custom_goals = [
    { name = "Build Docker (Jib)", goal = "com.google.cloud.tools:jib-maven-plugin:3.4.0:dockerBuild" },
    { name = "Push to Registry", goal = "com.google.cloud.tools:jib-maven-plugin:3.4.0:build" },
]
```

## Tips

1. **Keep names short**: The popup has limited width, so use concise names
2. **Use full coordinates**: Ensures the exact version is used, even if the plugin isn't in your POM
3. **Group related goals**: Organize goals logically (formatting, quality, deployment)
4. **Test goals first**: Run them from the command line to ensure they work as expected
5. **Module-scoped**: Goals execute on the currently focused module (respects `-pl` flag)

## Configuration Reload

Changes to `lazymvn.toml` are applied when:
- LazyMVN starts (loads configuration automatically)
- You press `Ctrl+E` to edit the config (reloads after editor closes)
- You switch projects with `Ctrl+R`

No manual restart needed!

## Error Handling

If no custom goals are configured and you press `Ctrl+G`, LazyMVN displays a helpful error message in the Output pane with configuration instructions.

## Differences from Custom Flags

| Feature | Custom Goals | Custom Flags |
|---------|-------------|--------------|
| **Purpose** | Execute Maven plugins | Add Maven options |
| **Popup** | Yes (`Ctrl+G`) | No (appears in Flags pane) |
| **Format** | `plugin:goal` | `-Dproperty=value` or `--option` |
| **Toggle** | Execute once | Enable/disable |
| **Example** | `formatter:format` | `-DskipTests=true` |

Use **custom goals** for one-time plugin executions (format, generate, check).  
Use **custom flags** for persistent options that modify Maven behavior.

## Related

- [Custom Flags Documentation](./CUSTOM_FLAGS.md)
- [Configuration Guide](../examples/README.md)
- [Spring Boot Launcher](./SPRING_BOOT_LAUNCHER.md)
