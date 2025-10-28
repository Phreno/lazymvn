# Custom Maven Flags - Quick Start Guide

## What is it?

Custom Maven Flags allows you to define project-specific Maven arguments in your configuration that appear as toggleable flags in LazyMVN's interface.

## Quick Example

Add this to your `lazymvn.toml`:

```toml
[maven]
custom_flags = [
    { name = "Development mode", flag = "-Dspring.profiles.active=dev", enabled = true },
    { name = "Skip integration tests", flag = "-DskipITs=true" },
]
```

## How to Use

### Step 1: Add Configuration

Edit your configuration file:
```bash
# In LazyMVN: press Ctrl+E
# Or manually edit: ~/.config/lazymvn/projects/<hash>/config.toml
```

Add your custom flags in the `[maven]` section.

### Step 2: View in UI

1. Start LazyMVN
2. Press `f` to open the Flags panel
3. Your custom flags appear after the built-in flags

### Step 3: Toggle Flags

- Use `↑`/`↓` or `j`/`k` to navigate
- Press `Space` to toggle a flag on/off
- Enabled flags show as `[x]`, disabled as `[ ]`

### Step 4: Run Commands

All enabled flags are automatically included when you run Maven commands:
- `c` - compile
- `t` - test  
- `p` - package
- `s` - start
- etc.

## Common Examples

### Environment Profiles
```toml
[maven]
custom_flags = [
    { name = "Dev profile", flag = "-Dspring.profiles.active=dev", enabled = true },
    { name = "Staging profile", flag = "-Dspring.profiles.active=staging" },
]
```

### Skip Options
```toml
[maven]
custom_flags = [
    { name = "Skip ITs", flag = "-DskipITs=true" },
    { name = "Skip JavaDoc", flag = "-Dmaven.javadoc.skip=true" },
]
```

### Debug Options
```toml
[maven]
custom_flags = [
    { name = "Debug logging", flag = "-Dlogging.level.root=DEBUG" },
    { name = "Show SQL", flag = "-Dhibernate.show_sql=true" },
]
```

### Combined Properties
```toml
[maven]
custom_flags = [
    { name = "Fast build", flag = "-DskipTests -Dmaven.javadoc.skip=true" },
]
```

## Tips

### Default Enabled
Add `enabled = true` to activate a flag by default:
```toml
{ name = "My flag", flag = "-Dmy.prop=value", enabled = true }
```

### View Command
Press `y` to yank (copy) the full Maven command with all flags.

### Live Reload
After editing config, press `Ctrl+E` or `Ctrl+R` to reload without restarting.

### Per-Module
Flag states are saved per module, so different modules can have different configurations.

## Full Documentation

- **User Guide**: `docs/user/CUSTOM_FLAGS.md`
- **Examples**: `examples/lazymvn.toml.custom-flags-example`
- **Test Script**: `scripts/test-custom-flags.sh`

## Testing

Run the test script to verify your setup:
```bash
./scripts/test-custom-flags.sh
```

## Support

For issues or questions, see the main README.md or open an issue on GitHub.
